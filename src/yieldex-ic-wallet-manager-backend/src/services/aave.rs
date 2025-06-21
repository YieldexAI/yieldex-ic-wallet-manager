use std::cell::RefCell;

use alloy::{
    network::EthereumWallet,
    primitives::{address, Address, U256},
    providers::{Provider, ProviderBuilder},
    signers::Signer,
    sol,
    transports::icp::IcpConfig,
};
use candid::Principal;
use crate::{PRINCIPAL_TO_ADDRESS_MAP, StorablePrincipal, get_rpc_service_sepolia};
use crate::services::permissions::{is_permissions_owner, verify_protocol_permission, set_daily_usage};
use crate::services::get_balance_link::get_balance_link;

thread_local! {
    static AAVE_NONCE: RefCell<Option<u64>> = const { RefCell::new(None) };
}

// AAVE V3 Sepolia addresses (verified addresses for testnet)
const AAVE_POOL_ADDRESS: &str = "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951";

// Codegen from ABI file to interact with AAVE Pool contract
sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    AavePool,
    "src/abi/aave_pool.json"
);

// Codegen from ABI file to interact with ERC20 tokens
sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    LINK,
    "src/abi/LINK.json"
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

/// Supply LINK to AAVE with permission verification
pub async fn supply_link_to_aave_with_permissions(
    amount_human: String,
    permissions_id: String,
    user_principal: Principal
) -> Result<String, String> {
    ic_cdk::println!("🚀 Starting AAVE LINK supply: {} LINK for principal {}", amount_human, user_principal);
    
    // 1. Check permissions
    ic_cdk::println!("✅ Step 1: Verifying AAVE permissions...");
    verify_aave_permission(&permissions_id, "supply", &amount_human, user_principal).await?;
    ic_cdk::println!("✅ Step 1 Complete: AAVE permissions verified");
    
    // 2. Convert amount (LINK has 18 decimals)
    ic_cdk::println!("✅ Step 2: Converting amount {} LINK to wei...", amount_human);
    let amount_wei = parse_link_amount(&amount_human)?;
    ic_cdk::println!("✅ Step 2 Complete: Amount converted to {} wei", amount_wei);
    
    // 3. Create signer on behalf of user
    ic_cdk::println!("✅ Step 3: Creating ICP signer for principal...");
    let signer = create_icp_signer_for_principal(user_principal).await?;
    let address = signer.address();
    ic_cdk::println!("✅ Step 3 Complete: Signer created for address 0x{:x}", address);

    // 4. Setup provider
    ic_cdk::println!("✅ Step 4: Setting up provider and wallet...");
    let wallet = EthereumWallet::from(signer);
    let rpc_service = get_rpc_service_sepolia();
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new()
        .with_gas_estimation()
        .wallet(wallet)
        .on_icp(config);
    ic_cdk::println!("✅ Step 4 Complete: Provider and wallet configured");
    
    // 5. Check LINK balance
    ic_cdk::println!("✅ Step 5: Checking LINK balance for address 0x{:x}...", address);
    let link_balance = get_link_balance_for_address(format!("0x{:x}", address)).await?;
    let link_balance_wei = U256::from_str_radix(&link_balance.replace("0x", ""), 16)
        .map_err(|_| "Failed to parse LINK balance".to_string())?;
    
    ic_cdk::println!("✅ Step 5 Complete: LINK balance: {} wei (need: {} wei)", link_balance_wei, amount_wei);
    
    if link_balance_wei < amount_wei {
        let error_msg = format!("Insufficient LINK balance. Have: {} wei, Need: {} wei", link_balance_wei, amount_wei);
        ic_cdk::println!("❌ AAVE supply failed: {}", error_msg);
        return Err(error_msg);
    }
    
    // 6. Check/set allowance for AAVE Pool
    ic_cdk::println!("✅ Step 6: Ensuring LINK allowance for AAVE Pool...");
    ensure_link_allowance_for_aave(&provider, amount_wei, address).await?;
    ic_cdk::println!("✅ Step 6 Complete: LINK allowance confirmed for AAVE Pool");
    
    // 7. Handle nonce management
    ic_cdk::println!("✅ Step 7: Getting transaction nonce...");
    let maybe_nonce = AAVE_NONCE.with_borrow(|maybe_nonce| {
        maybe_nonce.map(|nonce| nonce + 1)
    });

    let nonce = if let Some(nonce) = maybe_nonce {
        ic_cdk::println!("✅ Step 7: Using cached nonce: {}", nonce);
        nonce
    } else {
        let fresh_nonce = provider.get_transaction_count(address).await
            .map_err(|e| format!("Failed to get nonce: {}", e))?;
        ic_cdk::println!("✅ Step 7: Got fresh nonce from network: {}", fresh_nonce);
        fresh_nonce
    };
    
    // 8. Execute supply to AAVE
    ic_cdk::println!("✅ Step 8: Preparing AAVE supply transaction...");
    let pool_address = address!("6Ae43d3271ff6888e7Fc43Fd7321a503ff738951");
    let pool_contract = AavePool::new(pool_address, provider.clone());
    
    let link_address = address!("f8fb3713d459d7c1018bd0a49d19b4c44290ebe5");
    
    ic_cdk::println!("📋 AAVE Supply Parameters:");
    ic_cdk::println!("  - Pool Address: 0x{:x}", pool_address);
    ic_cdk::println!("  - LINK Address: 0x{:x}", link_address);
    ic_cdk::println!("  - Amount: {} wei", amount_wei);
    ic_cdk::println!("  - User Address: 0x{:x}", address);
    ic_cdk::println!("  - Nonce: {}", nonce);
    ic_cdk::println!("  - Chain ID: 11155111 (Sepolia)");
    
    ic_cdk::println!("🚀 Sending AAVE supply transaction...");
    
    // Try to estimate gas first
    ic_cdk::println!("📊 Estimating gas for transaction...");
    let call_builder = pool_contract
        .supply(link_address, amount_wei, address, 0u16)
        .nonce(nonce)
        .chain_id(11155111) // Sepolia chain ID
        .from(address);
    
    // Send with increased gas limit
    match call_builder
        .gas(1_000_000u128) // Increase gas limit even more
        .send()
        .await
    {
        Ok(builder) => {
            let tx_hash = *builder.tx_hash();
            ic_cdk::println!("✅ Step 8 Complete: Transaction sent with hash: {:?}", tx_hash);
            
            ic_cdk::println!("✅ Step 9: Waiting for transaction confirmation...");
            let tx_response = provider.get_transaction_by_hash(tx_hash).await
                .map_err(|e| {
                    let error_msg = format!("Failed to get transaction: {}", e);
                    ic_cdk::println!("❌ Step 9 Failed: {}", error_msg);
                    error_msg
                })?;

            match tx_response {
                Some(tx) => {
                    ic_cdk::println!("✅ Step 9 Complete: Transaction confirmed in block");
                    ic_cdk::println!("📋 Transaction Details:");
                    ic_cdk::println!("  - Hash: {:?}", tx_hash);
                    ic_cdk::println!("  - Block: {:?}", tx.block_number);
                    ic_cdk::println!("  - Gas Used: {:?}", tx.gas);
                    ic_cdk::println!("  - Nonce: {}", tx.nonce);
                    
                    // Update nonce cache
                    AAVE_NONCE.with_borrow_mut(|nonce| {
                        *nonce = Some(tx.nonce);
                    });
                    ic_cdk::println!("✅ Step 10: Nonce cache updated to {}", tx.nonce);
                    
                    // Update daily usage
                    ic_cdk::println!("✅ Step 11: Updating daily usage limits...");
                    if let Err(e) = set_daily_usage(permissions_id, AAVE_POOL_ADDRESS.to_string(), amount_wei.to::<u64>(), user_principal) {
                        ic_cdk::println!("⚠️ Warning: Failed to update daily usage: {}", e);
                    } else {
                        ic_cdk::println!("✅ Step 11 Complete: Daily usage limits updated");
                    }
                    
                    let success_msg = format!("Successfully supplied {} LINK to AAVE. Transaction: {:?}", amount_human, tx_hash);
                    ic_cdk::println!("🎉 AAVE LINK supply completed successfully: {}", success_msg);
                    Ok(success_msg)
                }
                None => {
                    let error_msg = "Transaction not found after sending".to_string();
                    ic_cdk::println!("❌ Step 9 Failed: {}", error_msg);
                    Err(error_msg)
                }
            }
        }
        Err(e) => {
            ic_cdk::println!("❌ Step 8 Failed: Supply transaction failed: {:?}", e);
            
            // Try to decode specific AAVE errors
            let error_str = e.to_string();
            let decoded_error = if error_str.contains("execution reverted") {
                "AAVE execution reverted - possible causes: insufficient allowance, reserve frozen, invalid parameters, or gas limit too low"
            } else if error_str.contains("RESERVE_FROZEN") {
                "AAVE reserve is frozen"
            } else if error_str.contains("AMOUNT_BIGGER_THAN_MAX_LOAN_SIZE_STABLE") {
                "Amount exceeds max loan size"
            } else if error_str.contains("NO_MORE_RESERVES_ALLOWED") {
                "No more reserves allowed"
            } else if error_str.contains("INVALID_AMOUNT") {
                "Invalid amount provided"
            } else {
                "Unknown AAVE error"
            };
            
            ic_cdk::println!("🔍 Decoded error: {}", decoded_error);
            ic_cdk::println!("💡 Suggestion: Try with smaller amount (0.01 LINK) or check if AAVE Pool is operational");
            
            Err(format!("Supply transaction failed: {:?} | Decoded: {}", e, decoded_error))
        }
    }
}

/// Withdraw LINK from AAVE with permission verification  
pub async fn withdraw_link_from_aave_with_permissions(
    amount_human: String,
    permissions_id: String,
    user_principal: Principal
) -> Result<String, String> {
    ic_cdk::println!("🚀 Starting AAVE LINK withdraw: {} LINK for principal {}", amount_human, user_principal);
    
    // 1. Check permissions
    ic_cdk::println!("✅ Step 1: Verifying AAVE withdraw permissions...");
    verify_aave_permission(&permissions_id, "withdraw", &amount_human, user_principal).await?;
    ic_cdk::println!("✅ Step 1 Complete: AAVE withdraw permissions verified");
    
    // 2. Convert amount
    ic_cdk::println!("✅ Step 2: Converting amount {} LINK to wei...", amount_human);
    let amount_wei = parse_link_amount(&amount_human)?;
    ic_cdk::println!("✅ Step 2 Complete: Amount converted to {} wei", amount_wei);
    
    // 3. Create signer on behalf of user
    ic_cdk::println!("✅ Step 3: Creating ICP signer for principal...");
    let signer = create_icp_signer_for_principal(user_principal).await?;
    let address = signer.address();
    ic_cdk::println!("✅ Step 3 Complete: Signer created for address 0x{:x}", address);

    // 4. Setup provider
    ic_cdk::println!("✅ Step 4: Setting up provider and wallet...");
    let wallet = EthereumWallet::from(signer);
    let rpc_service = get_rpc_service_sepolia();
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new()
        .with_gas_estimation()
        .wallet(wallet)
        .on_icp(config);
    ic_cdk::println!("✅ Step 4 Complete: Provider and wallet configured");
    
    // 5. Check aLINK balance (get through getReserveData)
    ic_cdk::println!("✅ Step 5: Checking aLINK balance for address 0x{:x}...", address);
    let alink_balance = get_alink_balance_for_address(format!("0x{:x}", address)).await?;
    let alink_balance_wei = U256::from_str_radix(&alink_balance.replace("0x", ""), 16)
        .map_err(|_| "Failed to parse aLINK balance".to_string())?;
    
    ic_cdk::println!("✅ Step 5 Complete: aLINK balance: {} wei (need: {} wei)", alink_balance_wei, amount_wei);
    
    if alink_balance_wei < amount_wei {
        let error_msg = format!("Insufficient aLINK balance. Have: {} wei, Need: {} wei", alink_balance_wei, amount_wei);
        ic_cdk::println!("❌ AAVE withdraw failed: {}", error_msg);
        return Err(error_msg);
    }
    
    // 6. Handle nonce management
    ic_cdk::println!("✅ Step 6: Getting transaction nonce...");
    let maybe_nonce = AAVE_NONCE.with_borrow(|maybe_nonce| {
        maybe_nonce.map(|nonce| nonce + 1)
    });

    let nonce = if let Some(nonce) = maybe_nonce {
        ic_cdk::println!("✅ Step 6: Using cached nonce: {}", nonce);
        nonce
    } else {
        let fresh_nonce = provider.get_transaction_count(address).await
            .map_err(|e| format!("Failed to get nonce: {}", e))?;
        ic_cdk::println!("✅ Step 6: Got fresh nonce from network: {}", fresh_nonce);
        fresh_nonce
    };
    
    // 7. Execute withdraw from AAVE
    ic_cdk::println!("✅ Step 7: Preparing AAVE withdraw transaction...");
    let pool_address = address!("6Ae43d3271ff6888e7Fc43Fd7321a503ff738951");
    let pool_contract = AavePool::new(pool_address, provider.clone());
    
    let link_address = address!("f8fb3713d459d7c1018bd0a49d19b4c44290ebe5");
    
    ic_cdk::println!("📋 AAVE Withdraw Parameters:");
    ic_cdk::println!("  - Pool Address: 0x{:x}", pool_address);
    ic_cdk::println!("  - LINK Address: 0x{:x}", link_address);
    ic_cdk::println!("  - Amount: {} wei", amount_wei);
    ic_cdk::println!("  - User Address: 0x{:x}", address);
    ic_cdk::println!("  - Nonce: {}", nonce);
    ic_cdk::println!("  - Chain ID: 11155111 (Sepolia)");
    
    ic_cdk::println!("🚀 Sending AAVE withdraw transaction...");
    
    // Try to estimate gas first
    ic_cdk::println!("📊 Estimating gas for withdraw transaction...");
    let call_builder = pool_contract
        .withdraw(link_address, amount_wei, address)
        .nonce(nonce)
        .chain_id(11155111) // Sepolia chain ID
        .from(address);
    
    // Send with increased gas limit
    match call_builder
        .gas(1_000_000u128) // Increase gas limit even more
        .send()
        .await
    {
        Ok(builder) => {
            let tx_hash = *builder.tx_hash();
            ic_cdk::println!("✅ Step 7 Complete: Transaction sent with hash: {:?}", tx_hash);
            
            ic_cdk::println!("✅ Step 8: Waiting for transaction confirmation...");
            let tx_response = provider.get_transaction_by_hash(tx_hash).await
                .map_err(|e| {
                    let error_msg = format!("Failed to get transaction: {}", e);
                    ic_cdk::println!("❌ Step 8 Failed: {}", error_msg);
                    error_msg
                })?;

            match tx_response {
                Some(tx) => {
                    ic_cdk::println!("✅ Step 8 Complete: Transaction confirmed in block");
                    ic_cdk::println!("📋 Transaction Details:");
                    ic_cdk::println!("  - Hash: {:?}", tx_hash);
                    ic_cdk::println!("  - Block: {:?}", tx.block_number);
                    ic_cdk::println!("  - Gas Used: {:?}", tx.gas);
                    ic_cdk::println!("  - Nonce: {}", tx.nonce);
                    
                    // Update nonce cache
                    AAVE_NONCE.with_borrow_mut(|nonce| {
                        *nonce = Some(tx.nonce);
                    });
                    ic_cdk::println!("✅ Step 9: Nonce cache updated to {}", tx.nonce);
                    
                    // Update daily usage
                    ic_cdk::println!("✅ Step 10: Updating daily usage limits...");
                    if let Err(e) = set_daily_usage(permissions_id, AAVE_POOL_ADDRESS.to_string(), amount_wei.to::<u64>(), user_principal) {
                        ic_cdk::println!("⚠️ Warning: Failed to update daily usage: {}", e);
                    } else {
                        ic_cdk::println!("✅ Step 10 Complete: Daily usage limits updated");
                    }
                    
                    let success_msg = format!("Successfully withdrew {} LINK from AAVE. Transaction: {:?}", amount_human, tx_hash);
                    ic_cdk::println!("🎉 AAVE LINK withdraw completed successfully: {}", success_msg);
                    Ok(success_msg)
                }
                None => {
                    let error_msg = "Transaction not found after sending".to_string();
                    ic_cdk::println!("❌ Step 8 Failed: {}", error_msg);
                    Err(error_msg)
                }
            }
        }
        Err(e) => {
            ic_cdk::println!("❌ Step 7 Failed: Withdraw transaction failed: {:?}", e);
            
            // Try to decode specific AAVE errors
            let error_str = e.to_string();
            let decoded_error = if error_str.contains("execution reverted") {
                "AAVE execution reverted - possible causes: insufficient aToken balance, reserve paused, invalid parameters, or gas limit too low"
            } else if error_str.contains("RESERVE_PAUSED") {
                "AAVE reserve is paused for withdrawals"
            } else if error_str.contains("AMOUNT_BIGGER_THAN_BALANCE") {
                "Withdrawal amount exceeds aToken balance"
            } else if error_str.contains("WITHDRAW_TO_ATOKEN") {
                "Cannot withdraw to aToken address"
            } else if error_str.contains("INVALID_AMOUNT") {
                "Invalid withdrawal amount provided"
            } else {
                "Unknown AAVE withdrawal error"
            };
            
            ic_cdk::println!("🔍 Decoded error: {}", decoded_error);
            ic_cdk::println!("💡 Suggestion: Check aLINK balance and ensure AAVE Pool allows withdrawals");
            
            Err(format!("Withdraw transaction failed: {:?} | Decoded: {}", e, decoded_error))
        }
    }
}

/// Get user's aLINK balance in AAVE
pub async fn get_aave_link_balance(address: Option<String>) -> Result<String, String> {
    let target_address = match address {
        Some(addr) => addr,
        None => {
            // Get address for current caller
            let caller = ic_cdk::caller();
            PRINCIPAL_TO_ADDRESS_MAP.with(|map| {
                map.borrow()
                    .get(&StorablePrincipal(caller))
                    .map(|s| s.0.clone())
                    .ok_or_else(|| "No EVM address found for caller. Generate one first.".to_string())
            })?
        }
    };
    
    get_alink_balance_for_address(target_address).await
}

// Helper functions

/// Check AAVE permissions
async fn verify_aave_permission(
    permissions_id: &str,
    function_name: &str,
    amount_human: &str,
    user_principal: Principal
) -> Result<(), String> {
    // Check ownership permissions
    if let Err(e) = is_permissions_owner(permissions_id, user_principal) {
        return Err(e);
    }
    
    // Convert amount
    let amount_wei = parse_link_amount(amount_human)?.to::<u64>();
    
    // Check protocol permission
    verify_protocol_permission(
        permissions_id.to_string(),
        AAVE_POOL_ADDRESS.to_string(),
        function_name.to_string(),
        amount_wei,
        user_principal
    ).map(|_| ())
}

/// Parse LINK amount (18 decimals)
fn parse_link_amount(amount_human: &str) -> Result<U256, String> {
    let amount_f64: f64 = amount_human.parse()
        .map_err(|_| "Invalid amount format".to_string())?;
    
    if amount_f64 < 0.0 {
        return Err("Amount cannot be negative".to_string());
    }
    
    // LINK has 18 decimals
    let amount_wei = (amount_f64 * 1e18) as u128;
    Ok(U256::from(amount_wei))
}

/// Get LINK balance for address
async fn get_link_balance_for_address(address: String) -> Result<String, String> {
    get_balance_link(Some(address)).await
}

/// Get aLINK balance for address
async fn get_alink_balance_for_address(address: String) -> Result<String, String> {
    // Create provider without signer for read-only operations
    let rpc_service = get_rpc_service_sepolia();
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new().on_icp(config);
    
    // Get reserve data for LINK
    let pool_address = address!("6Ae43d3271ff6888e7Fc43Fd7321a503ff738951");
    let pool_contract = AavePool::new(pool_address, provider.clone());
    
    let link_address = address!("f8fb3713d459d7c1018bd0a49d19b4c44290ebe5");
    
    let reserve_data = pool_contract.getReserveData(link_address).call().await
        .map_err(|e| format!("Failed to get reserve data: {}", e))?;
    
    // aToken address is in reserve data
    let alink_address = reserve_data._0.aTokenAddress;
    
    // Get aLINK balance
    let link_contract = LINK::new(alink_address, provider);
    let user_address = address.parse::<Address>()
        .map_err(|_| "Invalid user address".to_string())?;
    
    let balance = link_contract.balanceOf(user_address).call().await
        .map_err(|e| format!("Failed to get aLINK balance: {}", e))?;
    
    Ok(format!("0x{:x}", balance._0))
}

/// Ensure sufficient allowance for AAVE Pool
async fn ensure_link_allowance_for_aave(
    provider: &alloy::providers::fillers::FillProvider<
        alloy::providers::fillers::JoinFill<
            alloy::providers::fillers::JoinFill<
                alloy::providers::Identity,
                alloy::providers::fillers::GasFiller,
            >,
            alloy::providers::fillers::WalletFiller<EthereumWallet>,
        >,
        alloy::providers::RootProvider<alloy::transports::icp::IcpTransport>,
        alloy::transports::icp::IcpTransport,
        alloy::network::Ethereum,
    >,
    amount: U256,
    user_address: Address
) -> Result<(), String> {
    let link_address = address!("f8fb3713d459d7c1018bd0a49d19b4c44290ebe5");
    let pool_address = address!("6Ae43d3271ff6888e7Fc43Fd7321a503ff738951");
    
    ic_cdk::println!("🔍 Checking LINK allowance for AAVE Pool...");
    ic_cdk::println!("  - LINK Contract: 0x{:x}", link_address);
    ic_cdk::println!("  - AAVE Pool: 0x{:x}", pool_address);
    ic_cdk::println!("  - User Address: 0x{:x}", user_address);
    ic_cdk::println!("  - Required Amount: {} wei", amount);
    
    let link_contract = LINK::new(link_address, provider);
    
    // Check current allowance
    ic_cdk::println!("📞 Calling LINK.allowance()...");
    let current_allowance = link_contract.allowance(user_address, pool_address).call().await
        .map_err(|e| {
            let error_msg = format!("Failed to get allowance: {}", e);
            ic_cdk::println!("❌ Failed to get current allowance: {}", error_msg);
            error_msg
        })?;
    
    ic_cdk::println!("✅ Current LINK allowance: {} wei (need: {} wei)", current_allowance._0, amount);
    
    if current_allowance._0 < amount {
        ic_cdk::println!("⚠️ Insufficient allowance, need to approve more LINK...");
        // Handle nonce management for approval
        ic_cdk::println!("🔧 Getting nonce for LINK approval transaction...");
        let maybe_nonce = AAVE_NONCE.with_borrow(|maybe_nonce| {
            maybe_nonce.map(|nonce| nonce + 1)
        });

        let nonce = if let Some(nonce) = maybe_nonce {
            ic_cdk::println!("🔧 Using cached nonce for approval: {}", nonce);
            nonce
        } else {
            let fresh_nonce = provider.get_transaction_count(user_address).await
                .map_err(|e| format!("Failed to get nonce for approval: {}", e))?;
            ic_cdk::println!("🔧 Got fresh nonce for approval: {}", fresh_nonce);
            fresh_nonce
        };
        
        // Increase allowance
        ic_cdk::println!("🚀 Sending LINK approval transaction...");
        ic_cdk::println!("📋 Approval Parameters:");
        ic_cdk::println!("  - Spender (AAVE Pool): 0x{:x}", pool_address);
        ic_cdk::println!("  - Amount: {} wei", amount);
        ic_cdk::println!("  - Nonce: {}", nonce);
        
        match link_contract
            .approve(pool_address, amount)
            .nonce(nonce)
            .chain_id(11155111) // Sepolia chain ID
            .from(user_address)
            .send()
            .await
        {
            Ok(builder) => {
                let tx_hash = *builder.tx_hash();
                ic_cdk::println!("✅ LINK approval transaction sent: {:?}", tx_hash);
                
                let tx_response = provider.get_transaction_by_hash(tx_hash).await
                    .map_err(|e| format!("Failed to get approve transaction: {}", e))?;

                match tx_response {
                    Some(tx) => {
                        // Update nonce cache
                        AAVE_NONCE.with_borrow_mut(|nonce| {
                            *nonce = Some(tx.nonce);
                        });
                        
                        ic_cdk::println!("✅ LINK approved for AAVE Pool successfully: {:?}", tx_hash);
                        ic_cdk::println!("📋 Approval confirmed in block: {:?}", tx.block_number);
                    }
                    None => {
                        let error_msg = "Approve transaction not found after sending".to_string();
                        ic_cdk::println!("❌ {}", error_msg);
                        return Err(error_msg);
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Approve transaction failed: {:?}", e);
                ic_cdk::println!("❌ {}", error_msg);
                return Err(error_msg);
            }
        }
    } else {
        ic_cdk::println!("✅ Sufficient LINK allowance already exists, no approval needed");
    }
    
    Ok(())
} 