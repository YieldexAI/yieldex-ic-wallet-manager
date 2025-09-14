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
    permissions::{is_permissions_owner, verify_protocol_permission, add_protocol_permission, set_daily_usage},
    aave::{supply_link_to_aave_with_permissions, withdraw_link_from_aave_with_permissions, get_aave_link_balance, supply_to_aave_with_permissions, withdraw_from_aave_with_permissions}, // 🆕 AAVE Service Methods (Sprint 2)
    compound::{supply_usdc_to_compound_with_permissions, withdraw_usdc_from_compound_with_permissions, get_compound_usdc_balance}, // 🆕 Compound Service Methods
    rebalance::{rebalance_tokens, get_supported_rebalance_routes, get_rebalance_route_status}, // 🆕 Rebalance Service Methods
    rpc_service::{is_supported_chain, get_supported_chains_info} // 🆕 RPC Service imports
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

// 🆕 New type for protocol permissions (Task 1.1)
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct ProtocolPermission {
    pub protocol_address: String,
    pub allowed_functions: Vec<String>, // ["supply", "withdraw", "borrow"]
    pub max_amount_per_tx: Option<u64>,
    pub daily_limit: Option<u64>,
    pub total_used_today: u64,
    pub last_reset_date: u64, // Timestamp for daily limit reset
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
    chain_id: u64,                                  // 🆕 Chain ID for multi-chain support
    whitelisted_protocols: Vec<Protocol>,
    whitelisted_tokens: Vec<Token>,
    transfer_limits: Vec<TransferLimit>,
    protocol_permissions: Vec<ProtocolPermission>, // 🆕 New field (Task 1.1)
    created_at: u64,
    updated_at: u64,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct CreatePermissionsRequest {
    chain_id: u64,                                         // 🆕 Required chain ID
    whitelisted_protocols: Vec<Protocol>,
    whitelisted_tokens: Vec<Token>,
    transfer_limits: Vec<TransferLimit>,
    protocol_permissions: Option<Vec<ProtocolPermission>>, // 🆕 Add protocol permissions
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct UpdatePermissionsRequest {
    permissions_id: PermissionsId,
    chain_id: Option<u64>,                                 // 🆕 Optional chain ID update
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

// Normalize Ethereum address to lowercase without 0x prefix
fn normalize_address(address: &str) -> String {
    address.trim_start_matches("0x").to_lowercase()
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

    ic_cdk::println!("🔑 Starting EVM address generation for principal {}", user);

    // 1. Check if address already exists for the given principal
    ic_cdk::println!("✅ Step 1: Checking existing EVM address...");
    if let Some(storable_address) = PRINCIPAL_TO_ADDRESS_MAP.with(|map| map.borrow().get(&storable_principal)) {
        ic_cdk::println!("✅ Step 1 Complete: Address found for principal {}", user);
        ic_cdk::println!("📋 Existing EVM Address: {}", storable_address.0);
        ic_cdk::println!("🎉 EVM address retrieval completed successfully");
        return Ok(storable_address.0);
    }

    ic_cdk::println!("✅ Step 1 Complete: No existing address found, proceeding with generation");
    ic_cdk::println!("📝 Principal details: {}", user);

    // 2. Generate new address using IcpSigner
    ic_cdk::println!("✅ Step 2: Creating ICP signer with threshold ECDSA...");
    let signer = create_icp_signer().await.map_err(|e| {
        let error_msg = format!("Failed to create ICP signer: {}", e);
        ic_cdk::println!("❌ Step 2 Failed: {}", error_msg);
        error_msg
    })?;
    ic_cdk::println!("✅ Step 2 Complete: ICP signer created successfully");

    // Get the address from the signer
    ic_cdk::println!("✅ Step 3: Deriving EVM address from signer...");
    let address: Address = signer.address();
    let address_hex = format!("0x{:x}", address);
    ic_cdk::println!("✅ Step 3 Complete: EVM address derived: {}", address_hex);

    // 3. Store the new address
    ic_cdk::println!("✅ Step 4: Storing EVM address in stable memory...");
    PRINCIPAL_TO_ADDRESS_MAP.with(|map| {
        map.borrow_mut().insert(storable_principal, StorableString(address_hex.clone()));
    });
    ic_cdk::println!("✅ Step 4 Complete: Address stored successfully");
    
    ic_cdk::println!("📋 Generation Summary:");
    ic_cdk::println!("  - Principal: {}", user);
    ic_cdk::println!("  - Generated Address: {}", address_hex);
    ic_cdk::println!("  - Storage: Stable Memory");
    ic_cdk::println!("  - Derivation: IC Threshold ECDSA");
    
    ic_cdk::println!("🎉 EVM address generation completed successfully: {}", address_hex);

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

#[update]
async fn create_permissions(req: CreatePermissionsRequest) -> Result<Permissions, String> {
    let caller = ic_cdk::caller();
    
    ic_cdk::println!("🔐 Starting permissions creation for principal {}", caller);
    
    // Check if the caller has an EVM address
    ic_cdk::println!("✅ Step 1: Verifying user has EVM address...");
    match verify_user(caller) {
        Ok(true) => {
            ic_cdk::println!("✅ Step 1 Complete: User has valid EVM address");
        },
        Ok(false) => {
            let error_msg = "You must generate an EVM address first".to_string();
            ic_cdk::println!("❌ Step 1 Failed: {}", error_msg);
            return Err(error_msg);
        },
        Err(e) => {
            ic_cdk::println!("❌ Step 1 Failed: {}", e);
            return Err(e);
        },
    }
    
    // Generate unique permissions ID
    ic_cdk::println!("✅ Step 2: Generating unique permissions ID...");
    let permissions_id = generate_permissions_id().await;
    ic_cdk::println!("✅ Step 2 Complete: Generated permissions ID: {}", permissions_id);
    
    // Get current timestamp
    ic_cdk::println!("✅ Step 3: Setting timestamps...");
    let timestamp = now();
    ic_cdk::println!("✅ Step 3 Complete: Timestamp set to: {}", timestamp);
    
    // Validate chain_id
    ic_cdk::println!("✅ Step 4: Validating chain_id...");
    if !is_supported_chain(req.chain_id) {
        let error_msg = format!("Unsupported chain_id: {}. Supported chains: {:?}", 
                               req.chain_id, get_supported_chains_info());
        ic_cdk::println!("❌ Step 4 Failed: {}", error_msg);
        return Err(error_msg);
    }
    ic_cdk::println!("✅ Step 4 Complete: Chain ID {} is supported", req.chain_id);
    
    // Log request details
    ic_cdk::println!("📋 Permission Request Details:");
    ic_cdk::println!("  - Chain ID: {}", req.chain_id);
    ic_cdk::println!("  - Whitelisted Protocols: {} protocols", req.whitelisted_protocols.len());
    for (i, protocol) in req.whitelisted_protocols.iter().enumerate() {
        ic_cdk::println!("    {}. {} ({})", i + 1, protocol.name, protocol.address);
    }
    ic_cdk::println!("  - Whitelisted Tokens: {} tokens", req.whitelisted_tokens.len());
    for (i, token) in req.whitelisted_tokens.iter().enumerate() {
        ic_cdk::println!("    {}. {} ({})", i + 1, token.name, token.address);
    }
    ic_cdk::println!("  - Transfer Limits: {} limits", req.transfer_limits.len());
    for (i, limit) in req.transfer_limits.iter().enumerate() {
        ic_cdk::println!("    {}. Token {} - Daily: {}, Max TX: {}", 
                        i + 1, limit.token_address, limit.daily_limit, limit.max_tx_amount);
    }
    
    let empty_vec = vec![];
    let protocol_perms = req.protocol_permissions.as_ref().unwrap_or(&empty_vec);
    ic_cdk::println!("  - Protocol Permissions: {} permissions", protocol_perms.len());
    for (i, perm) in protocol_perms.iter().enumerate() {
        ic_cdk::println!("    {}. Protocol {} - Functions: {:?}", 
                        i + 1, perm.protocol_address, perm.allowed_functions);
        if let Some(max_tx) = perm.max_amount_per_tx {
            ic_cdk::println!("       Max TX: {}", max_tx);
        }
        if let Some(daily) = perm.daily_limit {
            ic_cdk::println!("       Daily Limit: {}", daily);
        }
    }
    
    // Create permissions struct with normalized addresses
    ic_cdk::println!("✅ Step 5: Creating permissions structure...");

    // Normalize protocol addresses
    let normalized_protocols = req.whitelisted_protocols.into_iter().map(|mut p| {
        p.address = normalize_address(&p.address);
        p
    }).collect();

    // Normalize token addresses
    let normalized_tokens = req.whitelisted_tokens.into_iter().map(|mut t| {
        t.address = normalize_address(&t.address);
        t
    }).collect();

    // Normalize transfer limit addresses
    let normalized_limits = req.transfer_limits.into_iter().map(|mut l| {
        l.token_address = normalize_address(&l.token_address);
        l
    }).collect();

    // Normalize protocol permission addresses
    let normalized_protocol_permissions = req.protocol_permissions.unwrap_or_default()
        .into_iter().map(|mut pp| {
            pp.protocol_address = normalize_address(&pp.protocol_address);
            pp
        }).collect();

    let permissions = Permissions {
        id: permissions_id.clone(),
        owner: caller,
        chain_id: req.chain_id,
        whitelisted_protocols: normalized_protocols,
        whitelisted_tokens: normalized_tokens,
        transfer_limits: normalized_limits,
        protocol_permissions: normalized_protocol_permissions,
        created_at: timestamp,
        updated_at: timestamp,
    };
    ic_cdk::println!("✅ Step 4 Complete: Permissions structure created");
    
    // Store in stable memory
    ic_cdk::println!("✅ Step 5: Storing permissions in stable memory...");
    PERMISSIONS_MAP.with(|map| {
        map.borrow_mut().insert(
            StorableString(permissions_id.clone()), 
            StorablePermissions(permissions.clone())
        );
    });
    ic_cdk::println!("✅ Step 5 Complete: Permissions stored successfully");
    
    ic_cdk::println!("📋 Creation Summary:");
    ic_cdk::println!("  - Permissions ID: {}", permissions_id);
    ic_cdk::println!("  - Owner: {}", caller);
    ic_cdk::println!("  - Total Protocols: {}", permissions.whitelisted_protocols.len());
    ic_cdk::println!("  - Total Tokens: {}", permissions.whitelisted_tokens.len());
    ic_cdk::println!("  - Total Limits: {}", permissions.transfer_limits.len());
    ic_cdk::println!("  - Total Protocol Permissions: {}", permissions.protocol_permissions.len());
    ic_cdk::println!("  - Created At: {}", timestamp);
    
    ic_cdk::println!("🎉 Permissions creation completed successfully: {}", permissions_id);
    
    Ok(permissions)
}

#[query]
fn get_permissions(permissions_id: String) -> Result<Permissions, String> {
    let caller = ic_cdk::caller();
    
    ic_cdk::println!("🔍 Starting permissions retrieval for ID: {}", permissions_id);
    ic_cdk::println!("📝 Requested by principal: {}", caller);
    
    let result = PERMISSIONS_MAP.with(|map| {
        map.borrow()
            .get(&StorableString(permissions_id.clone()))
            .map_or_else(
                || {
                    let error_msg = format!("Permissions with ID {} not found", permissions_id);
                    ic_cdk::println!("❌ Permissions not found: {}", error_msg);
                    Err(error_msg)
                },
                |p| {
                    if p.0.owner == caller {
                        ic_cdk::println!("✅ Permissions found and ownership verified");
                        ic_cdk::println!("📋 Permissions Details:");
                        ic_cdk::println!("  - ID: {}", p.0.id);
                        ic_cdk::println!("  - Owner: {}", p.0.owner);
                        ic_cdk::println!("  - Protocols: {}", p.0.whitelisted_protocols.len());
                        ic_cdk::println!("  - Tokens: {}", p.0.whitelisted_tokens.len());
                        ic_cdk::println!("  - Transfer Limits: {}", p.0.transfer_limits.len());
                        ic_cdk::println!("  - Protocol Permissions: {}", p.0.protocol_permissions.len());
                        ic_cdk::println!("  - Created At: {}", p.0.created_at);
                        ic_cdk::println!("  - Updated At: {}", p.0.updated_at);
                        ic_cdk::println!("🎉 Permissions retrieval completed successfully");
                        Ok(p.0)
                    } else {
                        let error_msg = "You do not have permission to access these permissions".to_string();
                        ic_cdk::println!("❌ Access denied: {}", error_msg);
                        ic_cdk::println!("  - Requested by: {}", caller);
                        ic_cdk::println!("  - Actual owner: {}", p.0.owner);
                        Err(error_msg)
                    }
                }
            )
    });
    
    result
}

#[query]
fn get_all_permissions() -> Result<Vec<Permissions>, String> {
    let caller = ic_cdk::caller();
    
    ic_cdk::println!("📋 Starting retrieval of all permissions for principal: {}", caller);
    
    // Collect all permissions owned by the caller
    let mut result = Vec::new();
    
    PERMISSIONS_MAP.with(|map| {
        let borrowed_map = map.borrow();
        
        ic_cdk::println!("✅ Step 1: Scanning permissions database...");
        let total_permissions = borrowed_map.len();
        ic_cdk::println!("  - Total permissions in database: {}", total_permissions);
        
        // Iterate through all permissions and filter by owner
        for (id, permissions) in borrowed_map.iter() {
            if permissions.0.owner == caller {
                ic_cdk::println!("  - Found owned permission: {}", id.0);
                result.push(permissions.0.clone());
            }
        }
        
        ic_cdk::println!("✅ Step 1 Complete: Database scan finished");
    });
    
    ic_cdk::println!("📋 Retrieval Summary:");
    ic_cdk::println!("  - Requested by: {}", caller);
    ic_cdk::println!("  - Owned permissions found: {}", result.len());
    
    // Log each permission briefly
    for (i, perm) in result.iter().enumerate() {
        ic_cdk::println!("  {}. ID: {} (Created: {}, Protocols: {}, Tokens: {})", 
                        i + 1, perm.id, perm.created_at, perm.whitelisted_protocols.len(), perm.whitelisted_tokens.len());
    }
    
    ic_cdk::println!("🎉 All permissions retrieval completed successfully: {} permissions found", result.len());
    
    Ok(result)
}

#[update]
fn update_permissions(req: UpdatePermissionsRequest) -> Result<Permissions, String> {
    let caller = ic_cdk::caller();
    let permissions_id = req.permissions_id.clone();
    
    ic_cdk::println!("🔄 Starting permissions update for ID: {}", permissions_id);
    ic_cdk::println!("📝 Requested by principal: {}", caller);
    
    // Check if permissions exist and caller is the owner
    ic_cdk::println!("✅ Step 1: Verifying permissions ownership...");
    if let Err(e) = is_permissions_owner(&permissions_id, caller) {
        ic_cdk::println!("❌ Step 1 Failed: {}", e);
        return Err(e);
    }
    ic_cdk::println!("✅ Step 1 Complete: Ownership verified");
    
    // Get the existing permissions
    ic_cdk::println!("✅ Step 2: Loading existing permissions...");
    let mut permissions = PERMISSIONS_MAP.with(|map| {
        map.borrow()
            .get(&StorableString(permissions_id.clone()))
            .unwrap().0.clone()
    });
    ic_cdk::println!("✅ Step 2 Complete: Existing permissions loaded");
    
    // Log current state
    ic_cdk::println!("📋 Current State:");
    ic_cdk::println!("  - Chain ID: {}", permissions.chain_id);
    ic_cdk::println!("  - Protocols: {}", permissions.whitelisted_protocols.len());
    ic_cdk::println!("  - Tokens: {}", permissions.whitelisted_tokens.len());
    ic_cdk::println!("  - Transfer Limits: {}", permissions.transfer_limits.len());
    ic_cdk::println!("  - Protocol Permissions: {}", permissions.protocol_permissions.len());
    ic_cdk::println!("  - Last Updated: {}", permissions.updated_at);
    
    // Update fields if provided
    ic_cdk::println!("✅ Step 3: Applying updates...");
    let mut changes_made = 0;
    
    if let Some(chain_id) = req.chain_id {
        ic_cdk::println!("🔄 Updating chain_id: {} -> {}", permissions.chain_id, chain_id);
        if !is_supported_chain(chain_id) {
            let error_msg = format!("Unsupported chain_id: {}. Supported chains: {:?}", 
                                   chain_id, get_supported_chains_info());
            ic_cdk::println!("❌ Chain ID validation failed: {}", error_msg);
            return Err(error_msg);
        }
        permissions.chain_id = chain_id;
        changes_made += 1;
    }
    
    if let Some(protocols) = req.whitelisted_protocols {
        ic_cdk::println!("🔄 Updating whitelisted protocols: {} -> {}", 
                        permissions.whitelisted_protocols.len(), protocols.len());
        for (i, protocol) in protocols.iter().enumerate() {
            ic_cdk::println!("    {}. {} ({})", i + 1, protocol.name, protocol.address);
        }
        permissions.whitelisted_protocols = protocols;
        changes_made += 1;
    }
    
    if let Some(tokens) = req.whitelisted_tokens {
        ic_cdk::println!("🔄 Updating whitelisted tokens: {} -> {}", 
                        permissions.whitelisted_tokens.len(), tokens.len());
        for (i, token) in tokens.iter().enumerate() {
            ic_cdk::println!("    {}. {} ({})", i + 1, token.name, token.address);
        }
        permissions.whitelisted_tokens = tokens;
        changes_made += 1;
    }
    
    if let Some(limits) = req.transfer_limits {
        ic_cdk::println!("🔄 Updating transfer limits: {} -> {}", 
                        permissions.transfer_limits.len(), limits.len());
        for (i, limit) in limits.iter().enumerate() {
            ic_cdk::println!("    {}. Token {} - Daily: {}, Max TX: {}", 
                            i + 1, limit.token_address, limit.daily_limit, limit.max_tx_amount);
        }
        permissions.transfer_limits = limits;
        changes_made += 1;
    }
    
    ic_cdk::println!("✅ Step 3 Complete: {} field(s) updated", changes_made);
    
    // Update the timestamp
    ic_cdk::println!("✅ Step 4: Updating timestamp...");
    let new_timestamp = now();
    permissions.updated_at = new_timestamp;
    ic_cdk::println!("✅ Step 4 Complete: Timestamp updated to {}", new_timestamp);
    
    // Save the updated permissions
    ic_cdk::println!("✅ Step 5: Saving updated permissions to stable memory...");
    PERMISSIONS_MAP.with(|map| {
        map.borrow_mut().insert(
            StorableString(permissions_id.clone()), 
            StorablePermissions(permissions.clone())
        );
    });
    ic_cdk::println!("✅ Step 5 Complete: Permissions saved successfully");
    
    ic_cdk::println!("📋 Update Summary:");
    ic_cdk::println!("  - Permissions ID: {}", permissions_id);
    ic_cdk::println!("  - Changes Made: {}", changes_made);
    ic_cdk::println!("  - Updated By: {}", caller);
    ic_cdk::println!("  - New Timestamp: {}", new_timestamp);
    ic_cdk::println!("  - Total Protocols: {}", permissions.whitelisted_protocols.len());
    ic_cdk::println!("  - Total Tokens: {}", permissions.whitelisted_tokens.len());
    ic_cdk::println!("  - Total Limits: {}", permissions.transfer_limits.len());
    
    ic_cdk::println!("🎉 Permissions update completed successfully: {}", permissions_id);
    
    Ok(permissions)
}

#[update]
fn delete_permissions(permissions_id: String) -> Result<bool, String> {
    let caller = ic_cdk::caller();
    
    ic_cdk::println!("🗑️ Starting permissions deletion for ID: {}", permissions_id);
    ic_cdk::println!("📝 Requested by principal: {}", caller);
    
    // Check if permissions exist and caller is the owner
    ic_cdk::println!("✅ Step 1: Verifying permissions ownership...");
    if let Err(e) = is_permissions_owner(&permissions_id, caller) {
        ic_cdk::println!("❌ Step 1 Failed: {}", e);
        return Err(e);
    }
    ic_cdk::println!("✅ Step 1 Complete: Ownership verified");
    
    // Log details before deletion
    ic_cdk::println!("✅ Step 2: Retrieving permissions details before deletion...");
    let permissions_details = PERMISSIONS_MAP.with(|map| {
        map.borrow()
            .get(&StorableString(permissions_id.clone()))
            .map(|p| p.0.clone())
    });
    
    if let Some(perms) = permissions_details {
        ic_cdk::println!("📋 Permissions to be deleted:");
        ic_cdk::println!("  - ID: {}", perms.id);
        ic_cdk::println!("  - Owner: {}", perms.owner);
        ic_cdk::println!("  - Protocols: {}", perms.whitelisted_protocols.len());
        ic_cdk::println!("  - Tokens: {}", perms.whitelisted_tokens.len());
        ic_cdk::println!("  - Transfer Limits: {}", perms.transfer_limits.len());
        ic_cdk::println!("  - Protocol Permissions: {}", perms.protocol_permissions.len());
        ic_cdk::println!("  - Created At: {}", perms.created_at);
        ic_cdk::println!("  - Updated At: {}", perms.updated_at);
        ic_cdk::println!("✅ Step 2 Complete: Details retrieved");
    } else {
        ic_cdk::println!("⚠️ Step 2 Warning: Could not retrieve details (permissions may already be deleted)");
    }
    
    // Delete the permissions
    ic_cdk::println!("✅ Step 3: Deleting permissions from stable memory...");
    let removed = PERMISSIONS_MAP.with(|map| {
        map.borrow_mut().remove(&StorableString(permissions_id.clone())).is_some()
    });
    
    if removed {
        ic_cdk::println!("✅ Step 3 Complete: Permissions successfully deleted");
        
        ic_cdk::println!("📋 Deletion Summary:");
        ic_cdk::println!("  - Permissions ID: {}", permissions_id);
        ic_cdk::println!("  - Deleted By: {}", caller);
        ic_cdk::println!("  - Deletion Time: {}", now());
        ic_cdk::println!("  - Status: Successfully Removed");
        
        ic_cdk::println!("🎉 Permissions deletion completed successfully: {}", permissions_id);
        Ok(true)
    } else {
        let error_msg = "Failed to delete permissions (not found)".to_string();
        ic_cdk::println!("❌ Step 3 Failed: {}", error_msg);
        Err(error_msg)
    }
}

// 🆕 New functions for protocol permissions (Task 1.1) - use permissions service

/// Check permission to perform protocol operation
#[query]
fn check_protocol_permission(
    permissions_id: String, 
    protocol_address: String, 
    function_name: String,
    amount: u64
) -> Result<bool, String> {
    let caller = ic_cdk::caller();
    verify_protocol_permission(permissions_id, protocol_address, function_name, amount, caller)
}

/// Add permission for protocol
#[update] 
fn update_protocol_permission(
    permissions_id: String,
    protocol_permission: ProtocolPermission
) -> Result<bool, String> {
    let caller = ic_cdk::caller();
    
    ic_cdk::println!("🔧 Starting protocol permission update for permissions ID: {}", permissions_id);
    ic_cdk::println!("📝 Requested by principal: {}", caller);
    
    ic_cdk::println!("📋 Protocol Permission Details:");
    ic_cdk::println!("  - Protocol Address: {}", protocol_permission.protocol_address);
    ic_cdk::println!("  - Allowed Functions: {:?}", protocol_permission.allowed_functions);
    if let Some(max_tx) = protocol_permission.max_amount_per_tx {
        ic_cdk::println!("  - Max TX Amount: {}", max_tx);
    } else {
        ic_cdk::println!("  - Max TX Amount: No limit");
    }
    if let Some(daily) = protocol_permission.daily_limit {
        ic_cdk::println!("  - Daily Limit: {}", daily);
    } else {
        ic_cdk::println!("  - Daily Limit: No limit");
    }
    ic_cdk::println!("  - Total Used Today: {}", protocol_permission.total_used_today);
    ic_cdk::println!("  - Last Reset Date: {}", protocol_permission.last_reset_date);
    
    ic_cdk::println!("✅ Step 1: Calling protocol permission service...");
    let result = add_protocol_permission(permissions_id.clone(), protocol_permission, caller);
    
    match &result {
        Ok(success) => {
            ic_cdk::println!("✅ Step 1 Complete: Protocol permission service succeeded");
            ic_cdk::println!("📋 Update Summary:");
            ic_cdk::println!("  - Permissions ID: {}", permissions_id);
            ic_cdk::println!("  - Updated By: {}", caller);
            ic_cdk::println!("  - Result: {}", success);
            ic_cdk::println!("  - Timestamp: {}", now());
            ic_cdk::println!("🎉 Protocol permission update completed successfully");
        }
        Err(error) => {
            ic_cdk::println!("❌ Step 1 Failed: {}", error);
        }
    }
    
    result
}

/// Update used limit for today
#[update]
fn update_daily_usage(
    permissions_id: String,
    protocol_address: String,
    amount_used: u64
) -> Result<bool, String> {
    let caller = ic_cdk::caller();
    set_daily_usage(permissions_id, protocol_address, amount_used, caller)
}

// --- Balance Service Methods ---

/// Get ETH balance for an address (or current user's address if none provided)
#[update]
async fn get_eth_balance(address: Option<String>) -> Result<String, String> {
    get_balance(address).await
}

/// Get USDC balance for an address (or current user's address if none provided)
#[update]
async fn get_usdc_balance(address: Option<String>, chain_id: u64) -> Result<String, String> {
    get_balance_usdc(address, chain_id).await
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

// --- AAVE Service Methods (Sprint 2) ---

/// Supply LINK to AAVE with permission verification
#[update]
async fn supply_link_to_aave_secured(
    amount_human: String, 
    permissions_id: String
) -> Result<String, String> {
    let caller = ic_cdk::caller();
    supply_link_to_aave_with_permissions(amount_human, permissions_id, caller).await
}

/// Withdraw LINK from AAVE with permission verification
#[update]
async fn withdraw_link_from_aave_secured(
    amount_human: String, 
    permissions_id: String
) -> Result<String, String> {
    let caller = ic_cdk::caller();
    withdraw_link_from_aave_with_permissions(amount_human, permissions_id, caller).await
}

/// Supply any token to AAVE with permission verification
#[update]
async fn supply_to_aave_secured(
    amount_human: String,
    permissions_id: String,
    token_address: String,
    token_symbol: String,
) -> Result<String, String> {
    let caller = ic_cdk::caller();
    let perminissions = get_permissions(permissions_id.clone());
    let chain_id = perminissions.unwrap().chain_id;
    let token_address = token_address.parse().map_err(|_| "Invalid token address format")?;
    supply_to_aave_with_permissions(token_address, token_symbol, amount_human, permissions_id, caller, chain_id).await
}

/// Withdraw any token from AAVE with permission verification
#[update]
async fn withdraw_from_aave_secured(
    amount_human: String,
    permissions_id: String,
    token_address: String,
    token_symbol: String,
) -> Result<String, String> {
    let caller = ic_cdk::caller();
    let perminissions = get_permissions(permissions_id.clone());
    let chain_id = perminissions.unwrap().chain_id;
    let token_address = token_address.parse().map_err(|_| "Invalid token address format")?;
    withdraw_from_aave_with_permissions(token_address, token_symbol, amount_human, permissions_id, caller, chain_id).await
}


/// Get user's aLINK balance in AAVE
#[update]
async fn get_aave_link_user_balance(address: Option<String>) -> Result<String, String> {
    get_aave_link_balance(address).await
}

// --- Compound Service Methods ---

/// Supply USDC to Compound with permission verification
#[update]
async fn supply_usdc_to_compound_secured(
    amount_human: String, 
    permissions_id: String
) -> Result<String, String> {
    let caller = ic_cdk::caller();
    supply_usdc_to_compound_with_permissions(amount_human, permissions_id, caller).await
}

/// Withdraw USDC from Compound with permission verification
#[update]
async fn withdraw_usdc_from_compound_secured(
    amount_human: String, 
    permissions_id: String
) -> Result<String, String> {
    let caller = ic_cdk::caller();
    withdraw_usdc_from_compound_with_permissions(amount_human, permissions_id, caller).await
}

/// Get user's USDC balance in Compound
#[update]
async fn get_compound_usdc_user_balance(address: Option<String>, chain_id: u64) -> Result<String, String> {
    get_compound_usdc_balance(address, chain_id).await
}

// --- Rebalance Service Methods ---

/// Rebalance tokens between DeFi protocols
#[update]
async fn rebalance_tokens_secured(
    amount: String,
    source_protocol: String,
    target_protocol: String,
    token: String,
    permissions_id: String
) -> Result<String, String> {
    let caller = ic_cdk::caller();
    rebalance_tokens(amount, source_protocol, target_protocol, token, permissions_id, caller).await
}

/// Get supported rebalance routes for a specific chain
#[query]
fn get_supported_rebalance_routes_query(chain_id: u64) -> Vec<(String, String, String)> {
    get_supported_rebalance_routes(chain_id)
}

/// Check rebalance route status for a specific chain
#[query]
fn check_rebalance_route_status(
    source_protocol: String,
    target_protocol: String,
    token: String,
    chain_id: u64
) -> String {
    get_rebalance_route_status(&source_protocol, &target_protocol, &token, chain_id)
}

/// Get supported protocol-token combinations for a specific chain
#[query]
fn get_protocol_token_support_query(chain_id: u64) -> Vec<(String, String)> {
    services::rebalance::get_protocol_token_support(chain_id)
}

// --- Uniswap Service Methods ---

// /// Get estimated quote for WETH → USDC swap
// #[update]
// async fn get_weth_usdc_quote_human_readable(weth_amount_human: String) -> Result<String, String> {
//     get_weth_usdc_quote_v3_human(weth_amount_human).await
// }

// /// Approve Universal Router to spend WETH for Uniswap V3 swaps
// #[update]
// async fn approve_weth_for_uniswap_v3(amount_human: String) -> Result<String, String> {
//     approve_weth_for_universal_router_human(amount_human).await
// }

// --- Chain Support API ---

/// Get list of supported chains
#[query]
fn get_supported_chains() -> Vec<(u64, String)> {
    get_supported_chains_info()
        .into_iter()
        .map(|(id, name)| (id, name.to_string()))
        .collect()
}

/// Check if a chain is supported
#[query]
fn is_chain_supported(chain_id: u64) -> bool {
    is_supported_chain(chain_id)
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
