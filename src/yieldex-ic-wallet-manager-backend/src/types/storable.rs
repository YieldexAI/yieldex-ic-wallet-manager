use candid::{CandidType, Principal};
use ic_stable_structures::Storable;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use super::permissions::Permissions;
use super::scheduler::{UserPosition, ApyHistoryRecord, RebalanceExecution};

// --- Storable Wrapper Types ---

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct StorablePrincipal(pub Principal);

impl Storable for StorablePrincipal {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(self.0.as_slice().to_vec())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        StorablePrincipal(Principal::from_slice(&bytes))
    }

    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct StorableString(pub String);

impl Storable for StorableString {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(self.0.as_bytes().to_vec())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        StorableString(String::from_utf8(bytes.into_owned()).expect("Invalid UTF-8 for string"))
    }

    // Max size increased to 128 bytes to accommodate APY record IDs
    // Format: "PROTOCOL:CHAIN_ID:TOKEN_ADDRESS:TIMESTAMP" (e.g., 67 bytes)
    // Also accommodates EVM addresses (42 bytes) and other use cases
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Bounded { max_size: 128, is_fixed_size: false };
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct StorablePermissions(pub Permissions);

impl Storable for StorablePermissions {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = candid::encode_one(&self.0).expect("Failed to encode permissions");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let permissions: Permissions = candid::decode_one(&bytes).expect("Failed to decode permissions");
        StorablePermissions(permissions)
    }

    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

// --- Scheduler Storable Wrappers ---

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct StorableUserPosition(pub UserPosition);

impl Storable for StorableUserPosition {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = candid::encode_one(&self.0).expect("Failed to encode UserPosition");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let position: UserPosition = candid::decode_one(&bytes).expect("Failed to decode UserPosition");
        StorableUserPosition(position)
    }

    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct StorableApyHistoryRecord(pub ApyHistoryRecord);

impl Storable for StorableApyHistoryRecord {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = candid::encode_one(&self.0).expect("Failed to encode ApyHistoryRecord");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let record: ApyHistoryRecord = candid::decode_one(&bytes).expect("Failed to decode ApyHistoryRecord");
        StorableApyHistoryRecord(record)
    }

    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct StorableRebalanceExecution(pub RebalanceExecution);

impl Storable for StorableRebalanceExecution {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = candid::encode_one(&self.0).expect("Failed to encode RebalanceExecution");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let execution: RebalanceExecution = candid::decode_one(&bytes).expect("Failed to decode RebalanceExecution");
        StorableRebalanceExecution(execution)
    }

    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}
