use std::cell::RefCell;

use alloy::{
    network::EthereumWallet,
    primitives::{address, Address, U256},
    providers::{Provider, ProviderBuilder},
    signers::Signer,
    sol,
    transports::icp::IcpConfig,
};

use crate::{create_icp_signer, get_rpc_service_sepolia};

thread_local! {
    static NONCE: RefCell<Option<u64>> = const { RefCell::new(None) };
}

// Codegen from ABI file to interact with the USDC contract.
sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    USDC,
    "src/abi/USDC.json"
);

/// Approve a spender to spend USDC tokens on behalf of the caller.
/// 
/// This function allows another address (spender) to transfer up to a specified
/// amount of USDC tokens from the caller's account. This is commonly used for:
/// - DEX trading (approve DEX to spend your tokens)
/// - Staking contracts
/// - DeFi protocols
/// 
/// The following RPC calls are made:
/// - `eth_getTransactionCount`: To determine the next nonce
/// - `eth_estimateGas`: To determine the gas limit  
/// - `eth_sendRawTransaction`: The transaction
/// - `eth_getTransactionByHash`: To confirm success
#[ic_cdk::update]
pub async fn approve_usdc(spender_address: String, amount: String) -> Result<String, String> {
    // Parse the spender address
    let spender_address = spender_address.parse::<Address>()
        .map_err(|e| format!("Invalid spender address: {}", e))?;
    
    // Parse the amount (USDC has 6 decimals)
    let amount = amount.parse::<U256>()
        .map_err(|e| format!("Invalid amount: {}", e))?;

    // Setup signer
    let signer = create_icp_signer().await?;
    let address = signer.address();

    // Setup provider
    let wallet = EthereumWallet::from(signer);
    let rpc_service = get_rpc_service_sepolia();
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new()
        .with_gas_estimation()
        .wallet(wallet)
        .on_icp(config);

    // Handle nonce management
    let maybe_nonce = NONCE.with_borrow(|maybe_nonce| {
        maybe_nonce.map(|nonce| nonce + 1)
    });

    let nonce = if let Some(nonce) = maybe_nonce {
        nonce
    } else {
        provider.get_transaction_count(address).await
            .map_err(|e| format!("Failed to get nonce: {}", e))?
    };

    // Create USDC contract instance (Sepolia USDC address)
    let contract = USDC::new(
        address!("1c7d4b196cb0c7b01d743fbc6116a902379c7238"),
        provider.clone(),
    );

    // Check current allowance
    let current_allowance = contract.allowance(address, spender_address).call().await
        .map_err(|e| format!("Failed to get current allowance: {}", e))?;

    ic_cdk::println!("Current allowance for {}: {}", spender_address, current_allowance._0);

    // Execute the approve transaction
    match contract
        .approve(spender_address, amount)
        .nonce(nonce)
        .chain_id(11155111) // Sepolia chain ID
        .from(address)
        .send()
        .await
    {
        Ok(builder) => {
            let tx_hash = *builder.tx_hash();
            let tx_response = provider.get_transaction_by_hash(tx_hash).await
                .map_err(|e| format!("Failed to get transaction: {}", e))?;

            match tx_response {
                Some(tx) => {
                    // Update nonce cache
                    NONCE.with_borrow_mut(|nonce| {
                        *nonce = Some(tx.nonce);
                    });
                    
                    // Log the approval
                    ic_cdk::println!(
                        "USDC approval successful: {} approved {} USDC (raw: {}) for spender {}",
                        address, 
                        amount.to_string().parse::<f64>().unwrap_or(0.0) / 1_000_000.0, // Convert to human readable
                        amount,
                        spender_address
                    );

                    Ok(format!(
                        "USDC approval successful! Tx: {:?}. Spender {} can now spend up to {} USDC", 
                        tx_hash,
                        spender_address,
                        amount.to_string().parse::<f64>().unwrap_or(0.0) / 1_000_000.0
                    ))
                }
                None => Err("Transaction not found after sending".to_string()),
            }
        }
        Err(e) => Err(format!("Approve transaction failed: {:?}", e)),
    }
}

/// Approve USDC spending with human-readable amount.
/// 
/// This function accepts amounts like "100.50" (meaning 100.50 USDC) and automatically 
/// converts them to the proper 6-decimal format required by USDC contracts.
#[ic_cdk::update]
pub async fn approve_usdc_human(spender_address: String, amount_human: String) -> Result<String, String> {
    // Parse human-readable amount (e.g., "100.50" USDC)
    let amount_f64: f64 = amount_human.parse()
        .map_err(|e| format!("Invalid amount format: {}", e))?;
    
    if amount_f64 < 0.0 {
        return Err("Amount cannot be negative".to_string());
    }
    
    // Convert to USDC units (multiply by 10^6 for USDC's 6 decimals)
    let amount_usdc = (amount_f64 * 1_000_000.0) as u128;
    let amount_u256 = U256::from(amount_usdc);
    
    // Use the main approve function
    approve_usdc(spender_address, amount_u256.to_string()).await
}

/// Get current allowance for a spender
#[ic_cdk::update]
pub async fn get_usdc_allowance(owner_address: Option<String>, spender_address: String) -> Result<String, String> {
    // Parse the spender address
    let spender_address = spender_address.parse::<Address>()
        .map_err(|e| format!("Invalid spender address: {}", e))?;
    
    // Determine owner address
    let owner_address = match owner_address {
        Some(addr) => addr.parse::<Address>()
            .map_err(|e| format!("Invalid owner address: {}", e))?,
        None => {
            let signer = create_icp_signer().await?;
            signer.address()
        }
    };

    // Setup provider (read-only, no wallet needed)
    let rpc_service = get_rpc_service_sepolia();
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new().on_icp(config);

    // Create USDC contract instance
    let contract = USDC::new(
        address!("1c7d4b196cb0c7b01d743fbc6116a902379c7238"),
        provider,
    );

    // Get allowance
    let allowance = contract.allowance(owner_address, spender_address).call().await
        .map_err(|e| format!("Failed to get allowance: {}", e))?;

    let allowance_human = allowance._0.to_string().parse::<f64>().unwrap_or(0.0) / 1_000_000.0;

    Ok(format!(
        "Allowance: {} USDC (raw: {}) - Owner: {}, Spender: {}",
        allowance_human,
        allowance._0,
        owner_address,
        spender_address
    ))
}

/// Revoke approval (set allowance to 0)
#[ic_cdk::update]
pub async fn revoke_usdc_approval(spender_address: String) -> Result<String, String> {
    approve_usdc(spender_address, "0".to_string()).await
} 