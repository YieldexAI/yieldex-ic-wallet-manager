use alloy::transports::icp::{RpcService, RpcApi};

// Chain ID constants for supported networks
pub const SEPOLIA_CHAIN_ID: u64 = 11155111;
pub const ARBITRUM_CHAIN_ID: u64 = 42161;
pub const BASE_CHAIN_ID: u64 = 8453;
pub const OPTIMISM_CHAIN_ID: u64 = 10;

/// Creates an RPC service for Ethereum Sepolia testnet
fn get_rpc_service_sepolia() -> RpcService {
    RpcService::Custom(RpcApi {
        url: "https://ic-alloy-evm-rpc-proxy.kristofer-977.workers.dev/eth-sepolia".to_string(),
        headers: None,
    })
}

/// Creates an RPC service for Arbitrum One mainnet
fn get_rpc_service_arbitrum() -> RpcService {
    RpcService::Custom(RpcApi { url: "https://yieldex-evm-proxy.exectrogod.workers.dev/arb-mainnet".to_string(), headers: None })
}

/// Creates an RPC service for Base mainnet
fn get_rpc_service_base() -> RpcService {
    RpcService::Custom(RpcApi {
        url: "https://base-rpc.publicnode.com".to_string(),
        headers: None,
    })
}

/// Creates an RPC service for Optimism mainnet
fn get_rpc_service_optimism() -> RpcService {
    RpcService::Custom(RpcApi {
        url: "https://optimism-rpc.publicnode.com".to_string(),
        headers: None,
    })
}

/// Main function to select an RPC service by chain_id
pub fn get_rpc_service_by_chain_id(chain_id: u64) -> Result<RpcService, String> {
    match chain_id {
        SEPOLIA_CHAIN_ID => Ok(get_rpc_service_sepolia()),
        ARBITRUM_CHAIN_ID => Ok(get_rpc_service_arbitrum()),
        BASE_CHAIN_ID => Ok(get_rpc_service_base()),
        OPTIMISM_CHAIN_ID => Ok(get_rpc_service_optimism()),
        _ => Err(format!("Unsupported chain_id: {}. Supported chains: {} (Sepolia), {} (Arbitrum), {} (Base), {} (Optimism)", 
                        chain_id, SEPOLIA_CHAIN_ID, ARBITRUM_CHAIN_ID, BASE_CHAIN_ID, OPTIMISM_CHAIN_ID))
    }
}

/// Returns a list of supported chain IDs
pub fn get_supported_chain_ids() -> Vec<u64> {
    vec![SEPOLIA_CHAIN_ID, ARBITRUM_CHAIN_ID, BASE_CHAIN_ID, OPTIMISM_CHAIN_ID]
}

/// Returns a human-readable network name by chain_id
pub fn get_chain_name(chain_id: u64) -> Option<&'static str> {
    match chain_id {
        SEPOLIA_CHAIN_ID => Some("Ethereum Sepolia"),
        ARBITRUM_CHAIN_ID => Some("Arbitrum One"),
        BASE_CHAIN_ID => Some("Base Mainnet"),
        OPTIMISM_CHAIN_ID => Some("Optimism Mainnet"),
        _ => None
    }
}

/// Checks if the specified chain_id is supported
pub fn is_supported_chain(chain_id: u64) -> bool {
    matches!(chain_id, SEPOLIA_CHAIN_ID | ARBITRUM_CHAIN_ID | BASE_CHAIN_ID | OPTIMISM_CHAIN_ID)
}

/// Returns information about all supported networks
pub fn get_supported_chains_info() -> Vec<(u64, &'static str)> {
    vec![
        (SEPOLIA_CHAIN_ID, "Ethereum Sepolia"),
        (ARBITRUM_CHAIN_ID, "Arbitrum One"),
        (BASE_CHAIN_ID, "Base Mainnet"),
        (OPTIMISM_CHAIN_ID, "Optimism Mainnet"),
    ]
}