use alloy::{
    network::TxSigner,
    primitives::{address, Address},
    providers::ProviderBuilder,
    sol,
    transports::icp::IcpConfig,
};

use crate::create_icp_signer;
use crate::services::rpc_service::{get_rpc_service_by_chain_id, SEPOLIA_CHAIN_ID, ARBITRUM_CHAIN_ID, BASE_CHAIN_ID, OPTIMISM_CHAIN_ID};

// Codegen from ABI file to interact with the contract.
sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    USDC,
    "src/abi/USDC.json"
);

/// Get USDC contract address based on chain_id
fn get_usdc_address(chain_id: u64) -> Result<Address, String> {
    match chain_id {
        SEPOLIA_CHAIN_ID => Ok(address!("1c7d4b196cb0c7b01d743fbc6116a902379c7238")), // USDC on Sepolia (bridged)
        ARBITRUM_CHAIN_ID => Ok(address!("af88d065e77c8cc2239327c5edb3a432268e5831")), // Native USDC on Arbitrum One
        BASE_CHAIN_ID => Ok(address!("833589fcd6edb6e08f4c7c32d4f71b54bda02913")), // Native USDC on Base
        OPTIMISM_CHAIN_ID => Ok(address!("7f5c764cbc14f9669b88837ca1490cca17c31607")), // USDC on Optimism
        _ => Err(format!("USDC not configured for chain_id: {}", chain_id))
    }
}

/// Request the USDC balance of an account for a specific chain.
#[ic_cdk::update]
pub async fn get_balance_usdc(address: Option<String>, chain_id: u64) -> Result<String, String> {
    let address = match address {
        Some(val) => val,
        None => {
            let signer = create_icp_signer().await?;
            signer.address().to_string()
        }
    };
    let address = address.parse::<Address>().map_err(|e| e.to_string())?;
    let rpc_service = get_rpc_service_by_chain_id(chain_id)?;
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new().on_icp(config);

    let usdc_address = get_usdc_address(chain_id)?;
    let contract = USDC::new(usdc_address, provider);

    let result = contract.balanceOf(address).call().await;
    match result {
        Ok(balance) => Ok(balance._0.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// Legacy function for backward compatibility - uses Sepolia by default
#[ic_cdk::update]
pub async fn get_balance_usdc_legacy(address: Option<String>) -> Result<String, String> {
    get_balance_usdc(address, SEPOLIA_CHAIN_ID).await
} 