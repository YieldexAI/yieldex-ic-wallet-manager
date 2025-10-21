# APY Parser Module

## 📋 Description

APY Parser is a module for automatic collection, storage, and provision of historical APY (Annual Percentage Yield) data from various DeFi protocols, as well as for managing user positions for automated rebalancing.

## 🏗️ Architecture

### Main Components

1. **APY Collection Engine** - periodically collects APY from protocols
2. **Position Manager** - manages user positions
3. **Persistent Storage** - StableBTreeMap is used to store data
4. **Scheduler Integration** - integration with the automatic rebalancing module

### Data Structures

#### UserPosition
```rust
pub struct UserPosition {
    pub position_id: String,           // Unique position ID
    pub user_principal: Principal,     // User's principal
    pub user_evm_address: String,      // User's EVM address
    pub permissions_id: String,        // Permissions ID
    pub protocol: String,              // "AAVE" | "COMPOUND"
    pub asset: String,                 // "USDC", "LINK", etc.
    pub token_address: String,         // Token EVM address
    pub chain_id: u64,                 // Chain ID (42161 = Arbitrum)
    pub position_size: String,         // Position size (human-readable)
    pub tracked: bool,                 // Track for auto rebalancing?
    pub added_at: u64,                 // Creation timestamp
    pub updated_at: u64,               // Update timestamp
}
```

#### ApyHistoryRecord
```rust
pub struct ApyHistoryRecord {
    pub record_id: String,             // Unique record ID
    pub protocol: String,              // "AAVE" | "COMPOUND"
    pub asset: String,                 // "USDC", "LINK", etc.
    pub token_address: String,         // Token EVM address
    pub chain_id: u64,                 // Chain ID
    pub apy: f64,                      // APY percent
    pub timestamp: u64,                // Record timestamp
}
```

#### ApyParserConfig
```rust
pub struct ApyParserConfig {
    pub enabled: bool,                 // Collector enabled
    pub interval_seconds: u64,         // Collection interval (default 900 = 15 minutes)
    pub last_execution: Option<u64>,   // Last collection time
    pub monitored_protocols: Vec<String>, // ["AAVE", "COMPOUND"]
    pub monitored_chains: Vec<u64>,    // [42161, 11155111]
}
```

## 🗄️ Storage (StableBTreeMap)

### APY_HISTORY_MAP
- **Key:** `record_id` (String)
- **Value:** `ApyHistoryRecord`
- **Key format:** `{protocol}:{chain_id}:{token_address}:{timestamp}`
- **Example:** `AAVE:42161:0xaf88d065e77c8cC2239327C5EDb3A432268e5831:1704067200000`

### USER_POSITIONS_MAP
- **Key:** `position_id` (String)
- **Value:** `UserPosition`
- **Key format:** `pos_{timestamp_hex}{random_hex}`
- **Example:** `pos_0000018d1234abcd5678ef90`

### REBALANCE_HISTORY_MAP
- **Key:** `execution_id` (String)
- **Value:** `RebalanceExecution`
- **Purpose:** History of all performed rebalances

## 🔧 API Endpoints

### User APIs

#### `create_position`
Create a new position to track.

```bash
dfx canister call yieldex-ic-wallet-manager-backend create_position '(
  "permissions_id",      # User's permissions ID
  "AAVE",                # Protocol
  "USDC",                # Token symbol
  "0xaf88d065e77c8cC2239327C5EDb3A432268e5831", # Token address
  42161,                 # Chain ID (Arbitrum)
  "1000",                # Position size in USDC
  true                   # Track for automatic rebalancing
)'
```
**Returns:** `UserPosition`

#### `get_my_positions`
Get all positions for current user.

```bash
dfx canister call yieldex-ic-wallet-manager-backend get_my_positions
```
**Returns:** `Vec<UserPosition>`

#### `update_position`
Update the parameters for a position.

```bash
dfx canister call yieldex-ic-wallet-manager-backend update_position '(
  "pos_0000018d1234abcd", # Position ID
  opt "2000",             # New position size (optional)
  opt false               # Disable tracking (optional)
)'
```
**Returns:** `Result<UserPosition, String>`

#### `delete_position`
Delete a position.

```bash
dfx canister call yieldex-ic-wallet-manager-backend delete_position '("pos_0000018d1234abcd")'
```
**Returns:** `Result<bool, String>`

#### `get_position`
Get a specific position by ID (only if owned by caller).

```bash
dfx canister call yieldex-ic-wallet-manager-backend get_position '("pos_0000018d1234abcd")'
```
**Returns:** `Result<UserPosition, String>`

---

### Admin APIs

#### `admin_init_apy_parser`
Initialize the APY parser (for existing canisters deployed before the module was added).

```bash
dfx canister call yieldex-ic-wallet-manager-backend admin_init_apy_parser
```
**Returns:** `Result<String, String>`

#### `admin_start_apy_parser`
Start periodic APY collection.

```bash
dfx canister call yieldex-ic-wallet-manager-backend admin_start_apy_parser
```
**Returns:** `Result<String, String>`

#### `admin_stop_apy_parser`
Stop APY collection.

```bash
dfx canister call yieldex-ic-wallet-manager-backend admin_stop_apy_parser
```
**Returns:** `Result<String, String>`

#### `admin_set_apy_parser_interval`
Set APY collection interval (in seconds).

```bash
# Set interval to 15 minutes (900 seconds)
dfx canister call yieldex-ic-wallet-manager-backend admin_set_apy_parser_interval '(900)'

# Set interval to 1 hour (3600 seconds)
dfx canister call yieldex-ic-wallet-manager-backend admin_set_apy_parser_interval '(3600)'
```
**Returns:** `Result<String, String>`

**Restriction:** Minimum 60 seconds

#### `admin_trigger_apy_collection`
Manually trigger APY collection (regardless of timer).

```bash
dfx canister call yieldex-ic-wallet-manager-backend admin_trigger_apy_collection
```
**Returns:** `Result<String, String>`

#### `admin_get_apy_history`
Get APY history for a specific protocol/token/network.

```bash
dfx canister call yieldex-ic-wallet-manager-backend admin_get_apy_history '(
  "AAVE",    # Protocol
  "USDC",    # Token
  42161,     # Chain ID
  opt 10     # Record limit (optional, default 100)
)'
```
**Returns:** `Vec<ApyHistoryRecord>` (sorted by descending timestamp)

#### `admin_get_all_positions`
Get all user positions in the system.

```bash
dfx canister call yieldex-ic-wallet-manager-backend admin_get_all_positions
```
**Returns:** `Vec<UserPosition>`

#### `admin_get_tracked_positions`
Get only positions marked as tracked (tracked = true).

```bash
dfx canister call yieldex-ic-wallet-manager-backend admin_get_tracked_positions
```
**Returns:** `Vec<UserPosition>`

## 🔄 Scheduler Integration

APY Parser is integrated with the automatic rebalancing module:

1. **Scheduler uses APY Parser to get positions:**
   ```rust
   let tracked_positions = apy_parser::get_tracked_positions();
   ```

2. **Scheduler uses cached APY data:**
   ```rust
   let apy = apy_parser::get_latest_apy(protocol, asset, chain_id).await?;
   ```
   - First checks the cache in `APY_HISTORY_MAP`
   - If unavailable or outdated, performs a live protocol query

3. **Rebalance history is saved in `REBALANCE_HISTORY_MAP`**

## 🚀 Quick Start

### 1. Deploy & Initialize

```bash
# Deploy canister
dfx deploy yieldex-ic-wallet-manager-backend

# APY Parser is auto-initialized in init()
# But must be started manually
dfx canister call yieldex-ic-wallet-manager-backend admin_start_apy_parser
```

### 2. Configuration (optional)

```bash
# Set APY collection interval (e.g., 30 minutes)
dfx canister call yieldex-ic-wallet-manager-backend admin_set_apy_parser_interval '(1800)'
```

### 3. Creating a user position

```bash
# 1. Generate EVM address (if not already created)
dfx canister call yieldex-ic-wallet-manager-backend generate_evm_address

# 2. Create permissions
dfx canister call yieldex-ic-wallet-manager-backend create_permissions '(record {
  chain_id = 42161;
  whitelisted_protocols = vec { record { name = "AAVE"; address = "0x794a61358D6845594F94dc1DB02A252b5b4814aD" } };
  whitelisted_tokens = vec { record { name = "USDC"; address = "0xaf88d065e77c8cC2239327C5EDb3A432268e5831" } };
  transfer_limits = vec {};
  protocol_permissions = null;
})'

# 3. Create position
dfx canister call yieldex-ic-wallet-manager-backend create_position '(
  "<permissions_id>",
  "AAVE",
  "USDC",
  "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
  42161,
  "1000",
  true
)'
```

### 4. Start Scheduler for Automatic Rebalancing

```bash
# Start scheduler
dfx canister call yieldex-ic-wallet-manager-backend admin_start_scheduler

# Set APY threshold for rebalancing (e.g., 0.5%)
dfx canister call yieldex-ic-wallet-manager-backend admin_set_apy_threshold '(0.5)'
```

## 📊 Monitoring

### Check APY Parser Status

```bash
# See latest collected data
dfx canister call yieldex-ic-wallet-manager-backend admin_get_apy_history '("AAVE", "USDC", 42161, opt 5)'

# List all tracked positions
dfx canister call yieldex-ic-wallet-manager-backend admin_get_tracked_positions

# Get scheduler status (includes positions info)
dfx canister call yieldex-ic-wallet-manager-backend admin_get_scheduler_status
```

### Manual APY Check

```bash
# Get current APY directly from protocol (admin only)
dfx canister call yieldex-ic-wallet-manager-backend get_current_apy '("USDC", 42161)'
```

## 🔍 Supported Protocols and Tokens

### AAVE V3

| Network   | Chain ID | Tokens | Pool Address |
|-----------|----------|--------|--------------|
| Arbitrum  | 42161    | USDC   | 0x794a61358D6845594F94dc1DB02A252b5b4814aD |
| Sepolia   | 11155111 | USDC   | 0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951 |
| Base      | 8453     | USDC   | 0x794a61358D6845594F94dc1DB02A252b5b4814aD |
| Optimism  | 10       | USDC   | 0x794a61358D6845594F94dc1DB02A252b5b4814aD |

### Compound III

| Network   | Chain ID | Tokens | Comet Address |
|-----------|----------|--------|---------------|
| Arbitrum  | 42161    | USDC   | 0x9c4ec768c28520b50860ea7a15bd7213a9ff58bf |

## 🔐 Security

### Access Control

- **User endpoints:** accessible only to position owner
- **Admin endpoints:** `is_admin()` check via `ADMIN_PRINCIPALS` list

### Data Validation

- Ownership validation when updating/deleting positions
- Permissions check before creating a position
- Minimum APY collection interval: 60 seconds

## 🐛 Debugging

### Logging

APY Parser outputs detailed logs via `ic_cdk::println!`:

```
📊 APY collection started at 1704067200000
📊 Collecting APY for AAVE on chain 42161
  Fetching APY for USDC (0xaf88...) on AAVE
  ✅ Stored APY: 5.23% for USDC on AAVE
✅ Collected 1 APY records for AAVE on chain 42161
📋 APY Collection Summary:
  - Total records collected: 1
  - Errors: 0
✅ APY collection completed
```

### Common Errors

**"Scheduler not initialized"**
- Solution: Call `admin_init_scheduler()` or `admin_init_apy_parser()`

**"Position ID cannot be empty"**
- Solution: Ensure all required fields are filled when creating a position

**"Interval must be at least 60 seconds"**
- Solution: Set interval >= 60 seconds

**"Token X not found for protocol Y"**
- Solution: Ensure that the token is supported by the protocol on the given network

## 📈 Performance

### Recommended Settings

- **APY collection interval:** 15-30 minutes (900-1800 seconds)
- **Scheduler interval:** 1-2 hours (3600-7200 seconds)
- **APY threshold for rebalancing:** 0.5-1.0%

### Limitations

- StableBTreeMap - unbounded for `UserPosition` and `ApyHistoryRecord`
- Periodic cleanup of old APY records is recommended (admin endpoint can be added)

## 🔄 Canister Upgrade

APY Parser correctly handles canister upgrades:

```rust
#[post_upgrade]
fn post_upgrade() {
    // Stable memory is automatically preserved

    // Timers are restored if they were enabled
    if apy_parser::is_apy_parser_enabled() {
        apy_parser::start_apy_parser_timer();
    }
}
```

## 📝 TODO / Future Improvements

- [ ] Add endpoint to clean old APY records
- [ ] Support for more tokens (ETH, DAI, USDT)
- [ ] Webhook alerts for significant APY changes
- [ ] Graphical monitoring dashboard for APY
- [ ] Export APY history in CSV
- [ ] Automatic position detection from on-chain data

## 🤝 Integration With Other Modules

### Scheduler Module
- Uses `get_tracked_positions()` to get positions
- Uses `get_latest_apy()` for rebalancing decisions

### Rebalance Module
- Rebalance history is stored in `REBALANCE_HISTORY_MAP`
- Available via `admin_get_rebalance_history()`

### Permissions Module
- Permissions are checked when creating positions
- Ownership is validated

## 📚 Usage Examples

### Scenario 1: Create and Track Position

```bash
# 1. Create a position
POSITION=$(dfx canister call yieldex-ic-wallet-manager-backend create_position '(
  "perm_123",
  "AAVE",
  "USDC",
  "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
  42161,
  "5000",
  true
)' | grep position_id | awk '{print $3}')

# 2. Check position
dfx canister call yieldex-ic-wallet-manager-backend get_position "(\"$POSITION\")"

# 3. Update position size
dfx canister call yieldex-ic-wallet-manager-backend update_position "(
  \"$POSITION\",
  opt \"7500\",
  null
)"
```

### Scenario 2: APY Monitoring

```bash
# Run daily to collect data
dfx canister call yieldex-ic-wallet-manager-backend admin_trigger_apy_collection

# Get history for last 24 hours (with 15-min interval = 96 records)
dfx canister call yieldex-ic-wallet-manager-backend admin_get_apy_history '("AAVE", "USDC", 42161, opt 96)'
```

### Scenario 3: Rebalancing Analysis

```bash
# Get last 10 rebalancings
dfx canister call yieldex-ic-wallet-manager-backend admin_get_rebalance_history '(opt 10)'

# Get rebalancings for a specific user
dfx canister call yieldex-ic-wallet-manager-backend admin_get_user_rebalance_history '(
  principal "hfugy-ahqdz-5sbki-vky4l-xceci-3se5z-2cb7k-jxjuq-qidax-gd53f-nqe",
  opt 20
)'
```

## 🔗 Related Files

- **Module:** [src/yieldex-ic-wallet-manager-backend/src/services/apy_parser.rs](src/yieldex-ic-wallet-manager-backend/src/services/apy_parser.rs)
- **Types:** [src/yieldex-ic-wallet-manager-backend/src/types/scheduler.rs](src/yieldex-ic-wallet-manager-backend/src/types/scheduler.rs)
- **API:** [src/yieldex-ic-wallet-manager-backend/src/lib.rs](src/yieldex-ic-wallet-manager-backend/src/lib.rs) (lines 1258-1449)
- **Integration:** [src/yieldex-ic-wallet-manager-backend/src/services/scheduler.rs](src/yieldex-ic-wallet-manager-backend/src/services/scheduler.rs)

---

**Version:** 1.0.0  
**Date:** 2025-01-21  
**Status:** ✅ Production Ready
