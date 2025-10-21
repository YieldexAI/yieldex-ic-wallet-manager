use candid::Principal;
use ic_cdk_timers::{set_timer_interval, clear_timer, TimerId};
use std::cell::RefCell;
use std::time::Duration;

use crate::types::{
    SchedulerConfig, SchedulerStatus, UserPosition,
    RebalanceExecution, SchedulerExecutionSummary, Recommendation,
    RecommendationType, StorableRebalanceExecution,
};
use crate::{REBALANCE_HISTORY_MAP, StorableString};

// =============================================================================
// Global State (will be integrated into lib.rs)
// =============================================================================

thread_local! {
    /// Global scheduler configuration
    static SCHEDULER_CONFIG: RefCell<Option<SchedulerConfig>> = RefCell::new(None);

    /// Active timer ID for the scheduler
    static SCHEDULER_TIMER_ID: RefCell<Option<TimerId>> = RefCell::new(None);
}

// =============================================================================
// Initialization Functions
// =============================================================================

/// Initialize scheduler with default configuration
pub fn init_scheduler() {
    ic_cdk::println!("📅 Initializing scheduler...");

    let now = crate::now();
    let config = SchedulerConfig {
        enabled: false,
        interval_seconds: 3600, // 1 hour default
        apy_threshold_percent: 0.5, // 0.5% APY difference
        min_position_size: "100".to_string(), // $100 USDC minimum
        last_execution: None,
        created_at: now,
        updated_at: now,
    };

    SCHEDULER_CONFIG.with(|c| {
        *c.borrow_mut() = Some(config.clone());
    });

    ic_cdk::println!("✅ Scheduler initialized with default config:");
    ic_cdk::println!("  - Enabled: {}", config.enabled);
    ic_cdk::println!("  - Interval: {} seconds", config.interval_seconds);
    ic_cdk::println!("  - APY Threshold: {}%", config.apy_threshold_percent);
    ic_cdk::println!("  - Min Position Size: ${}", config.min_position_size);
}

/// Start the scheduler timer
pub fn start_scheduler_timer() {
    ic_cdk::println!("🚀 Starting scheduler timer...");

    let config = SCHEDULER_CONFIG.with(|c| c.borrow().clone());

    let Some(config) = config else {
        ic_cdk::println!("❌ Cannot start timer: Scheduler not initialized");
        return;
    };

    if !config.enabled {
        ic_cdk::println!("⚠️ Scheduler is disabled, timer not started");
        return;
    }

    // Clear existing timer if any
    SCHEDULER_TIMER_ID.with(|timer_id| {
        if let Some(id) = timer_id.borrow().as_ref() {
            ic_cdk::println!("🔄 Clearing existing timer...");
            clear_timer(*id);
        }
    });

    // Create new periodic timer
    let interval = Duration::from_secs(config.interval_seconds);
    let timer_id = set_timer_interval(interval, || {
        ic_cdk::spawn(async {
            execute_scheduler_tick().await;
        });
    });

    SCHEDULER_TIMER_ID.with(|id| {
        *id.borrow_mut() = Some(timer_id);
    });

    ic_cdk::println!("✅ Scheduler timer started with interval: {} seconds", config.interval_seconds);
}

/// Stop the scheduler timer
pub fn stop_scheduler_timer() {
    ic_cdk::println!("🛑 Stopping scheduler timer...");

    SCHEDULER_TIMER_ID.with(|timer_id| {
        if let Some(id) = timer_id.borrow().as_ref() {
            clear_timer(*id);
            ic_cdk::println!("✅ Timer stopped successfully");
        } else {
            ic_cdk::println!("⚠️ No active timer to stop");
        }
        *timer_id.borrow_mut() = None;
    });
}

/// Check if scheduler is enabled
pub fn is_scheduler_enabled() -> bool {
    SCHEDULER_CONFIG.with(|c| {
        c.borrow().as_ref().map(|cfg| cfg.enabled).unwrap_or(false)
    })
}

// =============================================================================
// Main Scheduler Logic
// =============================================================================

/// Main scheduler tick - called by the timer
async fn execute_scheduler_tick() {
    ic_cdk::println!("⏰ Scheduler tick started at {}", crate::now());

    let config = SCHEDULER_CONFIG.with(|c| c.borrow().clone());

    let Some(mut config) = config else {
        ic_cdk::println!("❌ Scheduler not initialized");
        return;
    };

    if !config.enabled {
        ic_cdk::println!("⚠️ Scheduler is disabled, skipping tick");
        return;
    }

    let mut summary = SchedulerExecutionSummary {
        timestamp: crate::now(),
        positions_checked: 0,
        rebalances_triggered: 0,
        rebalances_successful: 0,
        rebalances_failed: 0,
        execution_ids: Vec::new(),
        errors: Vec::new(),
    };

    // Get all tracked positions (currently mock, will be from DB)
    let positions = get_tracked_positions();
    summary.positions_checked = positions.len() as u64;

    ic_cdk::println!("📊 Checking {} tracked positions...", positions.len());

    // Process each position
    for position in positions {
        ic_cdk::println!("🔍 Processing position: {} for user {}",
            position.position_id, position.user_principal);

        match process_position(&position, &config).await {
            Ok(Some(execution)) => {
                summary.rebalances_triggered += 1;

                if execution.result.status == "success" {
                    summary.rebalances_successful += 1;
                } else {
                    summary.rebalances_failed += 1;
                }

                summary.execution_ids.push(execution.execution_id.clone());

                // Store execution in history
                REBALANCE_HISTORY_MAP.with(|map| {
                    map.borrow_mut().insert(
                        StorableString(execution.execution_id.clone()),
                        StorableRebalanceExecution(execution)
                    );
                });
            },
            Ok(None) => {
                // No rebalance needed
                ic_cdk::println!("✅ No rebalance needed for position {}", position.position_id);
            },
            Err(e) => {
                summary.rebalances_failed += 1;
                summary.errors.push(format!("Position {}: {}", position.position_id, e));
                ic_cdk::println!("❌ Error processing position {}: {}", position.position_id, e);
            }
        }
    }

    // Update last execution time
    config.last_execution = Some(crate::now());
    config.updated_at = crate::now();
    SCHEDULER_CONFIG.with(|c| {
        *c.borrow_mut() = Some(config);
    });

    ic_cdk::println!("📋 Scheduler tick summary:");
    ic_cdk::println!("  - Positions checked: {}", summary.positions_checked);
    ic_cdk::println!("  - Rebalances triggered: {}", summary.rebalances_triggered);
    ic_cdk::println!("  - Successful: {}", summary.rebalances_successful);
    ic_cdk::println!("  - Failed: {}", summary.rebalances_failed);
    ic_cdk::println!("  - Errors: {}", summary.errors.len());

    ic_cdk::println!("✅ Scheduler tick completed");
}

/// Process a single position and determine if rebalance is needed
async fn process_position(
    position: &UserPosition,
    config: &SchedulerConfig,
) -> Result<Option<RebalanceExecution>, String> {
    // Check if position size meets minimum threshold
    let position_amount: f64 = position.position_size.parse()
        .map_err(|_| "Invalid position size")?;
    let min_amount: f64 = config.min_position_size.parse()
        .map_err(|_| "Invalid min position size")?;

    if position_amount < min_amount {
        ic_cdk::println!("  Position size ${} below minimum ${}, skipping",
            position_amount, min_amount);
        return Ok(None);
    }

    // Get APY rates for current protocol and alternative
    let current_protocol = &position.protocol;
    let alternative_protocol = if current_protocol == "AAVE" { "COMPOUND" } else { "AAVE" };

    ic_cdk::println!("  Comparing APY: {} vs {}", current_protocol, alternative_protocol);

    let current_apy = get_latest_apy(current_protocol, &position.asset, position.chain_id).await?;
    let alternative_apy = get_latest_apy(alternative_protocol, &position.asset, position.chain_id).await?;

    ic_cdk::println!("  Current APY ({}): {}%", current_protocol, current_apy);
    ic_cdk::println!("  Alternative APY ({}): {}%", alternative_protocol, alternative_apy);

    // Calculate APY difference
    let apy_difference = alternative_apy - current_apy;

    ic_cdk::println!("  APY Difference: {}%", apy_difference);
    ic_cdk::println!("  Threshold: {}%", config.apy_threshold_percent);

    // Check if rebalance is profitable
    if apy_difference < config.apy_threshold_percent {
        ic_cdk::println!("  APY difference below threshold, no rebalance needed");
        return Ok(None);
    }

    ic_cdk::println!("  ✅ APY difference exceeds threshold, generating recommendation...");

    // Generate recommendation
    let recommendation = generate_recommendation(
        position,
        current_protocol,
        alternative_protocol,
        current_apy,
        alternative_apy,
    )?;

    ic_cdk::println!("  📝 Recommendation generated: {} -> {}",
        recommendation.from_protocol, recommendation.to_protocol);

    // Execute recommendation
    ic_cdk::println!("  🚀 Executing rebalance...");
    let result = crate::services::rebalance::execute_recommendation(
        recommendation.clone(),
        position.permissions_id.clone(),
        position.user_principal,
    ).await?;

    // Create execution record
    let execution_id = generate_execution_id().await;
    let execution = RebalanceExecution {
        execution_id: execution_id.clone(),
        user_principal: position.user_principal,
        position_id: position.position_id.clone(),
        recommendation,
        result,
        apy_difference,
        timestamp: crate::now(),
    };

    ic_cdk::println!("  ✅ Rebalance executed: {} (status: {})",
        execution_id, execution.result.status);

    Ok(Some(execution))
}

/// Generate a recommendation based on position and APY comparison
fn generate_recommendation(
    position: &UserPosition,
    current_protocol: &str,
    target_protocol: &str,
    current_apy: f64,
    target_apy: f64,
) -> Result<Recommendation, String> {
    // Estimate profit (simplified - just APY difference)
    let position_amount: f64 = position.position_size.parse()
        .map_err(|_| "Invalid position size")?;
    let apy_diff = target_apy - current_apy;
    let estimated_annual_profit = position_amount * (apy_diff / 100.0);

    // Simplified gas cost estimate (will be more accurate with real data)
    let estimated_gas_cost = 5.0; // $5 estimate

    Ok(Recommendation {
        asset: position.asset.clone(),
        to_asset: position.asset.clone(),
        from_chain: get_chain_name(position.chain_id),
        to_chain: None,
        from_protocol: current_protocol.to_string(),
        to_protocol: target_protocol.to_string(),
        current_apy,
        target_apy,
        estimated_profit: estimated_annual_profit,
        gas_cost: estimated_gas_cost,
        position_size: position.position_size.clone(),
        pool_id: None,
        recommendation_type: RecommendationType::StandardTransfer,
        swap_details: None,
    })
}

// =============================================================================
// Mock / Temporary Functions (will be replaced with DB queries)
// =============================================================================

/// Get all tracked positions
fn get_tracked_positions() -> Vec<UserPosition> {
    ic_cdk::println!("📋 Getting tracked positions from database...");

    // Query USER_POSITIONS_MAP and filter by tracked=true
    crate::services::apy_parser::get_tracked_positions()
}

/// Get latest APY for a protocol (uses APY parser with fallback to live queries)
async fn get_latest_apy(protocol: &str, asset: &str, chain_id: u64) -> Result<f64, String> {
    ic_cdk::println!("  Getting APY for {} {} on chain {}", protocol, asset, chain_id);

    // Use APY parser which will check cache first, then fall back to live query
    crate::services::apy_parser::get_latest_apy(protocol, asset, chain_id).await
}

/// Get chain name from chain ID
fn get_chain_name(chain_id: u64) -> String {
    crate::services::rpc_service::get_chain_name(chain_id)
        .unwrap_or("Unknown")
        .to_string()
}

/// Generate unique execution ID
async fn generate_execution_id() -> String {
    let timestamp = crate::now();
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

    format!("exec_{}", hex::encode(id_bytes))
}

// =============================================================================
// Configuration Management Functions
// =============================================================================

/// Get current scheduler configuration
pub fn get_scheduler_config() -> Result<SchedulerConfig, String> {
    SCHEDULER_CONFIG.with(|c| {
        c.borrow()
            .clone()
            .ok_or_else(|| "Scheduler not initialized".to_string())
    })
}

/// Update scheduler configuration
pub fn update_scheduler_config(new_config: SchedulerConfig) -> Result<SchedulerConfig, String> {
    ic_cdk::println!("🔧 Updating scheduler configuration...");

    let mut config = new_config;
    config.updated_at = crate::now();

    SCHEDULER_CONFIG.with(|c| {
        *c.borrow_mut() = Some(config.clone());
    });

    ic_cdk::println!("✅ Configuration updated");
    Ok(config)
}

/// Enable the scheduler and start timer
pub fn enable_scheduler() -> Result<String, String> {
    ic_cdk::println!("▶️ Enabling scheduler...");

    SCHEDULER_CONFIG.with(|c| {
        let mut borrowed = c.borrow_mut();
        if let Some(ref mut config) = *borrowed {
            config.enabled = true;
            config.updated_at = crate::now();
        } else {
            return Err("Scheduler not initialized".to_string());
        }
        Ok(())
    })?;

    start_scheduler_timer();
    Ok("Scheduler enabled and timer started".to_string())
}

/// Disable the scheduler and stop timer
pub fn disable_scheduler() -> Result<String, String> {
    ic_cdk::println!("⏸️ Disabling scheduler...");

    SCHEDULER_CONFIG.with(|c| {
        let mut borrowed = c.borrow_mut();
        if let Some(ref mut config) = *borrowed {
            config.enabled = false;
            config.updated_at = crate::now();
        } else {
            return Err("Scheduler not initialized".to_string());
        }
        Ok(())
    })?;

    stop_scheduler_timer();
    Ok("Scheduler disabled and timer stopped".to_string())
}

/// Set scheduler interval (restarts timer if running)
pub fn set_scheduler_interval(seconds: u64) -> Result<String, String> {
    ic_cdk::println!("⏱️ Setting scheduler interval to {} seconds...", seconds);

    if seconds < 60 {
        return Err("Interval must be at least 60 seconds".to_string());
    }

    let was_enabled = is_scheduler_enabled();

    SCHEDULER_CONFIG.with(|c| {
        let mut borrowed = c.borrow_mut();
        if let Some(ref mut config) = *borrowed {
            config.interval_seconds = seconds;
            config.updated_at = crate::now();
        } else {
            return Err("Scheduler not initialized".to_string());
        }
        Ok(())
    })?;

    // Restart timer if it was running
    if was_enabled {
        stop_scheduler_timer();
        start_scheduler_timer();
    }

    Ok(format!("Interval updated to {} seconds", seconds))
}

/// Set APY threshold
pub fn set_apy_threshold(percent: f64) -> Result<String, String> {
    ic_cdk::println!("📊 Setting APY threshold to {}%...", percent);

    if percent < 0.0 {
        return Err("APY threshold must be positive".to_string());
    }

    SCHEDULER_CONFIG.with(|c| {
        let mut borrowed = c.borrow_mut();
        if let Some(ref mut config) = *borrowed {
            config.apy_threshold_percent = percent;
            config.updated_at = crate::now();
        } else {
            return Err("Scheduler not initialized".to_string());
        }
        Ok(())
    })?;

    Ok(format!("APY threshold updated to {}%", percent))
}

/// Manually trigger scheduler execution
pub async fn trigger_manual_execution() -> Result<Vec<RebalanceExecution>, String> {
    ic_cdk::println!("🔨 Manual scheduler execution triggered...");

    execute_scheduler_tick().await;

    // Return recent executions from this tick
    REBALANCE_HISTORY_MAP.with(|map| {
        let borrowed = map.borrow();
        let mut all: Vec<RebalanceExecution> = borrowed
            .iter()
            .map(|(_, exec)| exec.0.clone())
            .collect();

        // Sort by timestamp descending
        all.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let recent: Vec<_> = all.into_iter().take(10).collect();
        Ok(recent)
    })
}

// =============================================================================
// Monitoring Functions
// =============================================================================

/// Get scheduler status
pub fn get_scheduler_status() -> Result<SchedulerStatus, String> {
    let config = get_scheduler_config()?;

    let timer_active = SCHEDULER_TIMER_ID.with(|id| id.borrow().is_some());

    let total_rebalances = REBALANCE_HISTORY_MAP.with(|map| map.borrow().len() as u64);

    let total_positions_tracked = crate::services::apy_parser::get_tracked_positions().len() as u64;

    let last_result = REBALANCE_HISTORY_MAP.with(|map| {
        let borrowed = map.borrow();
        let mut all: Vec<RebalanceExecution> = borrowed
            .iter()
            .map(|(_, exec)| exec.0.clone())
            .collect();

        // Sort by timestamp descending
        all.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        all.first().map(|exec| {
            format!("ID: {}, Status: {}, User: {}",
                exec.execution_id, exec.result.status, exec.user_principal)
        })
    });

    Ok(SchedulerStatus {
        config,
        timer_active,
        total_positions_tracked,
        total_rebalances_executed: total_rebalances,
        last_execution_result: last_result,
    })
}

/// Get rebalance history (most recent first)
pub fn get_rebalance_history(limit: Option<u64>) -> Vec<RebalanceExecution> {
    REBALANCE_HISTORY_MAP.with(|map| {
        let borrowed = map.borrow();
        let mut all: Vec<RebalanceExecution> = borrowed
            .iter()
            .map(|(_, exec)| exec.0.clone())
            .collect();

        // Sort by timestamp descending
        all.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let limit = limit.unwrap_or(100) as usize;
        all.into_iter().take(limit).collect()
    })
}

/// Get rebalance history for specific user
pub fn get_user_rebalance_history(user: Principal, limit: Option<u64>) -> Vec<RebalanceExecution> {
    REBALANCE_HISTORY_MAP.with(|map| {
        let borrowed = map.borrow();
        let mut user_history: Vec<RebalanceExecution> = borrowed
            .iter()
            .filter_map(|(_, exec)| {
                let e = exec.0.clone();
                if e.user_principal == user {
                    Some(e)
                } else {
                    None
                }
            })
            .collect();

        // Sort by timestamp descending
        user_history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let limit = limit.unwrap_or(50) as usize;
        user_history.into_iter().take(limit).collect()
    })
}
