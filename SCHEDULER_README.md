# Scheduler Module - Automatic Rebalancing System

## Overview

The scheduler module provides an automatic system for rebalancing users’ positions between DeFi protocols (AAVE and Compound) based on APY rate comparison.

### Key Features

- ✅ **Single global timer** - one periodic timer for all users
- ✅ **Automatic recommendation generation** - based on live APYs from protocols
- ✅ **Configurable APY threshold** - rebalancing only occurs when threshold is exceeded
- ✅ **Full admin control** - start/stop, parameter configuration
- ✅ **Execution history** - logs all rebalancing events
- ✅ **Post-upgrade recovery** - timer auto-restarts after upgrade

## Architecture

### How It Works

1. **Initialization:** On canister deploy, a scheduler config is created (disabled by default)
2. **Activation:** Admin enables scheduler via `admin_start_scheduler()`
3. **Periodic Check:** Every N seconds (configurable) the following occurs:
   - Fetch list of tracked positions (from USER_POSITIONS DB)
   - For each position:
     - Fetch APY of current protocol
     - Fetch APY of alternative protocol
     - Compare difference with threshold
     - If difference > threshold → create and execute recommendation
   - Save results to history

### Components

#### 1. Data Types ([types/scheduler.rs](src/yieldex-ic-wallet-manager-backend/src/types/scheduler.rs))

- `SchedulerConfig` - global scheduler config
- `SchedulerStatus` - current status and statistics
- `UserPosition` - user’s position in a protocol (future DB)
- `ApyHistoryRecord` - historical APY data (future DB)
- `RebalanceExecution` - record of a rebalancing execution

#### 2. Core Logic ([services/scheduler.rs](src/yieldex-ic-wallet-manager-backend/src/services/scheduler.rs))

**Initialization:**
- `init_scheduler()` - create default config
- `start_scheduler_timer()` - start IC CDK timer
- `stop_scheduler_timer()` - stop timer

**Main Logic:**
- `execute_scheduler_tick()` - core function triggered by timer
- `process_position()` - process a single position
- `generate_recommendation()` - create Recommendation object

**Management:**
- `enable_scheduler()` / `disable_scheduler()`
- `set_scheduler_interval(seconds)` - change interval
- `set_apy_threshold(percent)` - set APY threshold
- `trigger_manual_execution()` - manual check & rebalance

**Monitoring:**
- `get_scheduler_status()` - current status
- `get_rebalance_history()` - execution history

#### 3. Admin API ([lib.rs](src/yieldex-ic-wallet-manager-backend/src/lib.rs))

All methods are admin-only (checked via `is_admin()`):

**Query methods:**
- `admin_get_scheduler_config()` - get config
- `admin_get_scheduler_status()` - get status and stats
- `admin_get_rebalance_history(limit)` - get all rebalances
- `admin_get_user_rebalance_history(user, limit)` - per-user history

**Update methods:**
- `admin_update_scheduler_config(config)` - update config
- `admin_start_scheduler()` - start scheduler
- `admin_stop_scheduler()` - stop scheduler
- `admin_set_scheduler_interval(seconds)` - set check interval
- `admin_set_apy_threshold(percent)` - set APY threshold
- `admin_trigger_rebalance()` - manually trigger check & rebalance

## Configuration

### SchedulerConfig Parameters

```rust
pub struct SchedulerConfig {
    pub enabled: bool,                    // Is scheduler enabled
    pub interval_seconds: u64,            // Check interval (default 3600 = 1 hour)
    pub apy_threshold_percent: f64,       // APY threshold for rebalancing (default 0.5%)
    pub min_position_size: String,        // Minimum position size (default "100")
    pub last_execution: Option<u64>,      // Timestamp of last execution
    pub created_at: u64,
    pub updated_at: u64,
}
```

### Default Values

- **enabled**: `false` (scheduler does not start automatically)
- **interval_seconds**: `3600` (1 hour)
- **apy_threshold_percent**: `0.5` (0.5% APY difference)
- **min_position_size**: `"100"` ($100 USDC minimum)

## Usage

### 1. Initial Setup

After deploying the canister, the scheduler is initialized but not active.

```bash
# Get current config
dfx canister call yieldex-ic-wallet-manager-backend admin_get_scheduler_config

# Start scheduler
dfx canister call yieldex-ic-wallet-manager-backend admin_start_scheduler

# Check status
dfx canister call yieldex-ic-wallet-manager-backend admin_get_scheduler_status
```

### 2. Configure Parameters

```bash
# Set check interval to 30 minutes (1800 seconds)
dfx canister call yieldex-ic-wallet-manager-backend admin_set_scheduler_interval '(1800 : nat64)'

# Set APY threshold to 1% (more conservative)
dfx canister call yieldex-ic-wallet-manager-backend admin_set_apy_threshold '(1.0 : float64)'
```

### 3. Manual Execution

```bash
# Manually trigger check and rebalancing
dfx canister call yieldex-ic-wallet-manager-backend admin_trigger_rebalance
```

### 4. Monitoring

```bash
# Get scheduler status
dfx canister call yieldex-ic-wallet-manager-backend admin_get_scheduler_status

# Get last 10 rebalances
dfx canister call yieldex-ic-wallet-manager-backend admin_get_rebalance_history '(opt 10 : opt nat64)'

# Get per-user history
dfx canister call yieldex-ic-wallet-manager-backend admin_get_user_rebalance_history '(principal "hfugy-ahqdz-...", opt 5 : opt nat64)'
```

### 5. Stopping

```bash
# Stop scheduler
dfx canister call yieldex-ic-wallet-manager-backend admin_stop_scheduler
```

## Current Limitations & TODO

### Implemented ✅

- [x] Scheduler data types
- [x] Scheduler core logic using IC CDK timers
- [x] Admin management API
- [x] Integration with lifecycle hooks (init/post_upgrade)
- [x] Auto recommendation generation
- [x] Real APY fetching from AAVE & Compound
- [x] In-memory execution history

### TODO / Upcoming Enhancements 📋

#### 1. Positions DB (USER_POSITIONS)

**Current state:** Mock fn `get_tracked_positions()` returns an empty array

**To implement:**
```rust
// Add StableBTreeMap in lib.rs
const USER_POSITIONS_MEMORY_ID: MemoryId = MemoryId::new(3);

thread_local! {
    static USER_POSITIONS_MAP: RefCell<StableBTreeMap<StorableString, StorableUserPosition, Memory>> = ...;
}

// API methods for managing positions
#[update]
fn add_tracked_position(position: UserPosition) -> Result<String, String>

#[update]
fn remove_tracked_position(position_id: String) -> Result<bool, String>

#[query]
fn get_user_positions(user: Principal) -> Vec<UserPosition>
```

#### 2. APY History DB (APY_HISTORY)

**Current state:** APY is fetched in real time from protocols

**Optional enhancement:**
- Store historical APY for analytics
- Trend/predictive calculations
- Optimization - cache APY to reduce repeated queries

#### 3. Stable memory for rebalancing history

**Current state:** History is stored in `RefCell<Vec<RebalanceExecution>>` (does not survive restart)

**Required:**
```rust
// In lib.rs
const REBALANCE_HISTORY_MEMORY_ID: MemoryId = MemoryId::new(5);

thread_local! {
    static REBALANCE_HISTORY_MAP: RefCell<StableBTreeMap<StorableString, StorableRebalanceExecution, Memory>> = ...;
}
```

#### 4. Advanced profit logic

**Current state:** Simple APY > threshold comparison

**Improvements:**
- Take gas costs into account for profitability
- Minimum profit including commissions
- Dynamic calculation of optimal rebalance timing

#### 5. Notifications and Alerts

- Log to event log
- Optional user notifications
- Dashboard for monitoring

#### 6. Multi-chain support

**Current state:** Works with a single network at once

**Improvements:**
- Support cross-chain rebalancing
- Cross-chain APY comparison

## Security

### Admin-only Access

All scheduler management functions are admin-only:

```rust
const ADMIN_PRINCIPALS: &[&str] = &[
    "hfugy-ahqdz-5sbki-vky4l-xceci-3se5z-2cb7k-jxjuq-qidax-gd53f-nqe",
];
```

Every admin function checks:
```rust
fn is_admin() -> Result<(), String> {
    let caller = ic_cdk::caller();
    if ADMIN_PRINCIPALS.contains(&caller.to_text().as_str()) {
        Ok(())
    } else {
        Err("Unauthorized: Only admins can call this function")
    }
}
```

### Rebalance Checks

1. **Ownership verification:** Permission check to ensure user owns the position
2. **Protocol permissions:** Check rights to interact with protocols
3. **Amount limits:** Obey permissions-based limits
4. **Minimum position size:** Only rebalance if position > minimum

## Log Examples

### Successful Scheduler Tick

```
⏰ Scheduler tick started at 1234567890000
📊 Checking 5 tracked positions...
🔍 Processing position: pos_001 for user hfugy-ahqdz-...
  Comparing APY: AAVE vs COMPOUND
  Fetching real-time APY for AAVE on USDC
  Current APY (AAVE): 3.2%
  Alternative APY (COMPOUND): 4.1%
  APY Difference: 0.9%
  Threshold: 0.5%
  ✅ APY difference exceeds threshold, generating recommendation...
  📝 Recommendation generated: AAVE -> COMPOUND
  🚀 Executing rebalance...
  ✅ Rebalance executed: exec_abc123 (status: success)
📋 Scheduler tick summary:
  - Positions checked: 5
  - Rebalances triggered: 2
  - Successful: 2
  - Failed: 0
  - Errors: 0
✅ Scheduler tick completed
```

## Technical Details

### IC CDK Timers

Scheduler uses `ic-cdk-timers` for periodic execution:

```rust
let interval = Duration::from_secs(config.interval_seconds);
let timer_id = set_timer_interval(interval, || {
    ic_cdk::spawn(async {
        execute_scheduler_tick().await;
    });
});
```

### Post-upgrade Recovery

On `post_upgrade` the scheduler state is checked:

```rust
#[post_upgrade]
fn post_upgrade() {
    if scheduler::is_scheduler_enabled() {
        scheduler::start_scheduler_timer();
    }
}
```

Scheduler config is stored in `RefCell<Option<SchedulerConfig>>` and will persist across upgrades via stable memory (to be implemented).

## Testing

### Unit tests (TODO)

```bash
cargo test --package yieldex-ic-wallet-manager-backend --lib services::scheduler
```

### Integration tests (TODO)

1. Scheduler initialization test
2. Start/stop timer test
3. Recommendation generation test
4. Rebalance execution test
5. Post-upgrade recovery test

## Related Files

- [types/scheduler.rs](src/yieldex-ic-wallet-manager-backend/src/types/scheduler.rs) - data types
- [types/storable.rs](src/yieldex-ic-wallet-manager-backend/src/types/storable.rs) - Storable wrappers
- [services/scheduler.rs](src/yieldex-ic-wallet-manager-backend/src/services/scheduler.rs) - main logic
- [services/rebalance.rs](src/yieldex-ic-wallet-manager-backend/src/services/rebalance.rs) - rebalance execution
- [lib.rs](src/yieldex-ic-wallet-manager-backend/src/lib.rs) - Admin API and lifecycle hooks

## FAQ

**Q: Why doesn’t the scheduler start automatically after deployment?**  
A: For security, the scheduler is disabled by default. The admin must explicitly enable it using `admin_start_scheduler()`.

**Q: How frequently can checks run?**  
A: Minimum interval is 60 seconds. 1-24 hours is recommended depending on APY volatility.

**Q: What if a rebalance fails?**  
A: Scheduler logs the error in `RebalanceExecution` with status="failed" or "partial" and keeps working.

**Q: Can a user opt out of automatic rebalancing?**  
A: Once the positions DB is implemented, a `tracked` flag per position can be managed.

**Q: Does the scheduler work on all networks?**  
A: Yes, but Compound is only on Arbitrum. AAVE runs on Arbitrum and Sepolia.
