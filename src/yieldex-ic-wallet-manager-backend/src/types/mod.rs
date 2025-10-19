// Re-export all types from submodules

pub mod permissions;
pub mod recommendation;
pub mod storable;

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
};
