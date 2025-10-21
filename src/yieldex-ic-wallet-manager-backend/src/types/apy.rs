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

/// APY Parser status information
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ApyParserStatus {
    /// Whether APY parser is enabled
    pub enabled: bool,
    /// Collection interval in seconds
    pub interval_seconds: u64,
    /// Whether timer is currently active
    pub timer_active: bool,
    /// Last execution timestamp
    pub last_execution: Option<u64>,
    /// Total APY records in database
    pub total_records: u64,
    /// Monitored protocols
    pub monitored_protocols: Vec<String>,
    /// Monitored chains
    pub monitored_chains: Vec<u64>,
}
