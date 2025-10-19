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
use crate::{PRINCIPAL_TO_ADDRESS_MAP, StorablePrincipal};
use crate::services::rpc_service::{get_rpc_service_by_chain_id, SEPOLIA_CHAIN_ID, BASE_CHAIN_ID, OPTIMISM_CHAIN_ID, ARBITRUM_CHAIN_ID};
use crate::services::permissions::{is_permissions_owner, verify_protocol_permission, set_daily_usage};
use crate::services::get_balance_link::get_balance_link;

thread_local! {
    static AAVE_NONCE: RefCell<Option<u64>> = const { RefCell::new(None) };
}

// AAVE V3 chain configuration
#[derive(Clone)]
struct AaveChainConfig {
    pool_address: Address,
    chain_id: u64,
}

/// Get AAVE configuration for a specific chain
fn get_aave_config(chain_id: u64) -> Result<AaveChainConfig, String> {
    match chain_id {
        SEPOLIA_CHAIN_ID => Ok(AaveChainConfig {
            pool_address: address!("6Ae43d3271ff6888e7Fc43Fd7321a503ff738951"), // AAVE V3 Pool on Sepolia
            chain_id: SEPOLIA_CHAIN_ID,
        }),
        ARBITRUM_CHAIN_ID => Ok(AaveChainConfig {
            pool_address: address!("794a61358D6845594F94dc1DB02A252b5b4814aD"), // AAVE V3 Pool on Arbitrum
            chain_id: ARBITRUM_CHAIN_ID,
        }),
        BASE_CHAIN_ID => Ok(AaveChainConfig {
            pool_address: address!("794a61358D6845594F94dc1DB02A252b5b4814aD"), // AAVE V3 Pool on Base
            chain_id: BASE_CHAIN_ID,
        }),
        OPTIMISM_CHAIN_ID => Ok(AaveChainConfig {
            pool_address: address!("794a61358D6845594F94dc1DB02A252b5b4814aD"), // AAVE V3 Pool on Optimism
            chain_id: OPTIMISM_CHAIN_ID,
        }),
        _ => Err(format!("AAVE V3 not supported on chain_id: {}", chain_id))
    }
}

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

// Codegen from ABI file to interact with USDC contract
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

/// Supply any token to AAVE with permission verification
pub async fn supply_to_aave_with_permissions(
    token_address: Address,
    token_symbol: String,
    amount_human: String,
    permissions_id: String,
    user_principal: Principal,
    chain_id: u64
) -> Result<String, String> {
    ic_cdk::println!("🚀 Starting AAVE {} supply: {} {} for principal {} on chain {}", token_symbol, amount_human, token_symbol, user_principal, chain_id);
    
    // 1. Get chain configuration
    ic_cdk::println!("✅ Step 2: Getting AAVE configuration for chain {}...", chain_id);
    let aave_config = get_aave_config(chain_id)?;
    ic_cdk::println!("✅ Step 2 Complete: AAVE config loaded for chain {}", chain_id);
    
    // 2. Check permissions
    ic_cdk::println!("✅ Step 2: Verifying AAVE permissions...");
    verify_aave_permission(&permissions_id, "supply", &amount_human, &token_symbol, user_principal, chain_id).await?;
    ic_cdk::println!("✅ Step 2 Complete: AAVE permissions verified");
    
    // 3. Convert amount
    ic_cdk::println!("✅ Step 3: Converting amount {} {} to wei...", amount_human, token_symbol);
    let amount_wei = parse_token_amount(&amount_human, &token_symbol)?;
    ic_cdk::println!("✅ Step 3 Complete: Amount converted to {} wei", amount_wei);
    
    // 3. Create signer on behalf of user
    ic_cdk::println!("✅ Step 3: Creating ICP signer for principal...");
    let signer = create_icp_signer_for_principal(user_principal).await?;
    let address = signer.address();
    ic_cdk::println!("✅ Step 3 Complete: Signer created for address 0x{:x}", address);

    // 5. Setup provider
    ic_cdk::println!("✅ Step 5: Setting up provider and wallet...");
    let wallet = EthereumWallet::from(signer);
    let rpc_service = get_rpc_service_by_chain_id(chain_id)?;
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new()
        .with_gas_estimation()
        .wallet(wallet)
        .on_icp(config);
    ic_cdk::println!("✅ Step 5 Complete: Provider and wallet configured");
    
    // 6. Check token balance
    ic_cdk::println!("✅ Step 6: Checking {} balance for address 0x{:x}...", token_symbol, address);
    let token_balance = get_token_balance_for_address(format!("0x{:x}", address), token_address, chain_id).await?;
    let token_balance_wei = U256::from_str_radix(&token_balance.replace("0x", ""), 16)
        .map_err(|_| format!("Failed to parse {} balance", token_symbol))?;
    
    ic_cdk::println!("✅ Step 6 Complete: {} balance: {} wei (need: {} wei)", token_symbol, token_balance_wei, amount_wei);
    
    if token_balance_wei < amount_wei {
        let error_msg = format!("Insufficient {} balance. Have: {} wei, Need: {} wei", token_symbol, token_balance_wei, amount_wei);
        ic_cdk::println!("❌ AAVE {} supply failed: {}", token_symbol, error_msg);
        return Err(error_msg);
    }
    
    // 7. Check/set allowance for AAVE Pool
    ic_cdk::println!("✅ Step 7: Ensuring {} allowance for AAVE Pool...", token_symbol);
    ensure_token_allowance_for_aave(&provider, token_address, amount_wei, address, &aave_config).await?;
    ic_cdk::println!("✅ Step 7 Complete: {} allowance confirmed for AAVE Pool", token_symbol);
    
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
    let pool_contract = AavePool::new(aave_config.pool_address, provider.clone());
    
    ic_cdk::println!("📋 AAVE Supply Parameters:");
    ic_cdk::println!("  - Pool Address: 0x{:x}", aave_config.pool_address);
    ic_cdk::println!("  - Token Address: 0x{:x}", token_address);
    ic_cdk::println!("  - Amount: {} wei", amount_wei);
    ic_cdk::println!("  - User Address: 0x{:x}", address);
    ic_cdk::println!("  - Nonce: {}", nonce);
    ic_cdk::println!("  - Chain ID: {}", chain_id);
    
    ic_cdk::println!("🚀 Sending AAVE supply transaction...");
    
    // Try to estimate gas first
    ic_cdk::println!("📊 Estimating gas for transaction...");
    let call_builder = pool_contract
        .supply(token_address, amount_wei, address, 0u16)
        .nonce(nonce)
        .chain_id(chain_id)
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
                    if let Err(e) = set_daily_usage(permissions_id, aave_config.pool_address.to_string(), amount_wei.to::<u64>(), user_principal) {
                        ic_cdk::println!("⚠️ Warning: Failed to update daily usage: {}", e);
                    } else {
                        ic_cdk::println!("✅ Step 11 Complete: Daily usage limits updated");
                    }
                    
                    let success_msg = format!("Successfully supplied {} {} to AAVE. Transaction: {:?}", amount_human, token_symbol, tx_hash);
                    ic_cdk::println!("🎉 AAVE {} supply completed successfully: {}", token_symbol, success_msg);
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

/// Withdraw any token from AAVE with permission verification
pub async fn withdraw_from_aave_with_permissions(
    token_address: Address,
    token_symbol: String,
    amount_human: String,
    permissions_id: String,
    user_principal: Principal,
    chain_id: u64
) -> Result<String, String> {
    ic_cdk::println!("🚀 Starting AAVE {} withdraw: {} {} for principal {} on chain {}", token_symbol, amount_human, token_symbol, user_principal, chain_id);
    
    // 1. Get chain configuration
    ic_cdk::println!("✅ Step 1: Getting AAVE configuration for chain {}...", chain_id);
    let aave_config = get_aave_config(chain_id)?;
    ic_cdk::println!("✅ Step 1 Complete: AAVE config loaded for chain {}", chain_id);
    
    // 2. Check permissions
    ic_cdk::println!("✅ Step 2: Verifying AAVE withdraw permissions...");
    verify_aave_permission(&permissions_id, "withdraw", &amount_human, &token_symbol, user_principal, chain_id).await?;
    ic_cdk::println!("✅ Step 2 Complete: AAVE withdraw permissions verified");
    
    // 3. Convert amount
    ic_cdk::println!("✅ Step 3: Converting amount {} {} to wei...", amount_human, token_symbol);
    let amount_wei = parse_token_amount(&amount_human, &token_symbol)?;
    ic_cdk::println!("✅ Step 3 Complete: Amount converted to {} wei", amount_wei);
    
    // 4. Create signer on behalf of user
    ic_cdk::println!("✅ Step 4: Creating ICP signer for principal...");
    let signer = create_icp_signer_for_principal(user_principal).await?;
    let address = signer.address();
    ic_cdk::println!("✅ Step 4 Complete: Signer created for address 0x{:x}", address);

    // 5. Setup provider
    ic_cdk::println!("✅ Step 5: Setting up provider and wallet...");
    let wallet = EthereumWallet::from(signer);
    let rpc_service = get_rpc_service_by_chain_id(chain_id)?;
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new()
        .with_gas_estimation()
        .wallet(wallet)
        .on_icp(config);
    ic_cdk::println!("✅ Step 5 Complete: Provider and wallet configured");
    
    // 6. Check aToken balance
    ic_cdk::println!("✅ Step 6: Checking a{} balance for address 0x{:x}...", token_symbol, address);
    let atoken_balance = get_atoken_balance_for_address(format!("0x{:x}", address), token_address, chain_id).await?;
    let atoken_balance_wei = U256::from_str_radix(&atoken_balance.replace("0x", ""), 16)
        .map_err(|_| format!("Failed to parse a{} balance", token_symbol))?;
    
    ic_cdk::println!("✅ Step 6 Complete: a{} balance: {} wei (need: {} wei)", token_symbol, atoken_balance_wei, amount_wei);
    
    if atoken_balance_wei < amount_wei {
        let error_msg = format!("Insufficient a{} balance. Have: {} wei, Need: {} wei", token_symbol, atoken_balance_wei, amount_wei);
        ic_cdk::println!("❌ AAVE {} withdraw failed: {}", token_symbol, error_msg);
        return Err(error_msg);
    }
    
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
    
    // 8. Execute withdraw from AAVE
    ic_cdk::println!("✅ Step 8: Preparing AAVE withdraw transaction...");
    let pool_address = aave_config.pool_address;
    let pool_contract = AavePool::new(pool_address, provider.clone());
    
    ic_cdk::println!("📋 AAVE Withdraw Parameters:");
    ic_cdk::println!("  - Pool Address: 0x{:x}", pool_address);
    ic_cdk::println!("  - {} Address: 0x{:x}", token_symbol, token_address);
    ic_cdk::println!("  - Amount: {} wei", amount_wei);
    ic_cdk::println!("  - User Address: 0x{:x}", address);
    ic_cdk::println!("  - Nonce: {}", nonce);
    ic_cdk::println!("  - Chain ID: {}", chain_id);
    
    ic_cdk::println!("🚀 Sending AAVE withdraw transaction...");
    
    // Try to estimate gas first
    ic_cdk::println!("📊 Estimating gas for withdraw transaction...");
    let call_builder = pool_contract
        .withdraw(token_address, amount_wei, address)
        .nonce(nonce)
        .chain_id(chain_id)
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
                    if let Err(e) = set_daily_usage(permissions_id, format!("0x{:x}", aave_config.pool_address), amount_wei.to::<u64>(), user_principal) {
                        ic_cdk::println!("⚠️ Warning: Failed to update daily usage: {}", e);
                    } else {
                        ic_cdk::println!("✅ Step 11 Complete: Daily usage limits updated");
                    }
                    
                    let success_msg = format!("Successfully withdrew {} {} from AAVE. Transaction: {:?}", amount_human, token_symbol, tx_hash);
                    ic_cdk::println!("🎉 AAVE {} withdraw completed successfully: {}", token_symbol, success_msg);
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
            ic_cdk::println!("❌ Step 8 Failed: Withdraw transaction failed: {:?}", e);
            
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
            ic_cdk::println!("💡 Suggestion: Check a{} balance and ensure AAVE Pool allows withdrawals", token_symbol);
            
            Err(format!("Withdraw transaction failed: {:?} | Decoded: {}", e, decoded_error))
        }
    }
}

/// Legacy wrapper function - Withdraw LINK from AAVE with permission verification  
pub async fn withdraw_link_from_aave_with_permissions(
    amount_human: String,
    permissions_id: String,
    user_principal: Principal
) -> Result<String, String> {
    let link_address = address!("f8fb3713d459d7c1018bd0a49d19b4c44290ebe5"); // LINK on Sepolia
    withdraw_from_aave_with_permissions(
        link_address,
        "LINK".to_string(),
        amount_human,
        permissions_id,
        user_principal,
        SEPOLIA_CHAIN_ID
    ).await
}

/// Get user's aToken balance in AAVE for any token on any supported chain
pub async fn get_aave_balance(token_address: Address, address: Option<String>, chain_id: u64) -> Result<String, String> {
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
    
    get_atoken_balance_for_address(target_address, token_address, chain_id).await
}

/// Legacy function - Get user's aLINK balance in AAVE on Sepolia
pub async fn get_aave_link_balance(address: Option<String>) -> Result<String, String> {
    let link_address = address!("f8fb3713d459d7c1018bd0a49d19b4c44290ebe5"); // LINK on Sepolia
    get_aave_balance(link_address, address, SEPOLIA_CHAIN_ID).await
}

// Helper functions

/// Check AAVE permissions for multi-chain support
async fn verify_aave_permission(
    permissions_id: &str,
    function_name: &str,
    amount_human: &str,
    token_symbol: &str,
    user_principal: Principal,
    chain_id: u64
) -> Result<(), String> {
    // Check ownership permissions
    if let Err(e) = is_permissions_owner(permissions_id, user_principal) {
        return Err(e);
    }

    // Get AAVE config for the chain
    let aave_config = get_aave_config(chain_id)?;

    // Convert amount with correct token decimals
    let amount_wei = parse_token_amount(amount_human, token_symbol)?.to::<u64>();

    // Check protocol permission
    verify_protocol_permission(
        permissions_id.to_string(),
        format!("{:x}", aave_config.pool_address),
        function_name.to_string(),
        amount_wei,
        user_principal
    ).map(|_| ())
}

/// Parse token amount with support for different decimals
fn parse_token_amount(amount_human: &str, token_symbol: &str) -> Result<U256, String> {
    let amount_f64: f64 = amount_human.parse()
        .map_err(|_| "Invalid amount format".to_string())?;
    
    if amount_f64 < 0.0 {
        return Err("Amount cannot be negative".to_string());
    }
    
    let amount_wei = match token_symbol {
        "LINK" | "WETH" | "ETH" => (amount_f64 * 1e18) as u128, // 18 decimals
        "USDC" | "USDT" => (amount_f64 * 1e6) as u128,         // 6 decimals
        _ => return Err(format!("Unsupported token: {}", token_symbol))
    };
    Ok(U256::from(amount_wei))
}

/// Legacy function - Parse LINK amount (18 decimals) for backward compatibility
fn parse_link_amount(amount_human: &str) -> Result<U256, String> {
    parse_token_amount(amount_human, "LINK")
}

/// Get token balance for address on specific chain
async fn get_token_balance_for_address(address: String, token_address: Address, chain_id: u64) -> Result<String, String> {
    // Create provider without signer for read-only operations
    let rpc_service = get_rpc_service_by_chain_id(chain_id)?;
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new().on_icp(config);
    
    let token_contract = LINK::new(token_address, provider);
    let user_address = address.parse::<Address>()
        .map_err(|_| "Invalid user address".to_string())?;
    
    let balance = token_contract.balanceOf(user_address).call().await
        .map_err(|e| format!("Failed to get token balance: {}", e))?;
    
    Ok(format!("0x{:x}", balance._0))
}

/// Legacy function - Get LINK balance for address
async fn get_link_balance_for_address(address: String) -> Result<String, String> {
    get_balance_link(Some(address)).await
}

/// Get aToken balance for address on specific chain
async fn get_atoken_balance_for_address(address: String, token_address: Address, chain_id: u64) -> Result<String, String> {
    // Create provider without signer for read-only operations
    let rpc_service = get_rpc_service_by_chain_id(chain_id)?;
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new().on_icp(config);
    
    // Get AAVE config for the chain
    let aave_config = get_aave_config(chain_id)?;
    let pool_contract = AavePool::new(aave_config.pool_address, provider.clone());
    
    let reserve_data = pool_contract.getReserveData(token_address).call().await
        .map_err(|e| format!("Failed to get reserve data: {}", e))?;
    
    // aToken address is in reserve data
    let atoken_address = reserve_data._0.aTokenAddress;
    
    // Get aToken balance
    let token_contract = LINK::new(atoken_address, provider);
    let user_address = address.parse::<Address>()
        .map_err(|_| "Invalid user address".to_string())?;
    
    let balance = token_contract.balanceOf(user_address).call().await
        .map_err(|e| format!("Failed to get aToken balance: {}", e))?;
    
    Ok(format!("0x{:x}", balance._0))
}

/// Legacy function - Get aLINK balance for address
async fn get_alink_balance_for_address(address: String) -> Result<String, String> {
    let link_address = address!("f8fb3713d459d7c1018bd0a49d19b4c44290ebe5"); // LINK on Sepolia
    get_atoken_balance_for_address(address, link_address, SEPOLIA_CHAIN_ID).await
}

/// Ensure sufficient token allowance for AAVE Pool
async fn ensure_token_allowance_for_aave(
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
    token_address: Address,
    amount: U256,
    user_address: Address,
    aave_config: &AaveChainConfig
) -> Result<(), String> {
    let pool_address = aave_config.pool_address;
    
    ic_cdk::println!("🔍 Checking token allowance for AAVE Pool...");
    ic_cdk::println!("  - Token Contract: 0x{:x}", token_address);
    ic_cdk::println!("  - AAVE Pool: 0x{:x}", pool_address);
    ic_cdk::println!("  - User Address: 0x{:x}", user_address);
    ic_cdk::println!("  - Required Amount: {} wei", amount);
    
    let token_contract = LINK::new(token_address, provider);
    
    // Check current allowance
    ic_cdk::println!("📞 Calling token.allowance()...");
    let current_allowance = token_contract.allowance(user_address, pool_address).call().await
        .map_err(|e| {
            let error_msg = format!("Failed to get allowance: {}", e);
            ic_cdk::println!("❌ Failed to get current allowance: {}", error_msg);
            error_msg
        })?;
    
    ic_cdk::println!("✅ Current token allowance: {} wei (need: {} wei)", current_allowance._0, amount);
    
    if current_allowance._0 < amount {
        ic_cdk::println!("⚠️ Insufficient allowance, need to approve more tokens...");
        // Always get fresh nonce for approval to avoid conflicts with other transactions
        ic_cdk::println!("🔧 Getting fresh nonce for token approval transaction...");
        let nonce = provider.get_transaction_count(user_address).await
            .map_err(|e| format!("Failed to get nonce for approval: {}", e))?;
        ic_cdk::println!("🔧 Got fresh nonce for approval: {}", nonce);
        
        // Increase allowance
        ic_cdk::println!("🚀 Sending token approval transaction...");
        ic_cdk::println!("📋 Approval Parameters:");
        ic_cdk::println!("  - Spender (AAVE Pool): 0x{:x}", pool_address);
        ic_cdk::println!("  - Amount: {} wei", amount);
        ic_cdk::println!("  - Nonce: {}", nonce);
        
        match token_contract
            .approve(pool_address, amount)
            .nonce(nonce)
            .chain_id(aave_config.chain_id)
            .from(user_address)
            .send()
            .await
        {
            Ok(builder) => {
                let tx_hash = *builder.tx_hash();
                ic_cdk::println!("✅ Token approval transaction sent: {:?}", tx_hash);
                
                let tx_response = provider.get_transaction_by_hash(tx_hash).await
                    .map_err(|e| format!("Failed to get approve transaction: {}", e))?;

                match tx_response {
                    Some(tx) => {
                        // Update nonce cache
                        AAVE_NONCE.with_borrow_mut(|nonce| {
                            *nonce = Some(tx.nonce);
                        });
                        
                        ic_cdk::println!("✅ Token approved for AAVE Pool successfully: {:?}", tx_hash);
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
        ic_cdk::println!("✅ Sufficient token allowance already exists, no approval needed");
    }
    
    Ok(())
}

/// Legacy wrapper function for backward compatibility - Supply LINK to AAVE on Sepolia
pub async fn supply_link_to_aave_with_permissions(
    amount_human: String,
    permissions_id: String,
    user_principal: Principal
) -> Result<String, String> {
    let link_address = address!("f8fb3713d459d7c1018bd0a49d19b4c44290ebe5"); // LINK on Sepolia
    supply_to_aave_with_permissions(
        link_address,
        "LINK".to_string(),
        amount_human,
        permissions_id,
        user_principal,
        SEPOLIA_CHAIN_ID
    ).await
}

/// Legacy wrapper function for backward compatibility - Withdraw LINK from AAVE on Sepolia  
pub async fn withdraw_link_from_aave_with_permissions_legacy(
    amount_human: String,
    permissions_id: String,
    user_principal: Principal
) -> Result<String, String> {
    let link_address = address!("f8fb3713d459d7c1018bd0a49d19b4c44290ebe5"); // LINK on Sepolia
    withdraw_from_aave_with_permissions(
        link_address,
        "LINK".to_string(),
        amount_human,
        permissions_id,
        user_principal,
        SEPOLIA_CHAIN_ID
    ).await
}
