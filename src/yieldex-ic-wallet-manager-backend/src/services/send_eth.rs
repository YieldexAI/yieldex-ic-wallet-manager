use std::cell::RefCell;

use alloy::{
    network::{EthereumWallet, TransactionBuilder},
    primitives::U256,
    providers::{Provider, ProviderBuilder},
    rpc::types::request::TransactionRequest,
    primitives::Address,
    signers::Signer,
    transports::icp::IcpConfig,
};

use crate::create_icp_signer;
use crate::services::rpc_service::{get_rpc_service_by_chain_id, SEPOLIA_CHAIN_ID};

thread_local! {
    static NONCE: RefCell<Option<u64>> = const { RefCell::new(None) };
}

/// This function will attempt to send ETH to a specified address.
///
/// Nonce handling is implemented manually instead of relying on the Alloy built in
/// `with_recommended_fillers` method. This minimizes the number of requests sent to the
/// EVM RPC.
///
/// The following RPC calls are made to complete a transaction:
/// - `eth_getTransactionCount`: To determine the next nonce. This call is only made once after
/// canister deployment, then the nonces are cached.
/// - `eth_estimateGas`: To determine the gas limit
/// - `eth_sendRawTransaction`: The transaction
/// - `eth_getTransactionByHash`: To determine if transaction was successful. Increment nonce only
/// if transaction was successful.
#[ic_cdk::update]
pub async fn send_eth(to_address: String, amount_wei: String) -> Result<String, String> {
    // Parse the recipient address
    let to_address = to_address.parse::<Address>().map_err(|e| format!("Invalid address: {}", e))?;
    
    // Parse the amount in wei
    let amount = amount_wei.parse::<U256>().map_err(|e| format!("Invalid amount: {}", e))?;

    // Setup signer - properly handle the Result
    let signer = create_icp_signer().await?;
    let address = signer.address();

    // Setup provider
    let wallet = EthereumWallet::from(signer);
    let rpc_service = get_rpc_service_by_chain_id(SEPOLIA_CHAIN_ID)?;
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new()
        .with_gas_estimation()
        .wallet(wallet)
        .on_icp(config);

    // Attempt to get nonce from thread-local storage
    let maybe_nonce = NONCE.with_borrow(|maybe_nonce| {
        // If a nonce exists, the next nonce to use is latest nonce + 1
        maybe_nonce.map(|nonce| nonce + 1)
    });

    // If no nonce exists, get it from the provider
    let nonce = if let Some(nonce) = maybe_nonce {
        nonce
    } else {
        provider.get_transaction_count(address).await.map_err(|e| format!("Failed to get nonce: {}", e))?
    };

    // Check ETH balance before sending
    let balance = provider.get_balance(address).await
        .map_err(|e| format!("Failed to get ETH balance: {}", e))?;
    
    if balance < amount {
        return Err(format!("Insufficient ETH balance. Have: {} wei, Need: {} wei", balance, amount));
    }

    let tx = TransactionRequest::default()
        .with_to(to_address)
        .with_value(amount)
        .with_nonce(nonce)
        .with_chain_id(11155111); // Sepolia chain ID

    let transport_result = provider.send_transaction(tx.clone()).await;
    match transport_result {
        Ok(builder) => {
            let tx_hash = *builder.tx_hash();
            let tx_response = provider.get_transaction_by_hash(tx_hash).await
                .map_err(|e| format!("Failed to get transaction: {}", e))?;

            match tx_response {
                Some(tx) => {
                    // The transaction has been mined and included in a block, the nonce
                    // has been consumed. Save it to thread-local storage. Next transaction
                    // for this address will use a nonce that is = this nonce + 1
                    NONCE.with_borrow_mut(|nonce| {
                        *nonce = Some(tx.nonce);
                    });
                    Ok(format!("ETH transaction successful: {:?}", tx_hash))
                }
                None => Err("Transaction not found after sending".to_string()),
            }
        }
        Err(e) => Err(format!("ETH transfer failed: {:?}", e)),
    }
}

/// Send ETH with human-readable amount conversion from Ether to Wei
#[ic_cdk::update]
pub async fn send_eth_human(to_address: String, amount_ether: String) -> Result<String, String> {
    // Parse human-readable amount (e.g., "0.001" ETH)
    let amount_f64: f64 = amount_ether.parse()
        .map_err(|e| format!("Invalid amount format: {}", e))?;
    
    if amount_f64 <= 0.0 {
        return Err("Amount must be positive".to_string());
    }
    
    // Convert to Wei (multiply by 10^18 for ETH)
    let amount_wei = (amount_f64 * 1e18) as u128;
    let amount_u256 = U256::from(amount_wei);
    
    // Use the main send function
    send_eth(to_address, amount_u256.to_string()).await
}