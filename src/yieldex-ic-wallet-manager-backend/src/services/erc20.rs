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
use crate::{PRINCIPAL_TO_ADDRESS_MAP, StorablePrincipal, get_rpc_service_sepolia};

thread_local! {
    static ERC20_NONCE_CACHE: RefCell<Option<u64>> = const { RefCell::new(None) };
}

// Universal ERC-20 contract interface using LINK.json as base
sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    ERC20Token,
    "src/abi/LINK.json"
);

/// Universal ERC-20 token information
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub address: Address,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

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
        _ => "dfx_test_key".to_string(),
    }
}

/// Get token information (name, symbol, decimals)
pub async fn get_token_info(token_address: String) -> Result<TokenInfo, String> {
    ic_cdk::println!("🔍 Getting token info for address: {}", token_address);
    
    let token_addr = token_address.parse::<Address>()
        .map_err(|_| "Invalid token address".to_string())?;
    
    let rpc_service = get_rpc_service_sepolia();
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new().on_icp(config);
    
    let contract = ERC20Token::new(token_addr, provider);
    
    // Get token details
    let name = contract.name().call().await
        .map_err(|e| format!("Failed to get token name: {}", e))?._0;
    
    let symbol = contract.symbol().call().await
        .map_err(|e| format!("Failed to get token symbol: {}", e))?._0;
    
    let decimals = contract.decimals().call().await
        .map_err(|e| format!("Failed to get token decimals: {}", e))?._0;
    
    let token_info = TokenInfo {
        address: token_addr,
        name,
        symbol,
        decimals,
    };
    
    ic_cdk::println!("✅ Token info retrieved: {} ({}) - {} decimals", 
                    token_info.name, token_info.symbol, token_info.decimals);
    
    Ok(token_info)
}

/// Get token balance for a specific address
pub async fn get_token_balance(token_address: String, user_address: Option<String>) -> Result<String, String> {
    ic_cdk::println!("💰 Getting token balance for {}", token_address);
    
    let token_addr = token_address.parse::<Address>()
        .map_err(|_| "Invalid token address".to_string())?;
    
    let target_address = match user_address {
        Some(addr) => addr,
        None => {
            let caller = ic_cdk::caller();
            PRINCIPAL_TO_ADDRESS_MAP.with(|map| {
                map.borrow()
                    .get(&StorablePrincipal(caller))
                    .map(|s| s.0.clone())
                    .ok_or_else(|| "No EVM address found for caller".to_string())
            })?
        }
    };
    
    let user_addr = target_address.parse::<Address>()
        .map_err(|_| "Invalid user address".to_string())?;
    
    let rpc_service = get_rpc_service_sepolia();
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new().on_icp(config);
    
    let contract = ERC20Token::new(token_addr, provider);
    
    let balance = contract.balanceOf(user_addr).call().await
        .map_err(|e| format!("Failed to get token balance: {}", e))?;
    
    ic_cdk::println!("✅ Token balance: {} wei", balance._0);
    Ok(format!("0x{:x}", balance._0))
}

/// Get token allowance between owner and spender
pub async fn get_token_allowance(
    token_address: String,
    owner_address: Option<String>,
    spender_address: String
) -> Result<String, String> {
    ic_cdk::println!("🔐 Getting token allowance for {} -> {}", 
                    owner_address.as_deref().unwrap_or("caller"), spender_address);
    
    let token_addr = token_address.parse::<Address>()
        .map_err(|_| "Invalid token address".to_string())?;
    
    let owner_addr = match owner_address {
        Some(addr) => addr.parse::<Address>().map_err(|_| "Invalid owner address".to_string())?,
        None => {
            let caller = ic_cdk::caller();
            let addr_str = PRINCIPAL_TO_ADDRESS_MAP.with(|map| {
                map.borrow()
                    .get(&StorablePrincipal(caller))
                    .map(|s| s.0.clone())
                    .ok_or_else(|| "No EVM address found for caller".to_string())
            })?;
            addr_str.parse::<Address>().map_err(|_| "Invalid caller address".to_string())?
        }
    };
    
    let spender_addr = spender_address.parse::<Address>()
        .map_err(|_| "Invalid spender address".to_string())?;
    
    let rpc_service = get_rpc_service_sepolia();
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new().on_icp(config);
    
    let contract = ERC20Token::new(token_addr, provider);
    
    let allowance = contract.allowance(owner_addr, spender_addr).call().await
        .map_err(|e| format!("Failed to get allowance: {}", e))?;
    
    ic_cdk::println!("✅ Token allowance: {} wei", allowance._0);
    Ok(format!("0x{:x}", allowance._0))
}

/// Approve token spending for a spender
pub async fn approve_token(
    token_address: String,
    spender_address: String,
    amount: U256,
    user_principal: Principal
) -> Result<String, String> {
    ic_cdk::println!("✅ Approving {} wei of token {} for spender {}", 
                    amount, token_address, spender_address);
    
    let token_addr = token_address.parse::<Address>()
        .map_err(|_| "Invalid token address".to_string())?;
    
    let spender_addr = spender_address.parse::<Address>()
        .map_err(|_| "Invalid spender address".to_string())?;
    
    // Create signer for user
    let signer = create_icp_signer_for_principal(user_principal).await?;
    let user_addr = signer.address();
    
    // Setup provider
    let wallet = EthereumWallet::from(signer);
    let rpc_service = get_rpc_service_sepolia();
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new()
        .with_gas_estimation()
        .wallet(wallet)
        .on_icp(config);
    
    // Get nonce
    let maybe_nonce = ERC20_NONCE_CACHE.with_borrow(|maybe_nonce| {
        maybe_nonce.map(|nonce| nonce + 1)
    });

    let nonce = if let Some(nonce) = maybe_nonce {
        ic_cdk::println!("🔧 Using cached nonce: {}", nonce);
        nonce
    } else {
        let fresh_nonce = provider.get_transaction_count(user_addr).await
            .map_err(|e| format!("Failed to get nonce: {}", e))?;
        ic_cdk::println!("🔧 Got fresh nonce: {}", fresh_nonce);
        fresh_nonce
    };
    
    let contract = ERC20Token::new(token_addr, provider.clone());
    
    // Send approval transaction
    match contract
        .approve(spender_addr, amount)
        .nonce(nonce)
        .chain_id(11155111) // Sepolia chain ID
        .from(user_addr)
        .send()
        .await
    {
        Ok(builder) => {
            let tx_hash = *builder.tx_hash();
            ic_cdk::println!("✅ Approval transaction sent: {:?}", tx_hash);
            
            let tx_response = provider.get_transaction_by_hash(tx_hash).await
                .map_err(|e| format!("Failed to get transaction: {}", e))?;

            match tx_response {
                Some(tx) => {
                    // Update nonce cache
                    ERC20_NONCE_CACHE.with_borrow_mut(|nonce| {
                        *nonce = Some(tx.nonce);
                    });
                    
                    let success_msg = format!("Token approval successful: {:?}", tx_hash);
                    ic_cdk::println!("🎉 {}", success_msg);
                    Ok(success_msg)
                }
                None => {
                    let error_msg = "Approval transaction not found after sending".to_string();
                    ic_cdk::println!("❌ {}", error_msg);
                    Err(error_msg)
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Approval transaction failed: {:?}", e);
            ic_cdk::println!("❌ {}", error_msg);
            Err(error_msg)
        }
    }
}

/// Transfer tokens to a specified address
pub async fn transfer_token(
    token_address: String,
    to_address: String,
    amount: U256,
    user_principal: Principal
) -> Result<String, String> {
    ic_cdk::println!("💸 Transferring {} wei of token {} to {}", 
                    amount, token_address, to_address);
    
    let token_addr = token_address.parse::<Address>()
        .map_err(|_| "Invalid token address".to_string())?;
    
    let to_addr = to_address.parse::<Address>()
        .map_err(|_| "Invalid recipient address".to_string())?;
    
    // Create signer for user
    let signer = create_icp_signer_for_principal(user_principal).await?;
    let user_addr = signer.address();
    
    // Setup provider
    let wallet = EthereumWallet::from(signer);
    let rpc_service = get_rpc_service_sepolia();
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new()
        .with_gas_estimation()
        .wallet(wallet)
        .on_icp(config);
    
    let contract = ERC20Token::new(token_addr, provider.clone());
    
    // Check balance
    let balance = contract.balanceOf(user_addr).call().await
        .map_err(|e| format!("Failed to get balance: {}", e))?;
    
    if balance._0 < amount {
        let error_msg = format!("Insufficient token balance. Have: {}, Need: {}", balance._0, amount);
        ic_cdk::println!("❌ {}", error_msg);
        return Err(error_msg);
    }
    
    // Get nonce
    let maybe_nonce = ERC20_NONCE_CACHE.with_borrow(|maybe_nonce| {
        maybe_nonce.map(|nonce| nonce + 1)
    });

    let nonce = if let Some(nonce) = maybe_nonce {
        ic_cdk::println!("🔧 Using cached nonce: {}", nonce);
        nonce
    } else {
        let fresh_nonce = provider.get_transaction_count(user_addr).await
            .map_err(|e| format!("Failed to get nonce: {}", e))?;
        ic_cdk::println!("🔧 Got fresh nonce: {}", fresh_nonce);
        fresh_nonce
    };
    
    // Send transfer transaction
    match contract
        .transfer(to_addr, amount)
        .nonce(nonce)
        .chain_id(11155111) // Sepolia chain ID
        .from(user_addr)
        .send()
        .await
    {
        Ok(builder) => {
            let tx_hash = *builder.tx_hash();
            ic_cdk::println!("✅ Transfer transaction sent: {:?}", tx_hash);
            
            let tx_response = provider.get_transaction_by_hash(tx_hash).await
                .map_err(|e| format!("Failed to get transaction: {}", e))?;

            match tx_response {
                Some(tx) => {
                    // Update nonce cache
                    ERC20_NONCE_CACHE.with_borrow_mut(|nonce| {
                        *nonce = Some(tx.nonce);
                    });
                    
                    let success_msg = format!("Token transfer successful: {:?}", tx_hash);
                    ic_cdk::println!("🎉 {}", success_msg);
                    Ok(success_msg)
                }
                None => {
                    let error_msg = "Transfer transaction not found after sending".to_string();
                    ic_cdk::println!("❌ {}", error_msg);
                    Err(error_msg)
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Transfer transaction failed: {:?}", e);
            ic_cdk::println!("❌ {}", error_msg);
            Err(error_msg)
        }
    }
}

/// Convert human-readable amount to Wei format based on token decimals
pub async fn parse_token_amount(token_address: String, amount_human: String) -> Result<U256, String> {
    ic_cdk::println!("🧮 Converting {} of token {} to wei", amount_human, token_address);
    
    let amount_f64: f64 = amount_human.parse()
        .map_err(|_| "Invalid amount format".to_string())?;
    
    if amount_f64 < 0.0 {
        return Err("Amount cannot be negative".to_string());
    }
    
    // Get token decimals
    let token_info = get_token_info(token_address).await?;
    let decimals = token_info.decimals;
    
    // Convert to Wei (multiply by 10^decimals)
    let multiplier = 10_f64.powi(decimals as i32);
    let amount_wei = (amount_f64 * multiplier) as u128;
    let amount_u256 = U256::from(amount_wei);
    
    ic_cdk::println!("✅ Converted {} {} to {} wei (decimals: {})", 
                    amount_human, token_info.symbol, amount_u256, decimals);
    
    Ok(amount_u256)
}

/// Ensure sufficient allowance for a spender
pub async fn ensure_token_allowance(
    token_address: String,
    spender_address: String, 
    required_amount: U256,
    user_principal: Principal
) -> Result<(), String> {
    ic_cdk::println!("🔍 Ensuring token allowance for {} -> {}", token_address, spender_address);
    
    // Get current allowance
    let current_allowance_str = get_token_allowance(
        token_address.clone(),
        None, // Use caller's address
        spender_address.clone()
    ).await?;
    
    let current_allowance = U256::from_str_radix(&current_allowance_str.replace("0x", ""), 16)
        .map_err(|_| "Failed to parse current allowance".to_string())?;
    
    ic_cdk::println!("📊 Current allowance: {} wei, Required: {} wei", 
                    current_allowance, required_amount);
    
    if current_allowance < required_amount {
        ic_cdk::println!("⚠️ Insufficient allowance, approving more tokens...");
        
        // Approve the required amount
        approve_token(
            token_address,
            spender_address,
            required_amount,
            user_principal
        ).await?;
        
        ic_cdk::println!("✅ Token allowance approved successfully");
    } else {
        ic_cdk::println!("✅ Sufficient allowance already exists");
    }
    
    Ok(())
} 