use candid::{CandidType, Deserialize};

/// APY information for a specific protocol
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ProtocolApyInfo {
    /// Protocol name (e.g., "AAVE", "Compound")
    pub protocol: String,
    /// Current APY as a percentage string (e.g., "5.23")
    pub apy: String,
    /// Chain ID where this APY is available
    pub chain_id: u64,
}

/// Aggregated APY response for a token across protocols
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ApyResponse {
    /// Token address or symbol queried
    pub token: String,
    /// Chain ID where rates were checked
    pub chain_id: u64,
    /// List of protocol APY information
    pub rates: Vec<ProtocolApyInfo>,
}
