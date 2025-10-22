use candid::Principal;
use ic_cdk_timers::{set_timer_interval, clear_timer, TimerId};
use std::cell::RefCell;
use std::time::Duration;
use alloy::primitives::Address;

use crate::types::{UserPosition, ApyHistoryRecord, StorableUserPosition, StorableApyHistoryRecord};
use crate::{
    StorableString,
    APY_HISTORY_MAP, USER_POSITIONS_MAP, now
};

// =============================================================================
// Global State for APY Parser
// =============================================================================

thread_local! {
    /// Configuration for APY collection
    static APY_PARSER_CONFIG: RefCell<ApyParserConfig> = RefCell::new(ApyParserConfig::default());

    /// Active timer ID for APY collection
    static APY_PARSER_TIMER_ID: RefCell<Option<TimerId>> = RefCell::new(None);
}

// =============================================================================
// Configuration Types
// =============================================================================

#[derive(Clone, Debug)]
pub struct ApyParserConfig {
    /// Whether APY collection is enabled
    pub enabled: bool,

    /// Interval between APY collections in seconds (default: 900 = 15 minutes)
    pub interval_seconds: u64,

    /// Last time APY collection ran
    pub last_execution: Option<u64>,

    /// List of protocols to monitor
    pub monitored_protocols: Vec<String>,

    /// List of chains to monitor
    pub monitored_chains: Vec<u64>,

    /// Whether automatic position synchronization is enabled
    pub auto_sync_positions: bool,
}

impl Default for ApyParserConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_seconds: 900, // 15 minutes
            last_execution: None,
            monitored_protocols: vec!["AAVE".to_string(), "COMPOUND".to_string()],
            monitored_chains: vec![
                crate::services::rpc_service::ARBITRUM_CHAIN_ID,
            ],
            auto_sync_positions: true, // Enabled by default to track user positions
        }
    }
}

// =============================================================================
// Initialization Functions
// =============================================================================

/// Initialize APY parser with default configuration
pub fn init_apy_parser() {
    ic_cdk::println!("📊 Initializing APY Parser...");

    APY_PARSER_CONFIG.with(|config| {
        let mut cfg = config.borrow_mut();
        *cfg = ApyParserConfig::default();
    });

    ic_cdk::println!("✅ APY Parser initialized");
    ic_cdk::println!("  - Interval: {} seconds", ApyParserConfig::default().interval_seconds);
    ic_cdk::println!("  - Monitored Protocols: {:?}", ApyParserConfig::default().monitored_protocols);
    ic_cdk::println!("  - Monitored Chains: {:?}", ApyParserConfig::default().monitored_chains);
}

/// Start the APY collection timer
pub fn start_apy_parser_timer() {
    ic_cdk::println!("🚀 Starting APY parser timer...");

    let config = APY_PARSER_CONFIG.with(|c| c.borrow().clone());

    if !config.enabled {
        ic_cdk::println!("⚠️ APY Parser is disabled, timer not started");
        return;
    }

    // Clear existing timer if any
    APY_PARSER_TIMER_ID.with(|timer_id| {
        if let Some(id) = timer_id.borrow().as_ref() {
            ic_cdk::println!("🔄 Clearing existing APY parser timer...");
            clear_timer(*id);
        }
    });

    // Create new periodic timer
    let interval = Duration::from_secs(config.interval_seconds);
    let timer_id = set_timer_interval(interval, || {
        ic_cdk::spawn(async {
            execute_apy_collection().await;
        });
    });

    APY_PARSER_TIMER_ID.with(|id| {
        *id.borrow_mut() = Some(timer_id);
    });

    ic_cdk::println!("✅ APY parser timer started with interval: {} seconds", config.interval_seconds);
}

/// Stop the APY collection timer
pub fn stop_apy_parser_timer() {
    ic_cdk::println!("🛑 Stopping APY parser timer...");

    APY_PARSER_TIMER_ID.with(|timer_id| {
        if let Some(id) = timer_id.borrow().as_ref() {
            clear_timer(*id);
            ic_cdk::println!("✅ APY parser timer stopped successfully");
        } else {
            ic_cdk::println!("⚠️ No active APY parser timer to stop");
        }
        *timer_id.borrow_mut() = None;
    });
}

/// Check if APY parser is enabled
pub fn is_apy_parser_enabled() -> bool {
    APY_PARSER_CONFIG.with(|c| c.borrow().enabled)
}

// =============================================================================
// APY Collection Logic
// =============================================================================

/// Main APY collection execution
async fn execute_apy_collection() {
    execute_apy_collection_internal(false).await;
}

async fn execute_apy_collection_internal(force: bool) {
    ic_cdk::println!("⏰ APY collection started at {}", now());

    let config = APY_PARSER_CONFIG.with(|c| c.borrow().clone());

    if !force && !config.enabled {
        ic_cdk::println!("⚠️ APY Parser is disabled, skipping collection");
        return;
    }

    if force && !config.enabled {
        ic_cdk::println!("🔨 Forced manual collection - APY Parser is disabled but continuing anyway");
    }

    let mut total_collected = 0;
    let mut total_errors = 0;

    // Iterate through all monitored protocols and chains
    for protocol in &config.monitored_protocols {
        for &chain_id in &config.monitored_chains {
            ic_cdk::println!("📊 Collecting APY for {} on chain {}", protocol, chain_id);

            match collect_protocol_apy(protocol, chain_id).await {
                Ok(count) => {
                    total_collected += count;
                    ic_cdk::println!("✅ Collected {} APY records for {} on chain {}", count, protocol, chain_id);
                }
                Err(e) => {
                    total_errors += 1;
                    ic_cdk::println!("❌ Error collecting APY for {} on chain {}: {}", protocol, chain_id, e);
                }
            }
        }
    }

    // Update last execution time
    APY_PARSER_CONFIG.with(|c| {
        let mut cfg = c.borrow_mut();
        cfg.last_execution = Some(now());
    });

    ic_cdk::println!("📋 APY Collection Summary:");
    ic_cdk::println!("  - Total records collected: {}", total_collected);
    ic_cdk::println!("  - Errors: {}", total_errors);
    ic_cdk::println!("✅ APY collection completed");
}

/// Collect APY for a specific protocol on a specific chain
async fn collect_protocol_apy(protocol: &str, chain_id: u64) -> Result<u32, String> {
    let mut collected_count = 0;

    // Get supported tokens for this protocol and chain
    let tokens = get_supported_tokens(protocol, chain_id)?;

    for token_info in tokens {
        ic_cdk::println!("  Fetching APY for {} ({}) on {}", token_info.symbol, token_info.address, protocol);

        match fetch_protocol_apy(protocol, &token_info, chain_id).await {
            Ok(apy_value) => {
                // Create APY history record
                let record = ApyHistoryRecord {
                    record_id: generate_apy_record_id(protocol, chain_id, &token_info.address),
                    protocol: protocol.to_string(),
                    asset: token_info.symbol.clone(),
                    token_address: token_info.address.clone(),
                    chain_id,
                    apy: apy_value,
                    timestamp: now(),
                };

                // Store the record
                store_apy_record(record)?;
                collected_count += 1;

                ic_cdk::println!("  ✅ Stored APY: {}% for {} on {}", apy_value, token_info.symbol, protocol);
            }
            Err(e) => {
                ic_cdk::println!("  ⚠️ Could not fetch APY for {} on {}: {}", token_info.symbol, protocol, e);
            }
        }
    }

    Ok(collected_count)
}

/// Fetch APY from a specific protocol
async fn fetch_protocol_apy(
    protocol: &str,
    token_info: &TokenInfo,
    chain_id: u64,
) -> Result<f64, String> {
    match protocol {
        "AAVE" => {
            let token_address: Address = token_info.address.parse()
                .map_err(|_| "Invalid token address".to_string())?;
            let apy_str = crate::services::aave::get_apy(token_address, chain_id).await?;
            apy_str.parse::<f64>()
                .map_err(|_| format!("Failed to parse APY: {}", apy_str))
        }
        "COMPOUND" => {
            // Compound only supports USDC on Arbitrum currently
            if chain_id != crate::services::rpc_service::ARBITRUM_CHAIN_ID {
                return Err("Compound only available on Arbitrum".to_string());
            }
            let apy_str = crate::services::compound::get_apy(chain_id).await?;
            apy_str.parse::<f64>()
                .map_err(|_| format!("Failed to parse APY: {}", apy_str))
        }
        _ => Err(format!("Unknown protocol: {}", protocol))
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

#[derive(Clone, Debug)]
struct TokenInfo {
    symbol: String,
    address: String,
}

/// Get list of supported tokens for a protocol on a chain
fn get_supported_tokens(protocol: &str, chain_id: u64) -> Result<Vec<TokenInfo>, String> {
    use crate::services::rpc_service::{ARBITRUM_CHAIN_ID, SEPOLIA_CHAIN_ID, BASE_CHAIN_ID, OPTIMISM_CHAIN_ID};

    match (protocol, chain_id) {
        ("AAVE", ARBITRUM_CHAIN_ID) => Ok(vec![
            TokenInfo {
                symbol: "USDC".to_string(),
                address: "0xaf88d065e77c8cC2239327C5EDb3A432268e5831".to_string()
            },
        ]),
        ("AAVE", SEPOLIA_CHAIN_ID) => Ok(vec![
            TokenInfo {
                symbol: "USDC".to_string(),
                address: "0x94a9D9AC8a22534E3FaCa9954e183B2c3736704F".to_string()
            },
        ]),
        ("AAVE", BASE_CHAIN_ID) => Ok(vec![
            TokenInfo {
                symbol: "USDC".to_string(),
                address: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string()
            },
        ]),
        ("AAVE", OPTIMISM_CHAIN_ID) => Ok(vec![
            TokenInfo {
                symbol: "USDC".to_string(),
                address: "0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85".to_string()
            },
        ]),
        ("COMPOUND", ARBITRUM_CHAIN_ID) => Ok(vec![
            TokenInfo {
                symbol: "USDC".to_string(),
                address: "0xaf88d065e77c8cC2239327C5EDb3A432268e5831".to_string()
            },
        ]),
        _ => Err(format!("Protocol {} not supported on chain {}", protocol, chain_id))
    }
}

/// Generate unique record ID for APY history
fn generate_apy_record_id(protocol: &str, chain_id: u64, token_address: &str) -> String {
    let timestamp = now();
    format!("{}:{}:{}:{}", protocol, chain_id, token_address, timestamp)
}

/// Store APY record in history
fn store_apy_record(record: ApyHistoryRecord) -> Result<(), String> {
    ic_cdk::println!("📝 Storing APY record: {} - {}% ({})",
        record.protocol, record.apy, record.asset);

    APY_HISTORY_MAP.with(|map| {
        map.borrow_mut().insert(
            StorableString(record.record_id.clone()),
            StorableApyHistoryRecord(record)
        );
    });

    Ok(())
}

// =============================================================================
// Public Query Functions
// =============================================================================

/// Get latest APY for a protocol/asset/chain combination
pub async fn get_latest_apy(protocol: &str, asset: &str, chain_id: u64) -> Result<f64, String> {
    ic_cdk::println!("🔍 Getting latest APY for {} {} on chain {}", protocol, asset, chain_id);

    // First try to get from APY_HISTORY_MAP (get most recent)
    let cached_apy = APY_HISTORY_MAP.with(|map| {
        let borrowed = map.borrow();
        let mut matching_records: Vec<ApyHistoryRecord> = borrowed
            .iter()
            .filter_map(|(_, record)| {
                let r = record.0.clone();
                if r.protocol == protocol && r.asset == asset && r.chain_id == chain_id {
                    Some(r)
                } else {
                    None
                }
            })
            .collect();

        // Sort by timestamp descending and get the most recent
        matching_records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        matching_records.first().map(|r| r.apy)
    });

    if let Some(apy) = cached_apy {
        ic_cdk::println!("  ✅ Found cached APY: {}%", apy);
        return Ok(apy);
    }

    ic_cdk::println!("  No cached APY found, fetching live from protocol...");

    // Get token address for the asset
    let tokens = get_supported_tokens(protocol, chain_id)?;
    let token_info = tokens.iter()
        .find(|t| t.symbol == asset)
        .ok_or_else(|| format!("Token {} not found for protocol {}", asset, protocol))?;

    fetch_protocol_apy(protocol, token_info, chain_id).await
}

/// Get APY history for a specific protocol/asset/chain
pub fn get_apy_history(
    protocol: &str,
    asset: &str,
    chain_id: u64,
    limit: Option<u64>,
) -> Vec<ApyHistoryRecord> {
    ic_cdk::println!("📜 Getting APY history for {} {} on chain {} (limit: {:?})",
        protocol, asset, chain_id, limit);

    APY_HISTORY_MAP.with(|map| {
        let borrowed = map.borrow();
        let mut matching_records: Vec<ApyHistoryRecord> = borrowed
            .iter()
            .filter_map(|(_, record)| {
                let r = record.0.clone();
                if r.protocol == protocol && r.asset == asset && r.chain_id == chain_id {
                    Some(r)
                } else {
                    None
                }
            })
            .collect();

        // Sort by timestamp descending (most recent first)
        matching_records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit
        let limit = limit.unwrap_or(100) as usize;
        matching_records.into_iter().take(limit).collect()
    })
}

/// Get all APY history records (no filtering)
pub fn get_all_apy_history(limit: Option<u64>) -> Vec<ApyHistoryRecord> {
    ic_cdk::println!("📜 Getting all APY history records (limit: {:?})", limit);

    APY_HISTORY_MAP.with(|map| {
        let borrowed = map.borrow();
        let mut all_records: Vec<ApyHistoryRecord> = borrowed
            .iter()
            .map(|(_, record)| record.0.clone())
            .collect();

        // Sort by timestamp descending (most recent first)
        all_records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit (default 100 to prevent overwhelming responses)
        let limit = limit.unwrap_or(100) as usize;
        all_records.into_iter().take(limit).collect()
    })
}

// =============================================================================
// User Position Management
// =============================================================================

/// Generate unique position ID
async fn generate_position_id() -> String {
    let timestamp = now();
    let random_result = ic_cdk::api::management_canister::main::raw_rand().await;
    let random_bytes = match random_result {
        Ok((bytes,)) => bytes,
        Err((_, err)) => {
            ic_cdk::println!("Warning: Random generation failed: {}", err);
            vec![0u8; 8]
        }
    };

    let mut id_bytes = timestamp.to_be_bytes().to_vec();
    if random_bytes.len() >= 8 {
        id_bytes.extend_from_slice(&random_bytes[0..8]);
    }

    format!("pos_{}", hex::encode(id_bytes))
}

/// Add a new user position
pub async fn add_user_position(
    user_principal: Principal,
    user_evm_address: String,
    permissions_id: String,
    protocol: String,
    asset: String,
    token_address: String,
    chain_id: u64,
    position_size: String,
    tracked: bool,
) -> Result<UserPosition, String> {
    ic_cdk::println!("➕ Adding user position for user {}", user_principal);

    // Validation
    if position_size.is_empty() {
        return Err("Position size cannot be empty".to_string());
    }

    if protocol.is_empty() {
        return Err("Protocol cannot be empty".to_string());
    }

    // Generate unique position ID
    let position_id = generate_position_id().await;
    let timestamp = now();

    let position = UserPosition {
        position_id: position_id.clone(),
        user_principal,
        user_evm_address,
        permissions_id,
        protocol,
        asset,
        token_address,
        chain_id,
        position_size,
        tracked,
        added_at: timestamp,
        updated_at: timestamp,
    };

    // Store in USER_POSITIONS_MAP
    USER_POSITIONS_MAP.with(|map| {
        map.borrow_mut().insert(
            StorableString(position_id.clone()),
            StorableUserPosition(position.clone())
        );
    });

    ic_cdk::println!("✅ User position added successfully: {}", position_id);

    Ok(position)
}

/// Get all tracked positions (for scheduler)
pub fn get_tracked_positions() -> Vec<UserPosition> {
    ic_cdk::println!("📋 Getting all tracked positions...");

    USER_POSITIONS_MAP.with(|map| {
        let borrowed = map.borrow();
        borrowed
            .iter()
            .filter_map(|(_, position)| {
                let p = position.0.clone();
                if p.tracked {
                    Some(p)
                } else {
                    None
                }
            })
            .collect()
    })
}

/// Get positions for a specific user
pub fn get_user_positions(user: Principal) -> Vec<UserPosition> {
    ic_cdk::println!("📋 Getting positions for user: {}", user);

    USER_POSITIONS_MAP.with(|map| {
        let borrowed = map.borrow();
        borrowed
            .iter()
            .filter_map(|(_, position)| {
                let p = position.0.clone();
                if p.user_principal == user {
                    Some(p)
                } else {
                    None
                }
            })
            .collect()
    })
}

/// Update user position
pub fn update_user_position(
    position_id: String,
    user: Principal,
    position_size: Option<String>,
    tracked: Option<bool>,
) -> Result<UserPosition, String> {
    ic_cdk::println!("🔄 Updating position: {}", position_id);

    USER_POSITIONS_MAP.with(|map| {
        let mut borrowed = map.borrow_mut();

        // Get existing position
        let storable_position = borrowed
            .get(&StorableString(position_id.clone()))
            .ok_or_else(|| format!("Position {} not found", position_id))?;

        let mut position = storable_position.0.clone();

        // Verify ownership
        if position.user_principal != user {
            return Err("You do not own this position".to_string());
        }

        // Apply updates
        let mut updated = false;

        if let Some(size) = position_size {
            position.position_size = size;
            updated = true;
        }

        if let Some(track) = tracked {
            position.tracked = track;
            updated = true;
        }

        if updated {
            position.updated_at = now();

            // Save back to map
            borrowed.insert(
                StorableString(position_id.clone()),
                StorableUserPosition(position.clone())
            );

            ic_cdk::println!("✅ Position updated successfully");
            Ok(position)
        } else {
            ic_cdk::println!("⚠️ No changes to apply");
            Ok(position)
        }
    })
}

/// Delete user position
pub fn delete_user_position(position_id: String, user: Principal) -> Result<bool, String> {
    ic_cdk::println!("🗑️ Deleting position: {} for user {}", position_id, user);

    USER_POSITIONS_MAP.with(|map| {
        let mut borrowed = map.borrow_mut();

        // First verify ownership
        let storable_position = borrowed
            .get(&StorableString(position_id.clone()))
            .ok_or_else(|| format!("Position {} not found", position_id))?;

        if storable_position.0.user_principal != user {
            return Err("You do not own this position".to_string());
        }

        // Remove from map
        borrowed.remove(&StorableString(position_id.clone()));

        ic_cdk::println!("✅ Position deleted successfully");
        Ok(true)
    })
}

/// Get a single position by ID
pub fn get_position_by_id(position_id: String) -> Result<UserPosition, String> {
    USER_POSITIONS_MAP.with(|map| {
        map.borrow()
            .get(&StorableString(position_id.clone()))
            .map(|p| p.0.clone())
            .ok_or_else(|| format!("Position {} not found", position_id))
    })
}

// =============================================================================
// Configuration Management
// =============================================================================

/// Enable APY parser
pub fn enable_apy_parser() -> Result<String, String> {
    ic_cdk::println!("▶️ Enabling APY parser...");

    APY_PARSER_CONFIG.with(|c| {
        let mut cfg = c.borrow_mut();
        cfg.enabled = true;
    });

    start_apy_parser_timer();
    Ok("APY parser enabled and timer started".to_string())
}

/// Disable APY parser
pub fn disable_apy_parser() -> Result<String, String> {
    ic_cdk::println!("⏸️ Disabling APY parser...");

    APY_PARSER_CONFIG.with(|c| {
        let mut cfg = c.borrow_mut();
        cfg.enabled = false;
    });

    stop_apy_parser_timer();
    Ok("APY parser disabled and timer stopped".to_string())
}

/// Set APY collection interval
pub fn set_apy_parser_interval(seconds: u64) -> Result<String, String> {
    ic_cdk::println!("⏱️ Setting APY parser interval to {} seconds...", seconds);

    if seconds < 60 {
        return Err("Interval must be at least 60 seconds".to_string());
    }

    let was_enabled = is_apy_parser_enabled();

    APY_PARSER_CONFIG.with(|c| {
        let mut cfg = c.borrow_mut();
        cfg.interval_seconds = seconds;
    });

    // Restart timer if it was running
    if was_enabled {
        stop_apy_parser_timer();
        start_apy_parser_timer();
    }

    Ok(format!("APY parser interval updated to {} seconds", seconds))
}

/// Manually trigger APY collection
pub async fn trigger_manual_apy_collection() -> Result<String, String> {
    ic_cdk::println!("🔨 Manual APY collection triggered...");

    execute_apy_collection_internal(true).await;

    Ok("APY collection executed successfully".to_string())
}

/// Get APY parser configuration
pub fn get_apy_parser_config() -> ApyParserConfig {
    APY_PARSER_CONFIG.with(|c| c.borrow().clone())
}

/// Get APY parser status
pub fn get_apy_parser_status() -> crate::types::ApyParserStatus {
    let config = APY_PARSER_CONFIG.with(|c| c.borrow().clone());

    // Check if timer is active
    let timer_active = APY_PARSER_TIMER_ID.with(|timer_id| {
        timer_id.borrow().is_some()
    });

    // Get total records count from APY_HISTORY_MAP
    let total_records = crate::APY_HISTORY_MAP.with(|map| {
        map.borrow().len()
    });

    crate::types::ApyParserStatus {
        enabled: config.enabled,
        interval_seconds: config.interval_seconds,
        timer_active,
        last_execution: config.last_execution,
        total_records,
        monitored_protocols: config.monitored_protocols.clone(),
        monitored_chains: config.monitored_chains.clone(),
    }
}

/// Clear all APY history records (Admin only - for data migration)
pub fn clear_apy_history() -> Result<String, String> {
    ic_cdk::println!("🗑️ Clearing all APY history...");

    let count = crate::APY_HISTORY_MAP.with(|map| {
        let len = map.borrow().len();
        // Get all keys
        let keys: Vec<_> = map.borrow().iter().map(|(k, _)| k).collect();
        // Remove all entries
        for key in keys {
            map.borrow_mut().remove(&key);
        }
        len
    });

    ic_cdk::println!("✅ Cleared {} APY history records", count);
    Ok(format!("Cleared {} APY history records", count))
}

// =============================================================================
// Position Auto-Sync Configuration
// =============================================================================

/// Enable automatic position synchronization
pub fn enable_position_auto_sync() -> Result<String, String> {
    ic_cdk::println!("▶️ Enabling automatic position synchronization...");

    APY_PARSER_CONFIG.with(|c| {
        let mut cfg = c.borrow_mut();
        cfg.auto_sync_positions = true;
    });

    ic_cdk::println!("✅ Automatic position synchronization enabled");
    Ok("Automatic position synchronization enabled".to_string())
}

/// Disable automatic position synchronization
pub fn disable_position_auto_sync() -> Result<String, String> {
    ic_cdk::println!("⏸️ Disabling automatic position synchronization...");

    APY_PARSER_CONFIG.with(|c| {
        let mut cfg = c.borrow_mut();
        cfg.auto_sync_positions = false;
    });

    ic_cdk::println!("✅ Automatic position synchronization disabled");
    Ok("Automatic position synchronization disabled".to_string())
}

/// Check if automatic position synchronization is enabled
pub fn is_position_auto_sync_enabled() -> bool {
    APY_PARSER_CONFIG.with(|c| c.borrow().auto_sync_positions)
}
