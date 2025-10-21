use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use crate::types::{Recommendation, ExecutionResult};

// =============================================================================
// Scheduler Configuration Types
// =============================================================================

/// Global scheduler configuration
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Whether the scheduler is currently enabled
    pub enabled: bool,

    /// Interval between scheduler checks in seconds (e.g., 3600 = 1 hour)
    pub interval_seconds: u64,

    /// Minimum APY difference percentage to trigger rebalance (e.g., 0.5 for 0.5%)
    pub apy_threshold_percent: f64,

    /// Minimum position size to consider for rebalancing (human-readable format, e.g., "100")
    pub min_position_size: String,

    /// Timestamp of last scheduler execution
    pub last_execution: Option<u64>,

    /// Configuration creation timestamp
    pub created_at: u64,

    /// Configuration last update timestamp
    pub updated_at: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_seconds: 3600, // 1 hour default
            apy_threshold_percent: 0.5, // 0.5% APY difference
            min_position_size: "100".to_string(), // $100 USDC minimum
            last_execution: None,
            created_at: 0,
            updated_at: 0,
        }
    }
}

/// Scheduler status information for monitoring
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct SchedulerStatus {
    pub config: SchedulerConfig,
    pub timer_active: bool,
    pub total_positions_tracked: u64,
    pub total_rebalances_executed: u64,
    pub last_execution_result: Option<String>,
}

// =============================================================================
// User Position Types (Future DB)
// =============================================================================

/// User's position in a DeFi protocol
/// This will be stored in a future database to track all user positions
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct UserPosition {
    /// Unique identifier for this position
    pub position_id: String,

    /// User who owns this position
    pub user_principal: Principal,

    /// Permissions ID associated with this position
    pub permissions_id: String,

    /// Protocol where the position is held ("AAVE" | "COMPOUND")
    pub protocol: String,

    /// Asset type (e.g., "USDC")
    pub asset: String,

    /// Chain ID where the position exists
    pub chain_id: u64,

    /// Current position size in human-readable format
    pub position_size: String,

    /// Whether this position should be tracked for auto-rebalancing
    pub tracked: bool,

    /// Last time this position was updated
    pub last_updated: u64,
}

// =============================================================================
// APY History Types (Future DB)
// =============================================================================

/// Historical APY record for a protocol
/// This will be stored in a future database to track APY over time
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct ApyHistoryRecord {
    /// Unique identifier for this record
    pub record_id: String,

    /// Protocol name ("AAVE" | "COMPOUND")
    pub protocol: String,

    /// Asset symbol (e.g., "USDC")
    pub asset: String,

    /// Chain ID
    pub chain_id: u64,

    /// APY percentage value
    pub apy: f64,

    /// Timestamp when this APY was recorded
    pub timestamp: u64,
}

// =============================================================================
// Rebalance Execution History
// =============================================================================

/// Record of a rebalance execution performed by the scheduler
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct RebalanceExecution {
    /// Unique execution ID
    pub execution_id: String,

    /// User for whom the rebalance was executed
    pub user_principal: Principal,

    /// Position that was rebalanced
    pub position_id: String,

    /// The recommendation that was generated and executed
    pub recommendation: Recommendation,

    /// Result of the execution
    pub result: ExecutionResult,

    /// APY difference that triggered the rebalance
    pub apy_difference: f64,

    /// Timestamp when execution occurred
    pub timestamp: u64,
}

/// Summary of a scheduler execution tick
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct SchedulerExecutionSummary {
    /// Timestamp of this execution
    pub timestamp: u64,

    /// Number of positions checked
    pub positions_checked: u64,

    /// Number of rebalances triggered
    pub rebalances_triggered: u64,

    /// Number of successful rebalances
    pub rebalances_successful: u64,

    /// Number of failed rebalances
    pub rebalances_failed: u64,

    /// List of execution IDs for this tick
    pub execution_ids: Vec<String>,

    /// Any errors encountered during the tick
    pub errors: Vec<String>,
}
