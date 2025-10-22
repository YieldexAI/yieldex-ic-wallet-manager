use candid::Principal;
use ic_cdk_macros::{init, post_upgrade, query, update};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;

// Alloy imports
use alloy::signers::icp::IcpSigner;
use alloy::signers::Signer; // The Signer trait
use alloy::primitives::Address;

// Types module
mod types;
use types::{
    Permissions, CreatePermissionsRequest, UpdatePermissionsRequest,
    ProtocolPermission, Recommendation, ExecutionResult,
    StorablePrincipal, StorableString, StorablePermissions,
    StorableUserPosition, StorableApyHistoryRecord, StorableRebalanceExecution,
    ProtocolApyInfo, ApyResponse, ApyParserStatus,
    SchedulerConfig, SchedulerStatus, RebalanceExecution,
    UserPosition, ApyHistoryRecord, // 🆕 APY Parser types
};

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
    aave::{supply_link_to_aave_with_permissions, withdraw_link_from_aave_with_permissions, get_aave_link_balance, supply_to_aave_with_permissions, withdraw_from_aave_with_permissions, get_apy as get_aave_apy}, // 🆕 AAVE Service Methods (Sprint 2)
    compound::{supply_usdc_to_compound_with_permissions, withdraw_usdc_from_compound_with_permissions, get_compound_usdc_balance, get_apy as get_compound_apy}, // 🆕 Compound Service Methods
    rebalance::{execute_recommendation as execute_recommendation_impl, validate_recommendation}, // 🆕 Rebalance Service Methods
    rpc_service::{is_supported_chain, get_supported_chains_info}, // 🆕 RPC Service imports
    scheduler, // 🆕 Scheduler module
    apy_parser, // 🆕 APY Parser module
};

// --- Types ---
type Memory = VirtualMemory<DefaultMemoryImpl>;

// --- State ---
const PRINCIPAL_MAP_MEMORY_ID: MemoryId = MemoryId::new(0);
const PERMISSIONS_MAP_MEMORY_ID: MemoryId = MemoryId::new(1);
const APY_HISTORY_MEMORY_ID: MemoryId = MemoryId::new(2);
const USER_POSITIONS_MEMORY_ID: MemoryId = MemoryId::new(3);
const REBALANCE_HISTORY_MEMORY_ID: MemoryId = MemoryId::new(4);

// Admin principals - hardcoded list of authorized administrators
const ADMIN_PRINCIPALS: &[&str] = &[
    "hfugy-ahqdz-5sbki-vky4l-xceci-3se5z-2cb7k-jxjuq-qidax-gd53f-nqe", // Your principal
];

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

    // Map RecordId -> APY History Record
    pub static APY_HISTORY_MAP: RefCell<StableBTreeMap<StorableString, StorableApyHistoryRecord, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(APY_HISTORY_MEMORY_ID)),
        )
    );

    // Map PositionId -> User Position
    pub static USER_POSITIONS_MAP: RefCell<StableBTreeMap<StorableString, StorableUserPosition, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(USER_POSITIONS_MEMORY_ID)),
        )
    );

    // Map ExecutionId -> Rebalance Execution
    pub static REBALANCE_HISTORY_MAP: RefCell<StableBTreeMap<StorableString, StorableRebalanceExecution, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(REBALANCE_HISTORY_MEMORY_ID)),
        )
    );
}

// --- Helper Functions ---

// Get current timestamp in milliseconds
fn now() -> u64 {
    // Returns milliseconds since Unix epoch
    ic_cdk::api::time() / 1_000_000
}

// Check if caller is an admin
fn is_admin() -> Result<(), String> {
    let caller = ic_cdk::caller();
    let caller_str = caller.to_text();

    if ADMIN_PRINCIPALS.contains(&caller_str.as_str()) {
        Ok(())
    } else {
        Err(format!("Unauthorized: Only admins can call this function. Caller: {}", caller_str))
    }
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

    if let Some(protocol_perms) = req.protocol_permissions {
        ic_cdk::println!("🔄 Updating protocol permissions: {} -> {}",
                        permissions.protocol_permissions.len(), protocol_perms.len());
        for (i, perm) in protocol_perms.iter().enumerate() {
            ic_cdk::println!("    {}. Protocol {} - Functions: {:?}",
                            i + 1, perm.protocol_address, perm.allowed_functions);
            if let Some(max_tx) = perm.max_amount_per_tx {
                ic_cdk::println!("       Max per TX: {}", max_tx);
            }
            if let Some(daily) = perm.daily_limit {
                ic_cdk::println!("       Daily limit: {}", daily);
            }
        }

        // Normalize protocol addresses in protocol_permissions
        let normalized_protocol_permissions = protocol_perms
            .into_iter()
            .map(|mut pp| {
                pp.protocol_address = normalize_address(&pp.protocol_address);
                pp
            })
            .collect();

        permissions.protocol_permissions = normalized_protocol_permissions;
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

/// Execute recommendation for rebalancing
#[update]
async fn execute_recommendation(
    recommendation: Recommendation,
    permissions_id: String
) -> Result<ExecutionResult, String> {
    let caller = ic_cdk::caller();
    execute_recommendation_impl(recommendation, permissions_id, caller).await
}

/// Validate recommendation without executing
#[query]
fn validate_recommendation_input(
    recommendation: Recommendation
) -> Result<String, String> {
    validate_recommendation(&recommendation)?;
    Ok("Recommendation is valid".to_string())
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

// --- Admin API ---

/// Helper function to get token address from symbol or address string
fn resolve_token_address(token: &str, chain_id: u64) -> Result<Address, String> {
    // Check if it's already an address (starts with 0x and correct length)
    if token.starts_with("0x") && token.len() == 42 {
        token.parse::<Address>()
            .map_err(|_| format!("Invalid token address format: {}", token))
    } else {
        // Resolve common token symbols to addresses based on chain_id
        use services::rpc_service::{SEPOLIA_CHAIN_ID, ARBITRUM_CHAIN_ID, BASE_CHAIN_ID, OPTIMISM_CHAIN_ID};

        let token_upper = token.to_uppercase();
        match (token_upper.as_str(), chain_id) {
            // USDC addresses
            ("USDC", ARBITRUM_CHAIN_ID) => "0xaf88d065e77c8cc2239327c5edb3a432268e5831".parse().map_err(|_| "Parse error".to_string()),
            ("USDC", BASE_CHAIN_ID) => "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".parse().map_err(|_| "Parse error".to_string()),
            ("USDC", OPTIMISM_CHAIN_ID) => "0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85".parse().map_err(|_| "Parse error".to_string()),
            ("USDC", SEPOLIA_CHAIN_ID) => "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238".parse().map_err(|_| "Parse error".to_string()),


            _ => Err(format!("Token '{}' not supported on chain_id {}", token, chain_id))
        }
    }
}

/// Get current APY rates for a token across AAVE and Compound protocols (Admin only)
///
/// # Arguments
/// * `token` - Token address (e.g., "0xaf88d065e77c8cc2239327c5edb3a432268e5831") or symbol (e.g., "USDC")
/// * `chain_id` - Chain ID to check rates on (e.g., 42161 for Arbitrum)
///
/// # Returns
/// ApyResponse containing rates from both AAVE and Compound where available
#[update]
async fn get_current_apy(token: String, chain_id: u64) -> Result<ApyResponse, String> {
    // Check admin access
    is_admin()?;

    ic_cdk::println!("🔍 [ADMIN] Getting APY rates for token '{}' on chain_id {}", token, chain_id);
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    // Validate chain is supported
    if !is_supported_chain(chain_id) {
        return Err(format!("Unsupported chain_id: {}. Supported chains: {:?}",
                          chain_id, get_supported_chains_info()));
    }

    // Resolve token address
    let token_address = resolve_token_address(&token, chain_id)?;
    ic_cdk::println!("✅ Resolved token to address: 0x{:x}", token_address);

    let mut rates = Vec::new();

    // Try to get AAVE APY
    ic_cdk::println!("📊 Fetching AAVE APY...");
    match get_aave_apy(token_address, chain_id).await {
        Ok(apy) => {
            ic_cdk::println!("✅ AAVE APY: {}%", apy);
            rates.push(ProtocolApyInfo {
                protocol: "AAVE".to_string(),
                apy,
                chain_id,
            });
        }
        Err(e) => {
            ic_cdk::println!("⚠️ AAVE APY not available: {}", e);
        }
    }

    // Try to get Compound APY (only on Arbitrum for USDC market)
    if chain_id == services::rpc_service::ARBITRUM_CHAIN_ID {
        ic_cdk::println!("📊 Fetching Compound APY...");
        match get_compound_apy(chain_id).await {
            Ok(apy) => {
                ic_cdk::println!("✅ Compound APY: {}%", apy);
                rates.push(ProtocolApyInfo {
                    protocol: "Compound".to_string(),
                    apy,
                    chain_id,
                });
            }
            Err(e) => {
                ic_cdk::println!("⚠️ Compound APY not available: {}", e);
            }
        }
    } else {
        ic_cdk::println!("ℹ️ Compound is only available on Arbitrum (chain_id: 42161)");
    }

    if rates.is_empty() {
        return Err(format!("No APY rates available for token '{}' on chain_id {}", token, chain_id));
    }

    ic_cdk::println!("🎉 Successfully retrieved {} APY rate(s)", rates.len());

    Ok(ApyResponse {
        token: format!("0x{:x}", token_address),
        chain_id,
        rates,
    })
}

// --- Scheduler Admin API ---

/// Initialize scheduler (Admin only) - for existing canisters that were deployed before scheduler
#[update]
fn admin_init_scheduler() -> Result<String, String> {
    is_admin()?;
    ic_cdk::println!("🔧 [ADMIN] Manually initializing scheduler");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    // Check if already initialized
    if let Ok(_config) = scheduler::get_scheduler_config() {
        return Err("Scheduler already initialized. Use admin_update_scheduler_config to modify.".to_string());
    }

    scheduler::init_scheduler();
    Ok("Scheduler initialized successfully. Use admin_start_scheduler to enable.".to_string())
}

/// Get current scheduler configuration (Admin only)
#[query]
fn admin_get_scheduler_config() -> Result<SchedulerConfig, String> {
    is_admin()?;
    ic_cdk::println!("🔍 [ADMIN] Getting scheduler configuration");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    scheduler::get_scheduler_config()
}

/// Update scheduler configuration (Admin only)
#[update]
fn admin_update_scheduler_config(config: SchedulerConfig) -> Result<SchedulerConfig, String> {
    is_admin()?;
    ic_cdk::println!("🔧 [ADMIN] Updating scheduler configuration");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    scheduler::update_scheduler_config(config)
}

/// Start the scheduler (Admin only)
#[update]
fn admin_start_scheduler() -> Result<String, String> {
    is_admin()?;
    ic_cdk::println!("▶️ [ADMIN] Starting scheduler");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    scheduler::enable_scheduler()
}

/// Stop the scheduler (Admin only)
#[update]
fn admin_stop_scheduler() -> Result<String, String> {
    is_admin()?;
    ic_cdk::println!("⏸️ [ADMIN] Stopping scheduler");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    scheduler::disable_scheduler()
}

/// Set scheduler interval in seconds (Admin only)
#[update]
fn admin_set_scheduler_interval(seconds: u64) -> Result<String, String> {
    is_admin()?;
    ic_cdk::println!("⏱️ [ADMIN] Setting scheduler interval to {} seconds", seconds);
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    scheduler::set_scheduler_interval(seconds)
}

/// Set APY threshold percentage (Admin only)
#[update]
fn admin_set_apy_threshold(percent: f64) -> Result<String, String> {
    is_admin()?;
    ic_cdk::println!("📊 [ADMIN] Setting APY threshold to {}%", percent);
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    scheduler::set_apy_threshold(percent)
}

/// Manually trigger scheduler execution (Admin only)
#[update]
async fn admin_trigger_rebalance() -> Result<Vec<RebalanceExecution>, String> {
    is_admin()?;
    ic_cdk::println!("🔨 [ADMIN] Manually triggering scheduler execution");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    scheduler::trigger_manual_execution().await
}

/// Get scheduler status and statistics (Admin only)
#[query]
fn admin_get_scheduler_status() -> Result<SchedulerStatus, String> {
    is_admin()?;
    ic_cdk::println!("📊 [ADMIN] Getting scheduler status");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    scheduler::get_scheduler_status()
}

/// Get rebalance execution history (Admin only)
#[query]
fn admin_get_rebalance_history(limit: Option<u64>) -> Result<Vec<RebalanceExecution>, String> {
    is_admin()?;
    ic_cdk::println!("📜 [ADMIN] Getting rebalance history (limit: {:?})", limit);
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    Ok(scheduler::get_rebalance_history(limit))
}

/// Get rebalance history for a specific user (Admin only)
#[query]
fn admin_get_user_rebalance_history(user: Principal, limit: Option<u64>) -> Result<Vec<RebalanceExecution>, String> {
    is_admin()?;
    ic_cdk::println!("📜 [ADMIN] Getting rebalance history for user: {}", user);
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    Ok(scheduler::get_user_rebalance_history(user, limit))
}

/// Clear all rebalance history (Admin only - for data migration)
#[update]
fn admin_clear_rebalance_history() -> Result<String, String> {
    is_admin()?;
    ic_cdk::println!("🗑️ [ADMIN] Clearing all rebalance history");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    scheduler::clear_rebalance_history()
}

// --- User Position Management API ---

/// Create a new position for automatic tracking and rebalancing
#[update]
async fn create_position(
    permissions_id: String,
    protocol: String,
    asset: String,
    token_address: String,
    chain_id: u64,
    position_size: String,
    tracked: bool,
) -> Result<UserPosition, String> {
    let caller = ic_cdk::caller();

    ic_cdk::println!("🆕 Creating position for user: {}", caller);

    // Get user's EVM address
    let user_evm_address = get_evm_address()?;

    // Verify permissions ownership
    is_permissions_owner(&permissions_id, caller)?;

    // Create the position
    apy_parser::add_user_position(
        caller,
        user_evm_address,
        permissions_id,
        protocol,
        asset,
        token_address,
        chain_id,
        position_size,
        tracked,
    ).await
}

/// Get all positions for the current user
#[query]
fn get_my_positions() -> Vec<UserPosition> {
    let caller = ic_cdk::caller();
    ic_cdk::println!("📋 Getting positions for user: {}", caller);

    apy_parser::get_user_positions(caller)
}

/// Update position tracking status or size
#[update]
fn update_position(
    position_id: String,
    position_size: Option<String>,
    tracked: Option<bool>,
) -> Result<UserPosition, String> {
    let caller = ic_cdk::caller();

    ic_cdk::println!("🔄 Updating position {} for user: {}", position_id, caller);

    apy_parser::update_user_position(position_id, caller, position_size, tracked)
}

/// Delete a position
#[update]
fn delete_position(position_id: String) -> Result<bool, String> {
    let caller = ic_cdk::caller();

    ic_cdk::println!("🗑️ Deleting position {} for user: {}", position_id, caller);

    apy_parser::delete_user_position(position_id, caller)
}

/// Get a specific position by ID (only if owned by caller)
#[query]
fn get_position(position_id: String) -> Result<UserPosition, String> {
    let caller = ic_cdk::caller();

    let position = apy_parser::get_position_by_id(position_id)?;

    // Verify ownership
    if position.user_principal != caller {
        return Err("You do not own this position".to_string());
    }

    Ok(position)
}

// --- APY Parser Admin API ---

/// Initialize APY parser (Admin only)
#[update]
fn admin_init_apy_parser() -> Result<String, String> {
    is_admin()?;
    ic_cdk::println!("🔧 [ADMIN] Manually initializing APY parser");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    apy_parser::init_apy_parser();
    Ok("APY parser initialized successfully. Use admin_start_apy_parser to enable.".to_string())
}

/// Start APY collection (Admin only)
#[update]
fn admin_start_apy_parser() -> Result<String, String> {
    is_admin()?;
    ic_cdk::println!("▶️ [ADMIN] Starting APY parser");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    apy_parser::enable_apy_parser()
}

/// Stop APY collection (Admin only)
#[update]
fn admin_stop_apy_parser() -> Result<String, String> {
    is_admin()?;
    ic_cdk::println!("⏸️ [ADMIN] Stopping APY parser");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    apy_parser::disable_apy_parser()
}

/// Set APY collection interval (Admin only)
#[update]
fn admin_set_apy_parser_interval(seconds: u64) -> Result<String, String> {
    is_admin()?;
    ic_cdk::println!("⏱️ [ADMIN] Setting APY parser interval to {} seconds", seconds);
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    apy_parser::set_apy_parser_interval(seconds)
}

/// Manually trigger APY collection (Admin only)
#[update]
async fn admin_trigger_apy_collection() -> Result<String, String> {
    is_admin()?;
    ic_cdk::println!("🔨 [ADMIN] Manually triggering APY collection");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    apy_parser::trigger_manual_apy_collection().await
}

/// Get APY history for a protocol/asset/chain (Admin only)
#[query]
fn admin_get_apy_history(
    protocol: String,
    asset: String,
    chain_id: u64,
    limit: Option<u64>,
) -> Vec<ApyHistoryRecord> {
    // Admin check
    if let Err(e) = is_admin() {
        ic_cdk::println!("❌ Admin check failed: {}", e);
        return vec![];
    }

    ic_cdk::println!("📜 [ADMIN] Getting APY history for {} {} on chain {}", protocol, asset, chain_id);
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    apy_parser::get_apy_history(&protocol, &asset, chain_id, limit)
}

/// Get all APY history (Public query method)
#[query]
fn get_apy_history_all(limit: Option<u64>) -> Vec<ApyHistoryRecord> {
    ic_cdk::println!("📜 [PUBLIC] Getting all APY history");
    ic_cdk::println!("📝 Requested by principal: {}", ic_cdk::caller());

    apy_parser::get_all_apy_history(limit)
}

/// Get latest APY for a protocol/asset/chain combination (Public method)
/// Returns cached APY if available, otherwise fetches live from protocol
#[update]
async fn get_latest_apy(protocol: String, asset: String, chain_id: u64) -> Result<f64, String> {
    ic_cdk::println!("📊 [PUBLIC] Getting latest APY for {} {} on chain {}", protocol, asset, chain_id);
    ic_cdk::println!("📝 Requested by principal: {}", ic_cdk::caller());

    apy_parser::get_latest_apy(&protocol, &asset, chain_id).await
}

/// Get all positions in the system (Admin only)
#[query]
fn admin_get_all_positions() -> Vec<UserPosition> {
    // Admin check
    if let Err(e) = is_admin() {
        ic_cdk::println!("❌ Admin check failed: {}", e);
        return vec![];
    }

    ic_cdk::println!("📋 [ADMIN] Getting all positions");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    USER_POSITIONS_MAP.with(|map| {
        map.borrow()
            .iter()
            .map(|(_, position)| position.0.clone())
            .collect()
    })
}

/// Get all tracked positions (Admin only)
#[query]
fn admin_get_tracked_positions() -> Vec<UserPosition> {
    // Admin check
    if let Err(e) = is_admin() {
        ic_cdk::println!("❌ Admin check failed: {}", e);
        return vec![];
    }

    ic_cdk::println!("📋 [ADMIN] Getting tracked positions");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    apy_parser::get_tracked_positions()
}

/// Get APY parser status (Admin only)
#[query]
fn admin_get_apy_parser_status() -> Result<ApyParserStatus, String> {
    is_admin()?;
    ic_cdk::println!("📊 [ADMIN] Getting APY parser status");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    Ok(apy_parser::get_apy_parser_status())
}

/// Clear all APY history (Admin only - for data migration)
#[update]
fn admin_clear_apy_history() -> Result<String, String> {
    is_admin()?;
    ic_cdk::println!("🗑️ [ADMIN] Clearing all APY history");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    apy_parser::clear_apy_history()
}

/// Enable automatic position synchronization (Admin only)
#[update]
fn admin_enable_position_auto_sync() -> Result<String, String> {
    is_admin()?;
    ic_cdk::println!("▶️ [ADMIN] Enabling automatic position synchronization");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    apy_parser::enable_position_auto_sync()
}

/// Disable automatic position synchronization (Admin only)
#[update]
fn admin_disable_position_auto_sync() -> Result<String, String> {
    is_admin()?;
    ic_cdk::println!("⏸️ [ADMIN] Disabling automatic position synchronization");
    ic_cdk::println!("📝 Requested by admin principal: {}", ic_cdk::caller());

    apy_parser::disable_position_auto_sync()
}

/// Check if automatic position synchronization is enabled (Public query)
#[query]
fn is_position_auto_sync_enabled() -> bool {
    apy_parser::is_position_auto_sync_enabled()
}

// --- Helper Functions ---

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
    ic_cdk::println!("🚀 Initializing SmartWallet Manager...");

    // Initialize scheduler
    scheduler::init_scheduler();

    // Initialize APY parser
    apy_parser::init_apy_parser();

    // Note: Timers will not auto-start - admin must enable them
    ic_cdk::println!("✅ SmartWallet Manager Initialized.");
    ic_cdk::println!("ℹ️ Scheduler initialized but not started. Use admin_start_scheduler() to enable.");
    ic_cdk::println!("ℹ️ APY Parser initialized but not started. Use admin_start_apy_parser() to enable.");
}

#[post_upgrade]
fn post_upgrade() {
    ic_cdk::println!("🔄 Upgrading SmartWallet Manager...");

    // Stable memory is automatically preserved, no specific restore needed for StableBTreeMap

    // Restore scheduler timer if it was enabled before upgrade
    if scheduler::is_scheduler_enabled() {
        ic_cdk::println!("🔄 Scheduler was enabled, restarting timer...");
        scheduler::start_scheduler_timer();
    } else {
        ic_cdk::println!("ℹ️ Scheduler is disabled, timer not started.");
    }

    // Restore APY parser timer if it was enabled before upgrade
    if apy_parser::is_apy_parser_enabled() {
        ic_cdk::println!("🔄 APY Parser was enabled, restarting timer...");
        apy_parser::start_apy_parser_timer();
    } else {
        ic_cdk::println!("ℹ️ APY Parser is disabled, timer not started.");
    }

    ic_cdk::println!("✅ SmartWallet Manager Upgraded.");
}

// --- Candid export ---
ic_cdk::export_candid!();
