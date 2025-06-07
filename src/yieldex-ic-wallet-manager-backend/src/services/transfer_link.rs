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

// Codegen from ABI file to interact with the contract.
sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    LINK,
    "src/abi/LINK.json"
);

/// Transfer LINK tokens to a specified address.
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
pub async fn transfer_link(to_address: String, amount: String) -> Result<String, String> {
    // Parse the recipient address
    let to_address = to_address.parse::<Address>().map_err(|e| format!("Invalid address: {}", e))?;
    
    // Parse the amount (LINK has 18 decimals)
    let amount = amount.parse::<U256>().map_err(|e| format!("Invalid amount: {}", e))?;

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

    // Create contract instance using LINK token address on Sepolia
    let contract = LINK::new(
        address!("779877A7B0D9E8603169DdbD7836e478b4624789"),
        provider.clone(),
    );

    // Check balance before transfer
    let balance = contract.balanceOf(address).call().await
        .map_err(|e| format!("Failed to get balance: {}", e))?;
    
    if balance._0 < amount {
        return Err(format!("Insufficient LINK balance. Have: {}, Need: {}", balance._0, amount));
    }

    // Execute the transfer
    match contract
        .transfer(to_address, amount)
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
                    // The transaction has been mined and included in a block, the nonce
                    // has been consumed. Save it to thread-local storage. Next transaction
                    // for this address will use a nonce that is = this nonce + 1
                    NONCE.with_borrow_mut(|nonce| {
                        *nonce = Some(tx.nonce);
                    });
                    Ok(format!("Transaction successful: {:?}", tx_hash))
                }
                None => Err("Transaction not found after sending".to_string()),
            }
        }
        Err(e) => Err(format!("Transfer failed: {:?}", e)),
    }
}

/// Transfer LINK tokens with automatic amount conversion from human-readable format.
/// 
/// This function accepts amounts like "1.5" and automatically converts them to the proper
/// 18-decimal format required by LINK contracts.
#[ic_cdk::update]
pub async fn transfer_link_human(to_address: String, amount_human: String) -> Result<String, String> {
    // Parse human-readable amount (e.g., "1.5" LINK)
    let amount_f64: f64 = amount_human.parse()
        .map_err(|e| format!("Invalid amount format: {}", e))?;
    
    if amount_f64 <= 0.0 {
        return Err("Amount must be positive".to_string());
    }
    
    // Convert to Wei (multiply by 10^18 for LINK)
    let amount_wei = (amount_f64 * 1e18) as u128;
    let amount_u256 = U256::from(amount_wei);
    
    // Use the main transfer function
    transfer_link(to_address, amount_u256.to_string()).await
} 