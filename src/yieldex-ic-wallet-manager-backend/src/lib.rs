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
use alloy::transports::icp::{RpcApi, RpcService};

// Services module
mod services;
use services::{
    get_balance::get_balance, 
    get_balance_link::get_balance_link, 
    get_balance_usdc::get_balance_usdc,
    transfer_link::{transfer_link, transfer_link_human},
    send_eth::{send_eth, send_eth_human},
    approve_usdc::{approve_usdc, approve_usdc_human, get_usdc_allowance, revoke_usdc_approval},
    approve_weth::{approve_weth_for_uniswap, approve_weth, approve_weth_human, get_weth_allowance, get_weth_balance, revoke_weth_approval},
    sign_message::{sign_message, sign_message_with_address, sign_hash},
    wrap_eth::{wrap_eth, wrap_eth_human, unwrap_weth, unwrap_weth_human},
    uniswap::{get_weth_usdc_quote_v3_human, approve_weth_for_universal_router_human}
};

// --- Types ---
type Memory = VirtualMemory<DefaultMemoryImpl>;

// --- Permissions Types ---
type PermissionsId = String;
type TokenAddress = String;

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Protocol {
    pub name: String,
    pub address: String,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct TransferLimit {
    token_address: TokenAddress,
    daily_limit: u64,
    max_tx_amount: u64,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token {
    pub name: String,
    pub address: String,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct Permissions {
    id: PermissionsId,
    owner: Principal,
    whitelisted_protocols: Vec<Protocol>,
    whitelisted_tokens: Vec<Token>,
    transfer_limits: Vec<TransferLimit>,
    created_at: u64,
    updated_at: u64,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct CreatePermissionsRequest {
    whitelisted_protocols: Vec<Protocol>,
    whitelisted_tokens: Vec<Token>,
    transfer_limits: Vec<TransferLimit>,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct UpdatePermissionsRequest {
    permissions_id: PermissionsId,
    whitelisted_protocols: Option<Vec<Protocol>>,
    whitelisted_tokens: Option<Vec<Token>>,
    transfer_limits: Option<Vec<TransferLimit>>,
}

// Storable implementations
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

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct StorablePermissions(Permissions);

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

// --- State ---
const PRINCIPAL_MAP_MEMORY_ID: MemoryId = MemoryId::new(0);
const PERMISSIONS_MAP_MEMORY_ID: MemoryId = MemoryId::new(1);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // Map Principal -> EVM Address (hex string)
    static PRINCIPAL_TO_ADDRESS_MAP: RefCell<StableBTreeMap<StorablePrincipal, StorableString, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(PRINCIPAL_MAP_MEMORY_ID)),
        )
    );
    
    // Map PermissionsId -> Permissions
    static PERMISSIONS_MAP: RefCell<StableBTreeMap<StorableString, StorablePermissions, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(PERMISSIONS_MAP_MEMORY_ID)),
        )
    );
}

// --- Helper Functions ---

// Get current timestamp in milliseconds
fn now() -> u64 {
    // Returns milliseconds since Unix epoch
    ic_cdk::api::time() / 1_000_000
}

// Generate a unique ID for Permissions
async fn generate_permissions_id() -> String {
    let timestamp: u64 = now();
    let random_result = ic_cdk::api::management_canister::main::raw_rand().await;
    let random_bytes = match random_result {
        Ok((bytes,)) => bytes,
        Err((_, err)) => {
            ic_cdk::println!("Error getting random bytes: {}", err);
            // Fallback to a timestamp-based ID if random generation fails
            format!("{:x}", timestamp).into_bytes()
        }
    };
    
    let mut id_bytes = timestamp.to_be_bytes().to_vec();
    if random_bytes.len() >= 8 {
        id_bytes.extend_from_slice(&random_bytes[0..8]);
    } else {
        // Fallback in case we received fewer than expected random bytes
        id_bytes.extend_from_slice(&random_bytes);
        // Pad with zeros if needed
        while id_bytes.len() < 16 {
            id_bytes.push(0);
        }
    }
    
    hex::encode(id_bytes)
}

// --- EVM Address Management ---

#[update]
async fn generate_evm_address() -> Result<String, String> {
    let user = ic_cdk::caller();
    let storable_principal = StorablePrincipal(user);

    // 1. Check if address already exists for the given principal
    if let Some(storable_address) = PRINCIPAL_TO_ADDRESS_MAP.with(|map| map.borrow().get(&storable_principal)) {
        ic_cdk::println!("Address found for principal {}", user);
        return Ok(storable_address.0);
    }

    ic_cdk::println!("Address not found for principal {}, generating...", user);

    // 2. Generate new address using IcpSigner
    let signer = create_icp_signer().await?;

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
fn get_evm_address() -> Result<String, String> {
    let user = ic_cdk::caller();
    PRINCIPAL_TO_ADDRESS_MAP.with(|map| {
        map.borrow()
            .get(&StorablePrincipal(user))
            .map(|storable| Ok(storable.0))
            .unwrap_or_else(|| Err("EVM address not found. Please create it via generate_evm_address.".to_string()))
    })
}

#[query]
fn verify_user(user: Principal) -> Result<bool, String> {
    // Check if the user has an address stored in the map.
    let exists = PRINCIPAL_TO_ADDRESS_MAP.with(|map| map.borrow().contains_key(&StorablePrincipal(user)));
    if exists {
        Ok(true)
    } else {
        Err("User does not have an EVM address".to_string())
    }
}

// --- Permissions Management ---

// Check if caller is owner of the permissions
fn is_permissions_owner(permissions_id: &str, caller: Principal) -> bool {
    PERMISSIONS_MAP.with(|map| {
        map.borrow()
            .get(&StorableString(permissions_id.to_string()))
            .map_or(false, |p| p.0.owner == caller)
    })
}

#[update]
async fn create_permissions(req: CreatePermissionsRequest) -> Result<Permissions, String> {
    let caller = ic_cdk::caller();
    // Check if the caller has an EVM address
    match verify_user(caller) {
        Ok(true) => {},
        Ok(false) => return Err("You must generate an EVM address first".to_string()),
        Err(e) => return Err(e),
    }
    let permissions_id = generate_permissions_id().await;
    let timestamp = now();
    let permissions = Permissions {
        id: permissions_id.clone(),
        owner: caller,
        whitelisted_protocols: req.whitelisted_protocols,
        whitelisted_tokens: req.whitelisted_tokens,
        transfer_limits: req.transfer_limits,
        created_at: timestamp,
        updated_at: timestamp,
    };
    PERMISSIONS_MAP.with(|map| {
        map.borrow_mut().insert(
            StorableString(permissions_id), 
            StorablePermissions(permissions.clone())
        );
    });
    Ok(permissions)
}

#[query]
fn get_permissions(permissions_id: String) -> Result<Permissions, String> {
    let caller = ic_cdk::caller();
    
    PERMISSIONS_MAP.with(|map| {
        map.borrow()
            .get(&StorableString(permissions_id.clone()))
            .map_or_else(
                || Err(format!("Permissions with ID {} not found", permissions_id)),
                |p| {
                    if p.0.owner == caller {
                        Ok(p.0)
                    } else {
                        Err("You do not have permission to access these permissions".to_string())
                    }
                }
            )
    })
}

#[query]
fn get_all_permissions() -> Result<Vec<Permissions>, String> {
    let caller = ic_cdk::caller();
    
    // Collect all permissions owned by the caller
    let mut result = Vec::new();
    
    PERMISSIONS_MAP.with(|map| {
        let borrowed_map = map.borrow();
        
        // Iterate through all permissions and filter by owner
        for (_, permissions) in borrowed_map.iter() {
            if permissions.0.owner == caller {
                result.push(permissions.0.clone());
            }
        }
    });
    
    Ok(result)
}

#[update]
fn update_permissions(req: UpdatePermissionsRequest) -> Result<Permissions, String> {
    let caller = ic_cdk::caller();
    let permissions_id = req.permissions_id.clone();
    
    // Check if permissions exist and caller is the owner
    if !is_permissions_owner(&permissions_id, caller) {
        return Err(format!("Permissions with ID {} not found or you do not have permission to update them", permissions_id));
    }
    
    // Get the existing permissions
    let mut permissions = PERMISSIONS_MAP.with(|map| {
        map.borrow()
            .get(&StorableString(permissions_id.clone()))
            .unwrap().0.clone()
    });
    
    // Update fields if provided
    if let Some(protocols) = req.whitelisted_protocols {
        permissions.whitelisted_protocols = protocols;
    }
    
    if let Some(tokens) = req.whitelisted_tokens {
        permissions.whitelisted_tokens = tokens;
    }
    
    if let Some(limits) = req.transfer_limits {
        permissions.transfer_limits = limits;
    }
    
    // Update the timestamp
    permissions.updated_at = now();
    
    // Save the updated permissions
    PERMISSIONS_MAP.with(|map| {
        map.borrow_mut().insert(
            StorableString(permissions_id), 
            StorablePermissions(permissions.clone())
        );
    });
    
    Ok(permissions)
}

#[update]
fn delete_permissions(permissions_id: String) -> Result<bool, String> {
    let caller = ic_cdk::caller();
    // Check if permissions exist and caller is the owner
    if !is_permissions_owner(&permissions_id, caller) {
        return Err(format!("Permissions with ID {} not found or you do not have permission to delete them", permissions_id));
    }
    // Delete the permissions
    let removed = PERMISSIONS_MAP.with(|map| map.borrow_mut().remove(&StorableString(permissions_id))).is_some();
    if removed {
        Ok(true)
    } else {
        Err("Failed to delete permissions (not found)".to_string())
    }
}

// --- Balance Service Methods ---

/// Get ETH balance for an address (or current user's address if none provided)
#[update]
async fn get_eth_balance(address: Option<String>) -> Result<String, String> {
    get_balance(address).await
}

/// Get USDC balance for an address (or current user's address if none provided)
#[update]
async fn get_usdc_balance(address: Option<String>) -> Result<String, String> {
    get_balance_usdc(address).await
}

/// Get LINK balance for an address (or current user's address if none provided)
#[update]
async fn get_link_balance(address: Option<String>) -> Result<String, String> {
    get_balance_link(address).await
}

// --- Transfer Service Methods ---

/// Transfer LINK tokens to a specified address
/// Amount should be in Wei format (18 decimals for LINK)
#[update]
async fn transfer_link_tokens(to_address: String, amount: String) -> Result<String, String> {
    transfer_link(to_address, amount).await
}

/// Transfer LINK tokens with human-readable amount (e.g. "1.5" for 1.5 LINK)
#[update]  
async fn transfer_link_human_readable(to_address: String, amount_human: String) -> Result<String, String> {
    transfer_link_human(to_address, amount_human).await
}

/// Send ETH to a specified address
/// Amount should be in Wei format (18 decimals for ETH)
#[update]
async fn send_eth_tokens(to_address: String, amount_wei: String) -> Result<String, String> {
    send_eth(to_address, amount_wei).await
}

/// Send ETH with human-readable amount (e.g. "0.001" for 0.001 ETH)
#[update]
async fn send_eth_human_readable(to_address: String, amount_ether: String) -> Result<String, String> {
    send_eth_human(to_address, amount_ether).await
}

// --- Approve Service Methods ---

/// Approve USDC spending for a spender address
/// Amount should be in USDC units (6 decimals)
#[update]
async fn approve_usdc_spending(spender_address: String, amount: String) -> Result<String, String> {
    approve_usdc(spender_address, amount).await
}

/// Approve USDC spending with human-readable amount (e.g. "100.50" for 100.50 USDC)
#[update]
async fn approve_usdc_human_readable(spender_address: String, amount_human: String) -> Result<String, String> {
    approve_usdc_human(spender_address, amount_human).await
}

/// Get current USDC allowance for a spender
#[update]
async fn get_usdc_allowance_info(owner_address: Option<String>, spender_address: String) -> Result<String, String> {
    get_usdc_allowance(owner_address, spender_address).await
}

/// Revoke USDC approval (set allowance to 0)
#[update]
async fn revoke_usdc_spending_approval(spender_address: String) -> Result<String, String> {
    revoke_usdc_approval(spender_address).await
}

// --- Message Signing Methods ---

/// Sign an arbitrary message using threshold ECDSA
#[update]
async fn sign_arbitrary_message(message: String) -> Result<String, String> {
    sign_message(message).await
}

/// Sign a message and return both signature and signer address
#[update]
async fn sign_message_with_signer_address(message: String) -> Result<String, String> {
    sign_message_with_address(message).await
}

/// Sign a 32-byte hash directly
#[update]
async fn sign_32_byte_hash(hash_hex: String) -> Result<String, String> {
    sign_hash(hash_hex).await
}

// --- WETH Approve Service Methods ---

/// Get WETH balance
#[update]
async fn get_weth_token_balance(address: Option<String>) -> Result<String, String> {
    get_weth_balance(address).await
}

/// Approve WETH spending for Uniswap V2 Router (simplified)
#[update]
async fn approve_weth_for_uniswap_trading(amount: String) -> Result<String, String> {
    approve_weth_for_uniswap(amount).await
}

/// Approve WETH spending for any address
#[update]
async fn approve_weth_spending(spender_address: String, amount: String) -> Result<String, String> {
    approve_weth(spender_address, amount).await
}

/// Approve WETH spending with human-readable amount (e.g. "1.5" for 1.5 WETH)
#[update]
async fn approve_weth_human_readable(spender_address: String, amount_human: String) -> Result<String, String> {
    approve_weth_human(spender_address, amount_human).await
}

/// Get current WETH allowance for a spender
#[update]
async fn get_weth_allowance_info(owner_address: Option<String>, spender_address: String) -> Result<String, String> {
    get_weth_allowance(owner_address, spender_address).await
}

/// Revoke WETH approval (set allowance to 0)
#[update]
async fn revoke_weth_spending_approval(spender_address: String) -> Result<String, String> {
    revoke_weth_approval(spender_address).await
}

// --- WETH Wrapping/Unwrapping Methods ---

/// Wrap ETH into WETH tokens by depositing ETH
/// Amount should be in Wei format (18 decimals for ETH)
#[update]
async fn wrap_eth_tokens(amount: String) -> Result<String, String> {
    wrap_eth(amount).await
}

/// Wrap ETH with human-readable amount (e.g. "0.1" for 0.1 ETH)
#[update]
async fn wrap_eth_human_readable(amount_human: String) -> Result<String, String> {
    wrap_eth_human(amount_human).await
}

/// Unwrap WETH back to ETH by withdrawing from WETH contract
/// Amount should be in Wei format (18 decimals for WETH)
#[update]
async fn unwrap_weth_tokens(amount: String) -> Result<String, String> {
    unwrap_weth(amount).await
}

/// Unwrap WETH with human-readable amount (e.g. "0.1" for 0.1 WETH)
#[update]
async fn unwrap_weth_human_readable(amount_human: String) -> Result<String, String> {
    unwrap_weth_human(amount_human).await
}

/// Get WETH balance for wrap/unwrap operations
#[update]
async fn get_weth_balance_for_wrapping(address: Option<String>) -> Result<String, String> {
    get_weth_balance(address).await
}

// --- Uniswap Service Methods ---

/// Get estimated quote for WETH → USDC swap
#[update]
async fn get_weth_usdc_quote_human_readable(weth_amount_human: String) -> Result<String, String> {
    get_weth_usdc_quote_v3_human(weth_amount_human).await
}

/// Approve Universal Router to spend WETH for Uniswap V3 swaps
#[update]
async fn approve_weth_for_uniswap_v3(amount_human: String) -> Result<String, String> {
    approve_weth_for_universal_router_human(amount_human).await
}

// --- RPC Service Configuration ---

fn get_rpc_service_sepolia() -> RpcService {
    // Using a custom RPC proxy for Sepolia testnet
    // Note: This is a demo proxy, use your own RPC endpoint in production
    RpcService::Custom(RpcApi {
        url: "https://ic-alloy-evm-rpc-proxy.kristofer-977.workers.dev/eth-sepolia".to_string(),
        headers: None,
    })
}

fn get_ecdsa_key_name() -> String {
    #[allow(clippy::option_env_unwrap)]
    let dfx_network = option_env!("DFX_NETWORK").unwrap_or("local");
    match dfx_network {
        "local" => "dfx_test_key".to_string(),
        "ic" => "key_1".to_string(),
        _ => "dfx_test_key".to_string(), // Default fallback
    }
}

async fn create_icp_signer() -> Result<IcpSigner, String> {
    let user = ic_cdk::caller();
    let derivation_path = vec![user.as_slice().to_vec()];
    let ecdsa_key_name = get_ecdsa_key_name();
    
    IcpSigner::new(derivation_path, &ecdsa_key_name, None)
        .await
        .map_err(|e| format!("Failed to create ICP signer: {}", e))
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
