use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

// --- Type Aliases ---
pub type PermissionsId = String;
pub type TokenAddress = String;

// --- Permission-related Types ---

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Protocol {
    pub name: String,
    pub address: String,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token {
    pub name: String,
    pub address: String,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct TransferLimit {
    pub token_address: TokenAddress,
    pub daily_limit: u64,
    pub max_tx_amount: u64,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct ProtocolPermission {
    pub protocol_address: String,
    pub allowed_functions: Vec<String>, // ["supply", "withdraw", "borrow"]
    pub max_amount_per_tx: Option<u64>,
    pub daily_limit: Option<u64>,
    pub total_used_today: u64,
    pub last_reset_date: u64, // Timestamp for daily limit reset
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct Permissions {
    pub id: PermissionsId,
    pub owner: Principal,
    pub chain_id: u64,
    pub whitelisted_protocols: Vec<Protocol>,
    pub whitelisted_tokens: Vec<Token>,
    pub transfer_limits: Vec<TransferLimit>,
    pub protocol_permissions: Vec<ProtocolPermission>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct CreatePermissionsRequest {
    pub chain_id: u64,
    pub whitelisted_protocols: Vec<Protocol>,
    pub whitelisted_tokens: Vec<Token>,
    pub transfer_limits: Vec<TransferLimit>,
    pub protocol_permissions: Option<Vec<ProtocolPermission>>,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct UpdatePermissionsRequest {
    pub permissions_id: PermissionsId,
    pub chain_id: Option<u64>,
    pub whitelisted_protocols: Option<Vec<Protocol>>,
    pub whitelisted_tokens: Option<Vec<Token>>,
    pub transfer_limits: Option<Vec<TransferLimit>>,
    pub protocol_permissions: Option<Vec<ProtocolPermission>>,
}
