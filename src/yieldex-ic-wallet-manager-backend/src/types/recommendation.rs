use candid::CandidType;
use serde::{Deserialize, Serialize};

// --- Recommendation Types ---

/// Recommendation type for rebalancing operations
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum RecommendationType {
    StandardTransfer, // Standard transfer between protocols on the same network
    CrossChainTransfer,  // Transfer between protocols on different networks
}

/// Swap operation details (optional, for future versions)
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct SwapDetails {
    pub from_token: String,
    pub to_token: String,
    pub from_market: Option<String>,
    pub to_market: Option<String>,
    pub swap_protocol: Option<String>,
}

/// Recommendation structure for rebalancing
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct Recommendation {
    pub asset: String,                        // "USDC"
    pub to_asset: String,                     // "USDC"
    pub from_chain: String,                   // "Arbitrum"
    pub to_chain: Option<String>,             // Optional for cross-chain
    pub from_protocol: String,                // "aave-v3" | "compound-v3"
    pub to_protocol: String,                  // "aave-v3" | "compound-v3"
    pub current_apy: f64,                     // Current annual percentage yield
    pub target_apy: f64,                      // Target annual percentage yield
    pub estimated_profit: f64,                // Estimated profit
    pub gas_cost: f64,                        // Gas cost
    pub position_size: String,                // Amount in human-readable format "1000"
    pub pool_id: Option<String>,              // Pool identifier
    pub recommendation_type: RecommendationType, // StandardTransfer or CrossChainTransfer
    pub swap_details: Option<SwapDetails>,    // For swap operations
}

/// Recommendation execution result
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub status: String,                       // "success" | "failed" | "partial"
    pub withdraw_tx: Option<String>,          // Withdraw transaction hash
    pub swap_tx: Option<String>,              // Swap transaction hash (for future use)
    pub supply_tx: Option<String>,            // Supply transaction hash
    pub amount_transferred: Option<String>,   // Amount actually transferred
    pub actual_gas_cost: Option<f64>,         // Actual gas cost
    pub error_details: Option<String>,        // Error details
}
