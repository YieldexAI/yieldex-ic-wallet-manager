use std::cell::RefCell;

use alloy::{
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
    signers::Signer,
    sol,
    transports::icp::IcpConfig,
};
use candid::Principal;
use crate::{PRINCIPAL_TO_ADDRESS_MAP, StorablePrincipal, PERMISSIONS_MAP, StorableString};
use crate::services::permissions::{verify_protocol_permission, set_daily_usage};
use crate::services::rpc_service::{get_rpc_service_by_chain_id, ARBITRUM_CHAIN_ID};

thread_local! {
    static COMPOUND_NONCE: RefCell<Option<u64>> = const { RefCell::new(None) };
}

// Compound Comet addresses for Arbitrum mainnet
// Native USDC market (Compound III cUSDCv3 proxy address)
const COMPOUND_COMET_USDC_ADDRESS: &str = "0x9c4ec768c28520b50860ea7a15bd7213a9ff58bf"; // cUSDCv3 Proxy for USDC on Arbitrum

// Codegen from ABI file to interact with Compound Comet contract
sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    CompoundComet,
    "src/abi/CompoundComet.json"
);

// Codegen from ABI file to interact with ERC20 tokens
sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    USDC,
    "src/abi/USDC.json"
);

/// Helper function to create ICP signer for a specific principal
async fn create_icp_signer_for_principal(principal: Principal) -> Result<alloy::signers::icp::IcpSigner, String> {
    let derivation_path = vec![principal.as_slice().to_vec()];
    let ecdsa_key_name = get_ecdsa_key_name();
    
    alloy::signers::icp::IcpSigner::new(derivation_path, &ecdsa_key_name, None)
        .await
        .map_err(|e| format!("Failed to create ICP signer: {}", e))
}

fn get_ecdsa_key_name() -> String {
    #[allow(clippy::option_env_unwrap)]
    let dfx_network = option_env!("DFX_NETWORK").unwrap_or("local");
    match dfx_network {
        "local" => "dfx_test_key".to_string(),
        "ic" => "key_1".to_string(),
        _ => "dfx_test_key".to_string(), // Default fallback
    }
}

/// Get Compound Comet contract address based on chain_id
fn get_compound_comet_address(chain_id: u64) -> Result<&'static str, String> {
    match chain_id {
        ARBITRUM_CHAIN_ID => Ok(COMPOUND_COMET_USDC_ADDRESS),
        _ => Err(format!("Compound not supported on chain_id: {}", chain_id))
    }
}

/// Supply USDC to Compound with permission verification
pub async fn supply_usdc_to_compound_with_permissions(
    amount_human: String,
    permissions_id: String,
    user_principal: Principal
) -> Result<String, String> {
    ic_cdk::println!("🚀 Starting Compound USDC supply: {} USDC for principal {}", amount_human, user_principal);
    
    // 1. Get permissions and chain_id
    ic_cdk::println!("✅ Step 1: Getting permissions and chain_id...");
    let permissions = PERMISSIONS_MAP.with(|map| {
        map.borrow()
            .get(&StorableString(permissions_id.clone()))
            .ok_or_else(|| "Permissions not found".to_string())
            .map(|p| p.0.clone())
    })?;
    
    // Check ownership
    if permissions.owner != user_principal {
        return Err("Not authorized to use these permissions".to_string());
    }
    
    let chain_id = permissions.chain_id;
    ic_cdk::println!("✅ Step 1 Complete: Using chain_id: {}", chain_id);
    
    // 2. Get RPC service for the chain
    let rpc_service = get_rpc_service_by_chain_id(chain_id)?;
    let config = IcpConfig::new(rpc_service);
    
    // 3. Get Compound contract address
    let compound_address = get_compound_comet_address(chain_id)?;
    ic_cdk::println!("✅ Using Compound Comet address: {}", compound_address);
    
    // 4. Check permissions
    ic_cdk::println!("✅ Step 2: Verifying Compound permissions...");
    verify_compound_permission(&permissions_id, "supply", &amount_human, user_principal, chain_id).await?;
    ic_cdk::println!("✅ Step 2 Complete: Compound permissions verified");
    
    // 5. Convert amount (USDC has 6 decimals)
    ic_cdk::println!("✅ Step 3: Converting amount {} USDC to units...", amount_human);
    let amount_units = parse_usdc_amount(&amount_human)?;
    ic_cdk::println!("✅ Step 3 Complete: Amount converted to {} units", amount_units);
    
    // 6. Create signer on behalf of user
    ic_cdk::println!("✅ Step 4: Creating ICP signer for principal...");
    let signer = create_icp_signer_for_principal(user_principal).await?;
    let address = signer.address();
    ic_cdk::println!("✅ Step 4 Complete: Signer created for address: {}", address);
    
    // 7. Setup provider and contract
    let wallet = EthereumWallet::from(signer);
    let provider = ProviderBuilder::new()
        .with_gas_estimation()
        .wallet(wallet)
        .on_icp(config);

    // Handle nonce management (same pattern as AAVE)
    let maybe_nonce = COMPOUND_NONCE.with_borrow(|maybe_nonce| {
        maybe_nonce.map(|nonce| nonce + 1)
    });

    let nonce = if let Some(nonce) = maybe_nonce {
        ic_cdk::println!("Using cached nonce: {}", nonce);
        nonce
    } else {
        let fresh_nonce = provider.get_transaction_count(address).await
            .map_err(|e| format!("Failed to get nonce: {}", e))?;
        ic_cdk::println!("Got fresh nonce from network: {}", fresh_nonce);
        fresh_nonce
    };
    
    // 8. First approve USDC spending by Compound
    ic_cdk::println!("✅ Step 5: Approving USDC spending by Compound...");
    let usdc_address = get_usdc_address(chain_id)?;
    let usdc_contract = USDC::new(usdc_address.parse::<Address>().unwrap(), &provider);
    
    let approve_call = usdc_contract
        .approve(compound_address.parse::<Address>().unwrap(), amount_units)
        .nonce(nonce)
        .chain_id(chain_id)
        .from(address);

    let approve_receipt = approve_call.send().await.map_err(|e| {
        ic_cdk::println!("❌ USDC approve failed: {}", e);
        format!("USDC approve failed: {}", e)
    })?;

    let approve_tx_hash = *approve_receipt.tx_hash();
    ic_cdk::println!("✅ Step 5 Complete: USDC approved, hash: {:?}", approve_tx_hash);

    // Wait for approve transaction and update nonce cache
    let approve_tx_response = provider.get_transaction_by_hash(approve_tx_hash).await
        .map_err(|e| format!("Failed to get approve transaction: {}", e))?;

    match approve_tx_response {
        Some(tx) => {
            COMPOUND_NONCE.with_borrow_mut(|nonce_cache| {
                *nonce_cache = Some(tx.nonce);
            });
            ic_cdk::println!("Approve tx confirmed, nonce cache updated to {}", tx.nonce);
        }
        None => {
            return Err("Approve transaction not found after sending".to_string());
        }
    }
    
    // 9. Supply USDC to Compound
    ic_cdk::println!("✅ Step 6: Supplying USDC to Compound...");
    let compound_contract = CompoundComet::new(compound_address.parse::<Address>().unwrap(), &provider);

    // Get next nonce for supply transaction
    let supply_nonce = nonce + 1;

    let supply_call = compound_contract
        .supply(usdc_address.parse::<Address>().unwrap(), amount_units)
        .nonce(supply_nonce)
        .chain_id(chain_id)
        .from(address);

    let supply_receipt = supply_call.send().await.map_err(|e| {
        ic_cdk::println!("❌ Compound supply failed: {}", e);
        format!("Compound supply failed: {}", e)
    })?;

    let supply_tx_hash = *supply_receipt.tx_hash();
    ic_cdk::println!("✅ Step 6 Complete: USDC supplied to Compound, hash: {:?}", supply_tx_hash);

    // Wait for supply transaction and update nonce cache
    let supply_tx_response = provider.get_transaction_by_hash(supply_tx_hash).await
        .map_err(|e| format!("Failed to get supply transaction: {}", e))?;

    match supply_tx_response {
        Some(tx) => {
            COMPOUND_NONCE.with_borrow_mut(|nonce_cache| {
                *nonce_cache = Some(tx.nonce);
            });
            ic_cdk::println!("Supply tx confirmed, nonce cache updated to {}", tx.nonce);
        }
        None => {
            return Err("Supply transaction not found after sending".to_string());
        }
    }
    
    // 10. Update daily usage for permissions
    ic_cdk::println!("✅ Step 7: Updating protocol usage tracking...");
    let usage_result = set_daily_usage(
        permissions_id,
        compound_address.to_string(),
        amount_units.try_into().unwrap_or(0),
        user_principal
    );
    
    match usage_result {
        Ok(_) => ic_cdk::println!("✅ Step 7 Complete: Usage tracking updated"),
        Err(e) => ic_cdk::println!("⚠️ Step 7 Warning: Usage tracking failed: {}", e),
    }
    
    let tx_hash = format!("{:?}", supply_tx_hash);
    let success_message = format!(
        "✅ Successfully supplied {} USDC to Compound! Transaction: {}",
        amount_human, tx_hash
    );
    
    ic_cdk::println!("🎉 Compound supply completed successfully");
    Ok(success_message)
}

/// Withdraw USDC from Compound with permission verification
pub async fn withdraw_usdc_from_compound_with_permissions(
    amount_human: String,
    permissions_id: String,
    user_principal: Principal
) -> Result<String, String> {
    ic_cdk::println!("🚀 Starting Compound USDC withdrawal: {} USDC for principal {}", amount_human, user_principal);
    
    // 1. Get permissions and chain_id
    ic_cdk::println!("✅ Step 1: Getting permissions and chain_id...");
    let permissions = PERMISSIONS_MAP.with(|map| {
        map.borrow()
            .get(&StorableString(permissions_id.clone()))
            .ok_or_else(|| "Permissions not found".to_string())
            .map(|p| p.0.clone())
    })?;
    
    // Check ownership
    if permissions.owner != user_principal {
        return Err("Not authorized to use these permissions".to_string());
    }
    
    let chain_id = permissions.chain_id;
    ic_cdk::println!("✅ Step 1 Complete: Using chain_id: {}", chain_id);
    
    // 2. Get RPC service for the chain
    let rpc_service = get_rpc_service_by_chain_id(chain_id)?;
    let config = IcpConfig::new(rpc_service);
    
    // 3. Get Compound contract address
    let compound_address = get_compound_comet_address(chain_id)?;
    ic_cdk::println!("✅ Using Compound Comet address: {}", compound_address);
    
    // 4. Check permissions
    ic_cdk::println!("✅ Step 2: Verifying Compound withdraw permissions...");
    verify_compound_permission(&permissions_id, "withdraw", &amount_human, user_principal, chain_id).await?;
    ic_cdk::println!("✅ Step 2 Complete: Compound withdraw permissions verified");
    
    // 5. Convert amount (USDC has 6 decimals)
    ic_cdk::println!("✅ Step 3: Converting amount {} USDC to units...", amount_human);
    let amount_units = parse_usdc_amount(&amount_human)?;
    ic_cdk::println!("✅ Step 3 Complete: Amount converted to {} units", amount_units);
    
    // 6. Create signer on behalf of user
    ic_cdk::println!("✅ Step 4: Creating ICP signer for principal...");
    let signer = create_icp_signer_for_principal(user_principal).await?;
    let address = signer.address();
    ic_cdk::println!("✅ Step 4 Complete: Signer created for address: {}", address);
    
    // 7. Setup provider and contract
    let wallet = EthereumWallet::from(signer);
    let provider = ProviderBuilder::new()
        .with_gas_estimation()
        .wallet(wallet)
        .on_icp(config);

    // Handle nonce management (same pattern as AAVE)
    let maybe_nonce = COMPOUND_NONCE.with_borrow(|maybe_nonce| {
        maybe_nonce.map(|nonce| nonce + 1)
    });

    let nonce = if let Some(nonce) = maybe_nonce {
        ic_cdk::println!("Using cached nonce: {}", nonce);
        nonce
    } else {
        let fresh_nonce = provider.get_transaction_count(address).await
            .map_err(|e| format!("Failed to get nonce: {}", e))?;
        ic_cdk::println!("Got fresh nonce from network: {}", fresh_nonce);
        fresh_nonce
    };
    
    // 8. Check cUSDC balance before withdrawal
    ic_cdk::println!("✅ Step 5: Checking cUSDC balance...");
    let compound_contract = CompoundComet::new(compound_address.parse::<Address>().unwrap(), &provider);
    
    let user_balance = compound_contract
        .balanceOf(address)
        .call()
        .await
        .map_err(|e| format!("Failed to get Compound balance: {}", e))?;
    
    ic_cdk::println!("✅ Step 5 Complete: cUSDC balance: {} units", user_balance._0);
    
    if user_balance._0 < amount_units {
        let error_msg = format!("Insufficient cUSDC balance. Have: {} units, Need: {} units", user_balance._0, amount_units);
        ic_cdk::println!("❌ Compound withdrawal failed: {}", error_msg);
        return Err(error_msg);
    }
    
    // 9. Withdraw USDC from Compound
    ic_cdk::println!("✅ Step 6: Withdrawing USDC from Compound...");
    let usdc_address = get_usdc_address(chain_id)?;

    let withdraw_call = compound_contract
        .withdraw(usdc_address.parse::<Address>().unwrap(), amount_units)
        .nonce(nonce)
        .chain_id(chain_id)
        .from(address);

    let withdraw_receipt = withdraw_call.send().await.map_err(|e| {
        ic_cdk::println!("❌ Compound withdraw failed: {}", e);
        format!("Compound withdraw failed: {}", e)
    })?;

    let withdraw_tx_hash = *withdraw_receipt.tx_hash();
    ic_cdk::println!("✅ Step 6 Complete: USDC withdrawn from Compound, hash: {:?}", withdraw_tx_hash);

    // Wait for withdraw transaction and update nonce cache
    let withdraw_tx_response = provider.get_transaction_by_hash(withdraw_tx_hash).await
        .map_err(|e| format!("Failed to get withdraw transaction: {}", e))?;

    match withdraw_tx_response {
        Some(tx) => {
            COMPOUND_NONCE.with_borrow_mut(|nonce_cache| {
                *nonce_cache = Some(tx.nonce);
            });
            ic_cdk::println!("Withdraw tx confirmed, nonce cache updated to {}", tx.nonce);
        }
        None => {
            return Err("Withdraw transaction not found after sending".to_string());
        }
    }
    
    // 10. Update daily usage for permissions
    ic_cdk::println!("✅ Step 7: Updating protocol usage tracking...");
    let usage_result = set_daily_usage(
        permissions_id,
        compound_address.to_string(),
        amount_units.try_into().unwrap_or(0),
        user_principal
    );
    
    match usage_result {
        Ok(_) => ic_cdk::println!("✅ Step 7 Complete: Usage tracking updated"),
        Err(e) => ic_cdk::println!("⚠️ Step 7 Warning: Usage tracking failed: {}", e),
    }
    
    let tx_hash = format!("{:?}", withdraw_tx_hash);
    let success_message = format!(
        "✅ Successfully withdrew {} USDC from Compound! Transaction: {}",
        amount_human, tx_hash
    );
    
    ic_cdk::println!("🎉 Compound withdrawal completed successfully");
    Ok(success_message)
}

/// Get user's USDC balance in Compound
pub async fn get_compound_usdc_balance(address: Option<String>, chain_id: u64) -> Result<String, String> {
    ic_cdk::println!("🔍 Getting Compound USDC balance for chain_id: {}", chain_id);
    
    let rpc_service = get_rpc_service_by_chain_id(chain_id)?;
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new().on_icp(config);
    
    let compound_address = get_compound_comet_address(chain_id)?;
    let compound_contract = CompoundComet::new(compound_address.parse::<Address>().unwrap(), &provider);
    
    let user_address = match address {
        Some(addr) => addr.parse::<Address>().map_err(|e| format!("Invalid address: {}", e))?,
        None => {
            // Get address from current user's principal
            let caller = ic_cdk::caller();
            PRINCIPAL_TO_ADDRESS_MAP.with(|map| {
                map.borrow()
                    .get(&StorablePrincipal(caller))
                    .map(|addr| addr.0.parse::<Address>().unwrap())
                    .ok_or_else(|| "User does not have an EVM address".to_string())
            })?
        }
    };
    
    let balance = compound_contract
        .balanceOf(user_address)
        .call()
        .await
        .map_err(|e| format!("Failed to get Compound balance: {}", e))?;
    
    let balance_human = format_usdc_amount(balance._0);
    ic_cdk::println!(
        "📊 Compound USDC balance: {} (units: {})",
        balance_human,
        balance._0
    );
    Ok(balance_human)
}

// Helper functions

/// Verify Compound protocol permission
async fn verify_compound_permission(
    permissions_id: &str,
    function_name: &str,
    amount_human: &str,
    user_principal: Principal,
    chain_id: u64
) -> Result<(), String> {
    let compound_address = get_compound_comet_address(chain_id)?;
    let amount_units: u64 = parse_usdc_amount(amount_human)?.try_into().unwrap_or(0);
    
    let result = verify_protocol_permission(
        permissions_id.to_string(),
        compound_address.to_string(),
        function_name.to_string(),
        amount_units,
        user_principal
    )?;
    
    if result {
        Ok(())
    } else {
        Err("Protocol permission check failed".to_string())
    }
}

/// Parse human-readable USDC amount to units (6 decimals)
fn parse_usdc_amount(amount_human: &str) -> Result<U256, String> {
    let amount_f64: f64 = amount_human.parse()
        .map_err(|_| format!("Invalid amount format: {}", amount_human))?;
    
    if amount_f64 < 0.0 {
        return Err("Amount cannot be negative".to_string());
    }
    
    // USDC has 6 decimals
    let amount_units = (amount_f64 * 1_000_000.0) as u64;
    Ok(U256::from(amount_units))
}

/// Format USDC units to human-readable amount
fn format_usdc_amount(amount_units: U256) -> String {
    let amount_u64: u64 = amount_units.try_into().unwrap_or(0);
    let amount_f64 = amount_u64 as f64 / 1_000_000.0;
    format!("{:.6}", amount_f64)
}

/// Get USDC contract address based on chain_id
fn get_usdc_address(chain_id: u64) -> Result<&'static str, String> {
    match chain_id {
        ARBITRUM_CHAIN_ID => Ok("0xaf88d065e77c8cc2239327c5edb3a432268e5831"), // Native USDC on Arbitrum One
        _ => Err(format!("USDC not configured for chain_id: {}", chain_id))
    }
}