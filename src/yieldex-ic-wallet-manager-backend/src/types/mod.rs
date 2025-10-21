// Re-export all types from submodules

pub mod permissions;
pub mod recommendation;
pub mod storable;
pub mod apy;
pub mod scheduler;

// Re-export commonly used types for convenience
pub use permissions::{
    ProtocolPermission,
    Permissions, CreatePermissionsRequest, UpdatePermissionsRequest,
};

pub use recommendation::{
    RecommendationType, Recommendation, ExecutionResult,
};

pub use storable::{
    StorablePrincipal, StorableString, StorablePermissions,
    StorableUserPosition, StorableApyHistoryRecord, StorableRebalanceExecution,
};

pub use apy::{
    ProtocolApyInfo, ApyResponse,
};

pub use scheduler::{
    SchedulerConfig, SchedulerStatus, UserPosition, ApyHistoryRecord,
    RebalanceExecution, SchedulerExecutionSummary,
};
