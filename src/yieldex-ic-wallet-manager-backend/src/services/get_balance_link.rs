use alloy::{
    network::TxSigner,
    primitives::{address, Address},
    providers::ProviderBuilder,
    sol,
    transports::icp::IcpConfig,
};

use crate::{create_icp_signer, get_rpc_service_sepolia};

// Codegen from ABI file to interact with the contract.
sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    LINK,
    "src/abi/LINK.json"
);

/// Request the LINK balance of an account.
#[ic_cdk::update]
pub async fn get_balance_link(address: Option<String>) -> Result<String, String> {
    let address = match address {
        Some(val) => val,
        None => {
            let signer = create_icp_signer().await?;
            signer.address().to_string()
        }
    };
    let address = address.parse::<Address>().map_err(|e| e.to_string())?;
    let rpc_service = get_rpc_service_sepolia();
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new().on_icp(config);

    let contract = LINK::new(
        address!("779877A7B0D9E8603169DdbD7836e478b4624789"),
        provider,
    );

    let result = contract.balanceOf(address).call().await;
    match result {
        Ok(balance) => Ok(balance._0.to_string()),
        Err(e) => Err(e.to_string()),
    }
} 