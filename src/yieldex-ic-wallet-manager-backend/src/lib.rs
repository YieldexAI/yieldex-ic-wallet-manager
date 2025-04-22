use candid::{CandidType, Principal};
use ic_cdk_macros::{init, post_upgrade, query, update};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::RefCell;

// Alloy imports
use alloy::signers::icp::IcpSigner;
use alloy::signers::Signer; // The Signer trait
use alloy::primitives::Address;

// --- Configuration ---
const KEY_NAME: &str = "dfx_test_key"; // Use "key_1" for mainnet

// --- Types ---
type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct StorablePrincipal(Principal);

impl Storable for StorablePrincipal {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(self.0.as_slice().to_vec())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        StorablePrincipal(Principal::from_slice(&bytes))
    }

    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct StorableString(String);

impl Storable for StorableString {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(self.0.as_bytes().to_vec())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        StorableString(String::from_utf8(bytes.into_owned()).expect("Invalid UTF-8 for string"))
    }

    // Assuming max EVM address hex length (0x + 40 hex chars) = 42 bytes
    // Add some buffer
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Bounded { max_size: 64, is_fixed_size: false };
}

// --- State ---
const PRINCIPAL_MAP_MEMORY_ID: MemoryId = MemoryId::new(0);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // Map Principal -> EVM Address (hex string)
    static PRINCIPAL_TO_ADDRESS_MAP: RefCell<StableBTreeMap<StorablePrincipal, StorableString, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(PRINCIPAL_MAP_MEMORY_ID)),
        )
    );
}

// --- Helper Functions ---

// Removed get_ecdsa_key_id function as it's no longer used

// Removed get_principal_public_key

// Removed public_key_to_evm_address

// --- Canister Methods ---

#[update]
async fn generate_evm_address_for_principal(user: Principal) -> Result<String, String> {
    let storable_principal = StorablePrincipal(user);

    // 1. Check if address already exists for the given principal
    if let Some(storable_address) = PRINCIPAL_TO_ADDRESS_MAP.with(|map| map.borrow().get(&storable_principal)) {
        ic_cdk::println!("Address found for principal {}", user);
        return Ok(storable_address.0);
    }

    ic_cdk::println!("Address not found for principal {}, generating...", user);

    // 2. Generate new address using IcpSigner
    let derivation_path = vec![user.as_slice().to_vec()];

    // Create the ICP Signer. Pass derivation path, key name string, and chain ID (None for address gen)
    // Handle potential errors during signer creation (e.g., if management canister call fails)
    let signer = IcpSigner::new(derivation_path, KEY_NAME, None)
        .await
        .map_err(|e| format!("Failed to create ICP signer: {}", e))?;

    // Get the address from the signer
    let address: Address = signer.address();

    // Format the address as a hex string
    let address_hex = format!("0x{:x}", address);

    // 3. Store the new address
    PRINCIPAL_TO_ADDRESS_MAP.with(|map| {
        map.borrow_mut().insert(storable_principal, StorableString(address_hex.clone()));
    });
    ic_cdk::println!("Stored address {} for principal {}", address_hex, user);

    Ok(address_hex)
}

#[query]
fn verify_user(user: Principal) -> bool {
    // Check if the user has an address stored in the map.
    PRINCIPAL_TO_ADDRESS_MAP.with(|map| map.borrow().contains_key(&StorablePrincipal(user)))
}

#[update]
fn manage_whitelist(/* Placeholder arguments if needed later */) /* -> Result<(), String> */ {
    // Placeholder stub for Milestone 2 - Does nothing for now.
    ic_cdk::println!("manage_whitelist called (not implemented yet)");
    // Ok(()) // If returning Result
}

// --- Lifecycle Hooks (for stable memory) ---

#[init]
fn init() {
    // Initialization logic if needed (e.g., setting owners, initial config)
    ic_cdk::println!("SmartWallet Manager Initialized.");
}

#[post_upgrade]
fn post_upgrade() {
    // Stable memory is automatically preserved, no specific restore needed for StableBTreeMap
    ic_cdk::println!("SmartWallet Manager Upgraded.");
}

// --- Candid export ---
ic_cdk::export_candid!();
