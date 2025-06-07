use std::cell::RefCell;

use alloy::{
    network::EthereumWallet,
    primitives::{address, U256},
    providers::{Provider, ProviderBuilder},
    signers::Signer,
    sol,
    transports::icp::IcpConfig,
};

use crate::{create_icp_signer, get_rpc_service_sepolia};

thread_local! {
    static NONCE: RefCell<Option<u64>> = const { RefCell::new(None) };
}

// Codegen from ABI file to interact with the WETH contract.
sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    WETH,
    "src/abi/WETH.json"
);

/// Wrap ETH into WETH tokens by depositing ETH into the WETH contract.
/// 
/// This function converts ETH to WETH (Wrapped ETH) in a 1:1 ratio.
/// The ETH is sent to the WETH contract's deposit() function which mints
/// equivalent WETH tokens to the caller's address.
/// 
/// The following RPC calls are made:
/// - `eth_getTransactionCount`: To determine the next nonce
/// - `eth_estimateGas`: To determine the gas limit  
/// - `eth_sendRawTransaction`: The transaction
/// - `eth_getTransactionByHash`: To confirm success
#[ic_cdk::update]
pub async fn wrap_eth(amount: String) -> Result<String, String> {
    // Parse the amount (ETH has 18 decimals)
    let amount = amount.parse::<U256>()
        .map_err(|e| format!("Invalid amount: {}", e))?;

    if amount == U256::ZERO {
        return Err("Amount must be greater than 0".to_string());
    }

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

    // Check ETH balance first
    let balance = provider.get_balance(address).await
        .map_err(|e| format!("Failed to get ETH balance: {}", e))?;

    if balance < amount {
        return Err(format!(
            "Insufficient ETH balance. Have: {} ETH, Need: {} ETH", 
            balance.to_string().parse::<f64>().unwrap_or(0.0) / 1e18,
            amount.to_string().parse::<f64>().unwrap_or(0.0) / 1e18
        ));
    }

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

    // Create WETH contract instance (Sepolia WETH address)
    let contract = WETH::new(
        address!("7b79995e5f793A07Bc00c21412e50Ecae098E7f9"),
        provider.clone(),
    );

    // Execute the deposit transaction (payable function)
    match contract
        .deposit()
        .value(amount) // Send ETH with the transaction
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
                    
                    // Log the wrap
                    ic_cdk::println!(
                        "ETH wrap successful: {} wrapped {} ETH (raw: {}) into WETH",
                        address, 
                        amount.to_string().parse::<f64>().unwrap_or(0.0) / 1e18, // Convert to human readable
                        amount
                    );

                    Ok(format!(
                        "ETH wrap successful! Tx: {:?}. Wrapped {} ETH into WETH", 
                        tx_hash,
                        amount.to_string().parse::<f64>().unwrap_or(0.0) / 1e18
                    ))
                }
                None => Err("Transaction not found after sending".to_string()),
            }
        }
        Err(e) => Err(format!("Wrap transaction failed: {:?}", e)),
    }
}

/// Wrap ETH with human-readable amount.
/// 
/// This function accepts amounts like "0.1" (meaning 0.1 ETH) and automatically 
/// converts them to the proper 18-decimal format required by ETH/WETH contracts.
#[ic_cdk::update]
pub async fn wrap_eth_human(amount_human: String) -> Result<String, String> {
    // Parse human-readable amount (e.g., "0.1" ETH)
    let amount_f64: f64 = amount_human.parse()
        .map_err(|e| format!("Invalid amount format: {}", e))?;
    
    if amount_f64 <= 0.0 {
        return Err("Amount must be greater than 0".to_string());
    }
    
    // Convert to Wei (multiply by 10^18 for ETH's 18 decimals)
    let amount_wei = (amount_f64 * 1e18) as u128;
    let amount_u256 = U256::from(amount_wei);
    
    // Use the main wrap function
    wrap_eth(amount_u256.to_string()).await
}

/// Unwrap WETH back to ETH by calling withdraw() on the WETH contract.
/// 
/// This function converts WETH back to ETH in a 1:1 ratio.
/// The WETH tokens are burned and equivalent ETH is sent to the caller's address.
#[ic_cdk::update]
pub async fn unwrap_weth(amount: String) -> Result<String, String> {
    // Parse the amount (WETH has 18 decimals, same as ETH)
    let amount = amount.parse::<U256>()
        .map_err(|e| format!("Invalid amount: {}", e))?;

    if amount == U256::ZERO {
        return Err("Amount must be greater than 0".to_string());
    }

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

    // Create WETH contract instance
    let contract = WETH::new(
        address!("7b79995e5f793A07Bc00c21412e50Ecae098E7f9"),
        provider.clone(),
    );

    // Check WETH balance first
    let weth_balance = contract.balanceOf(address).call().await
        .map_err(|e| format!("Failed to get WETH balance: {}", e))?;

    if weth_balance._0 < amount {
        return Err(format!(
            "Insufficient WETH balance. Have: {} WETH, Need: {} WETH", 
            weth_balance._0.to_string().parse::<f64>().unwrap_or(0.0) / 1e18,
            amount.to_string().parse::<f64>().unwrap_or(0.0) / 1e18
        ));
    }

    // Execute the withdraw transaction
    match contract
        .withdraw(amount)
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
                    
                    // Log the unwrap
                    ic_cdk::println!(
                        "WETH unwrap successful: {} unwrapped {} WETH (raw: {}) into ETH",
                        address, 
                        amount.to_string().parse::<f64>().unwrap_or(0.0) / 1e18,
                        amount
                    );

                    Ok(format!(
                        "WETH unwrap successful! Tx: {:?}. Unwrapped {} WETH into ETH", 
                        tx_hash,
                        amount.to_string().parse::<f64>().unwrap_or(0.0) / 1e18
                    ))
                }
                None => Err("Transaction not found after sending".to_string()),
            }
        }
        Err(e) => Err(format!("Unwrap transaction failed: {:?}", e)),
    }
}

/// Unwrap WETH with human-readable amount.
#[ic_cdk::update]
pub async fn unwrap_weth_human(amount_human: String) -> Result<String, String> {
    // Parse human-readable amount (e.g., "0.1" WETH)
    let amount_f64: f64 = amount_human.parse()
        .map_err(|e| format!("Invalid amount format: {}", e))?;
    
    if amount_f64 <= 0.0 {
        return Err("Amount must be greater than 0".to_string());
    }
    
    // Convert to Wei (multiply by 10^18 for WETH's 18 decimals)
    let amount_wei = (amount_f64 * 1e18) as u128;
    let amount_u256 = U256::from(amount_wei);
    
    // Use the main unwrap function
    unwrap_weth(amount_u256.to_string()).await
}

 