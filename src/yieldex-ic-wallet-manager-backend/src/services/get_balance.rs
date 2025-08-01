use alloy::{
    network::TxSigner,
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    transports::icp::IcpConfig,
};

use crate::create_icp_signer;
use crate::services::rpc_service::{get_rpc_service_by_chain_id, SEPOLIA_CHAIN_ID};

/// Request the balance of an ETH account.
#[ic_cdk::update]
pub async fn get_balance(address: Option<String>) -> Result<String, String> {
    let address = match address {
        Some(val) => val,
        None => {
            let signer = create_icp_signer().await?;
            signer.address().to_string()
        }
    };
    let address = address.parse::<Address>().map_err(|e| e.to_string())?;
    let rpc_service = get_rpc_service_by_chain_id(SEPOLIA_CHAIN_ID)?;
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new().on_icp(config);
    let result = provider.get_balance(address).await;

    match result {
        Ok(balance) => Ok(balance.to_string()),
        Err(e) => Err(e.to_string()),
    }
} 