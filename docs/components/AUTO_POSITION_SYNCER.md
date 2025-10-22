# Automatic Position Synchronization

## Overview

The Automatic Position Synchronization feature automatically tracks and updates user positions in DeFi protocols (AAVE, Compound) whenever users perform supply or withdraw operations. This eliminates the need for manual position management and ensures that position data is always in sync with on-chain state.

## Table of Contents

- [Features](#features)
- [Architecture](#architecture)
- [How It Works](#how-it-works)
- [Configuration](#configuration)
- [API Reference](#api-reference)
- [Usage Examples](#usage-examples)
- [Error Handling](#error-handling)
- [Implementation Details](#implementation-details)

---

## Features

### Core Capabilities

- **Auto-create positions**: Automatically creates a new `UserPosition` record on first supply to a protocol
- **Auto-update positions**: Updates position size when users supply or withdraw funds
- **Auto-delete positions**: Removes position records when balance reaches zero
- **Rebalancing support**: Properly handles position updates during protocol-to-protocol rebalancing
- **Global enable/disable**: Admin-controlled flag to enable or disable the feature globally
- **Non-blocking**: Position sync failures are logged as warnings and don't interrupt supply/withdraw operations

### Position Tracking Strategy

Positions are identified by the following composite key:
- `user_principal` - User's Internet Computer Principal
- `protocol` - Protocol name (e.g., "AAVE", "COMPOUND")
- `asset` - Asset symbol (e.g., "USDC", "LINK")
- `chain_id` - Blockchain network ID

**Note**: `permissions_id` is NOT part of the lookup key, allowing users to have one position per protocol/asset/chain combination regardless of which permission set was used.

---

## Architecture

### Module Structure

```
src/yieldex-ic-wallet-manager-backend/src/
├── services/
│   ├── position_sync.rs          # Core synchronization logic
│   ├── apy_parser.rs             # Config storage and management
│   ├── compound.rs               # Compound integration
│   ├── aave.rs                   # AAVE integration
│   └── rebalance.rs              # Rebalancing integration
└── lib.rs                        # Admin API endpoints
```

### Configuration Storage

The auto-sync flag is stored in `ApyParserConfig`:

```rust
pub struct ApyParserConfig {
    pub enabled: bool,
    pub interval_seconds: u64,
    pub last_execution: Option<u64>,
    pub monitored_protocols: Vec<String>,
    pub monitored_chains: Vec<u64>,
    pub auto_sync_positions: bool,  // 🆕 Auto-sync feature flag
}
```

**Default**: `auto_sync_positions = false` (disabled for safety)

---

## How It Works

### Supply Flow

```
┌─────────────────────────────────────────────────────────────┐
│ User calls: supply_usdc_to_compound_with_permissions(100)  │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
            ┌────────────────┐
            │ Execute Supply │
            └───────┬────────┘
                    │
                    ▼
            ┌───────────────┐
            │  ✅ Success?  │
            └───┬───────────┘
                │ Yes
                ▼
    ┌───────────────────────────┐
    │ sync_position_after_supply│
    └──────────┬────────────────┘
               │
               ▼
    ┌──────────────────────┐
    │ Find existing position│
    │ by: user + protocol + │
    │     asset + chain_id  │
    └──────┬───────────────┘
           │
           ▼
    ┌─────────────┐
    │ Exists?     │
    └──┬──────┬───┘
       │      │
    No │      │ Yes
       │      │
       ▼      ▼
   ┌──────┐ ┌─────────────────┐
   │CREATE│ │UPDATE:          │
   │new   │ │position_size =  │
   │posit.│ │old_size + amount│
   └──────┘ └─────────────────┘
```

### Withdraw Flow

```
┌──────────────────────────────────────────────────────────────┐
│ User calls: withdraw_usdc_from_compound_with_permissions(50)│
└────────────────────┬─────────────────────────────────────────┘
                     │
                     ▼
            ┌────────────────┐
            │Execute Withdraw│
            └───────┬────────┘
                    │
                    ▼
            ┌───────────────┐
            │  ✅ Success?  │
            └───┬───────────┘
                │ Yes
                ▼
    ┌────────────────────────────┐
    │ sync_position_after_withdraw│
    └──────────┬─────────────────┘
               │
               ▼
    ┌──────────────────────┐
    │ Find existing position│
    └──────┬───────────────┘
           │
           ▼
    ┌─────────────┐
    │ Found?      │
    └──┬──────────┘
       │ Yes
       ▼
    ┌─────────────────────┐
    │ new_size =          │
    │ old_size - amount   │
    └──────┬──────────────┘
           │
           ▼
    ┌─────────────┐
    │ new_size    │
    │ <= 0?       │
    └──┬──────┬───┘
       │      │
    Yes│      │No
       │      │
       ▼      ▼
   ┌──────┐ ┌─────────┐
   │DELETE│ │UPDATE   │
   │posit.│ │position │
   └──────┘ └─────────┘
```

### Rebalance Flow

When executing a recommendation to move funds from one protocol to another:

```
┌─────────────────────────────────────────────────────┐
│ execute_recommendation(AAVE → COMPOUND, amount=200)│
└───────────────────┬─────────────────────────────────┘
                    │
                    ▼
        ┌───────────────────────┐
        │ 1. Withdraw from AAVE │
        └──────────┬────────────┘
                   │
                   ▼
        ┌──────────────────┐
        │  ✅ Withdraw OK? │
        └───┬──────────────┘
            │
         No │ Yes
            │  │
            │  ▼
            │  ┌─────────────────────────────┐
            │  │ sync_position_after_withdraw│
            │  │ (AAVE position decreased)   │
            │  └──────────┬──────────────────┘
            │             │
            │             ▼
            │  ┌──────────────────────┐
            │  │ 2. Supply to COMPOUND│
            │  └──────┬───────────────┘
            │         │
            │         ▼
            │  ┌─────────────────┐
            │  │  ✅ Supply OK?  │
            │  └───┬─────────────┘
            │      │
            │   No │ Yes
            │      │  │
            ▼      │  ▼
    ┌────────────┐ │  ┌───────────────────────────┐
    │ Status:    │ │  │ sync_position_after_supply│
    │ "failed"   │ │  │ (COMPOUND position incr.) │
    │            │ │  └──────────┬────────────────┘
    │ No positions│ │             │
    │ synced     │ │             ▼
    └────────────┘ │  ┌───────────────┐
                   │  │ Status:       │
                   │  │ "success"     │
                   │  │               │
                   │  │ Both positions│
                   │  │ synced        │
                   │  └───────────────┘
                   ▼
          ┌──────────────────┐
          │ Status: "partial"│
          │                  │
          │ Only AAVE        │
          │ position synced  │
          └──────────────────┘
```

**Key Points:**
- ✅ **Both succeed**: Both positions are synced
- ❌ **Withdraw fails**: No positions are synced
- ⚠️ **Supply fails**: Only source position is synced (funds withdrawn but not re-deposited)

---

## Configuration

### Enable Auto-Sync (Admin Only)

```bash
dfx canister call yieldex-ic-wallet-manager-backend admin_enable_position_auto_sync
```

**Response:**
```
(variant { Ok = "Automatic position synchronization enabled" })
```

### Disable Auto-Sync (Admin Only)

```bash
dfx canister call yieldex-ic-wallet-manager-backend admin_disable_position_auto_sync
```

**Response:**
```
(variant { Ok = "Automatic position synchronization disabled" })
```

### Check Status (Public Query)

```bash
dfx canister call yieldex-ic-wallet-manager-backend is_position_auto_sync_enabled
```

**Response:**
```
(true)  // or (false)
```

---

## API Reference

### Admin Functions

#### `admin_enable_position_auto_sync()`

Enables automatic position synchronization globally.

**Access:** Admin only
**Type:** Update call
**Returns:** `Result<String, String>`

#### `admin_disable_position_auto_sync()`

Disables automatic position synchronization globally.

**Access:** Admin only
**Type:** Update call
**Returns:** `Result<String, String>`

### Public Query Functions

#### `is_position_auto_sync_enabled()`

Checks if automatic position synchronization is currently enabled.

**Access:** Public
**Type:** Query call
**Returns:** `bool`

### Internal Functions (position_sync.rs)

#### `sync_position_after_supply()`

```rust
pub async fn sync_position_after_supply(
    user_principal: Principal,
    permissions_id: String,
    protocol: String,
    asset: String,
    token_address: String,
    chain_id: u64,
    amount_supplied: String,
) -> Result<(), String>
```

Called after successful supply operation. Creates or updates user position.

#### `sync_position_after_withdraw()`

```rust
pub async fn sync_position_after_withdraw(
    user_principal: Principal,
    protocol: String,
    asset: String,
    chain_id: u64,
    amount_withdrawn: String,
) -> Result<(), String>
```

Called after successful withdraw operation. Updates or deletes user position.

#### `find_user_position()`

```rust
pub fn find_user_position(
    user: Principal,
    protocol: &str,
    asset: &str,
    chain_id: u64
) -> Option<UserPosition>
```

Searches for existing position by composite key.

---

## Usage Examples

### Example 1: First Supply to Compound

**Scenario:** User supplies 100 USDC to Compound for the first time.

```bash
# Auto-sync enabled
dfx canister call yieldex-ic-wallet-manager-backend \
  supply_usdc_to_compound_with_permissions \
  '("100", "perm_abc123", principal "xxxxx-xxxxx")'
```

**What Happens:**
1. ✅ Supply transaction executes successfully
2. 🔍 System searches for existing position (user + COMPOUND + USDC + chain)
3. ❌ No position found
4. ➕ Creates new `UserPosition`:
   - `protocol`: "COMPOUND"
   - `asset`: "USDC"
   - `position_size`: "100"
   - `tracked`: false (default)

**Logs:**
```
🚀 Starting Compound USDC supply: 100 USDC...
✅ Step 8 Complete: USDC supplied to Compound
✅ Step 8: Syncing user position...
🔍 Searching for position: user=xxxxx, protocol=COMPOUND, asset=USDC, chain_id=42161
ℹ️ No existing position found
➕ Creating new position for user xxxxx
✅ New position created: pos_abc123 with size 100
✅ Step 8 Complete: User position synced
```

### Example 2: Adding to Existing Position

**Scenario:** User already has 100 USDC in AAVE, supplies another 50 USDC.

```bash
dfx canister call yieldex-ic-wallet-manager-backend \
  supply_to_aave_with_permissions \
  '(
    record { address = "0xaf88...5831" },
    "USDC",
    "50",
    "perm_xyz789",
    principal "xxxxx-xxxxx",
    42161
  )'
```

**What Happens:**
1. ✅ Supply transaction executes successfully
2. 🔍 System searches for existing position
3. ✅ Position found with `position_size = "100"`
4. 🔄 Updates position:
   - New `position_size`: "150" (100 + 50)
   - `updated_at`: current timestamp

**Logs:**
```
🚀 Starting AAVE USDC supply: 50 USDC...
✅ Step 12: Syncing user position...
🔍 Searching for position: user=xxxxx, protocol=AAVE, asset=USDC, chain_id=42161
✅ Found existing position: pos_def456
📝 Updating existing position: pos_def456
💰 Position size change: 100 → 150 (added 50)
✅ Position updated: 100 → 150 (added 50)
✅ Step 12 Complete: User position synced
```

### Example 3: Withdrawing to Zero

**Scenario:** User withdraws all 150 USDC from their AAVE position.

```bash
dfx canister call yieldex-ic-wallet-manager-backend \
  withdraw_from_aave_with_permissions \
  '(
    record { address = "0xaf88...5831" },
    "USDC",
    "150",
    "perm_xyz789",
    principal "xxxxx-xxxxx",
    42161
  )'
```

**What Happens:**
1. ✅ Withdraw transaction executes successfully
2. 🔍 System searches for existing position
3. ✅ Position found with `position_size = "150"`
4. 🧮 Calculates new size: 150 - 150 = 0
5. 🗑️ Deletes position (balance reached zero)

**Logs:**
```
🚀 Starting AAVE USDC withdraw: 150 USDC...
✅ Step 12: Syncing user position...
🔍 Searching for position: user=xxxxx, protocol=AAVE, asset=USDC, chain_id=42161
✅ Found existing position: pos_def456
📝 Updating position after withdrawal: pos_def456
💰 Position size change: 150 → 0 (withdrawn 150)
🗑️ Position balance reached zero, deleting position: pos_def456
✅ Position deleted successfully
✅ Step 12 Complete: User position synced
```

### Example 4: Rebalancing Between Protocols

**Scenario:** User rebalances 200 USDC from AAVE to Compound.

```bash
dfx canister call yieldex-ic-wallet-manager-backend \
  execute_recommendation \
  '(
    record {
      from_protocol = "AAVE";
      to_protocol = "COMPOUND";
      asset = "USDC";
      to_asset = "USDC";
      from_chain = 42161;
      to_chain = null;
      position_size = "200";
      recommendation_type = variant { StandardTransfer };
    },
    "perm_abc123",
    principal "xxxxx-xxxxx"
  )'
```

**What Happens:**
1. ✅ Withdraw 200 USDC from AAVE succeeds
2. 🔄 Sync AAVE position: 300 → 100 (decreased by 200)
3. ✅ Supply 200 USDC to Compound succeeds
4. 🔄 Sync Compound position: 0 → 200 (new position created)
5. 📊 Result status: "success"

**Logs:**
```
🚀 Starting recommendation execution
📤 Step 1: Withdrawing from AAVE...
✅ Withdraw successful
🔄 Syncing source position after withdrawal...
✅ Source position synced after withdrawal
📥 Step 2: Supplying to COMPOUND...
✅ Supply successful
🔄 Syncing target position after supply...
✅ Target position synced after supply
🎉 Rebalance flow completed with status: success
```

---

## Error Handling

### Non-Blocking Design

Position synchronization is designed to be **non-blocking**. If sync fails, the main operation (supply/withdraw) is still considered successful, and only a warning is logged.

**Example:**
```rust
match crate::services::position_sync::sync_position_after_supply(...).await {
    Ok(_) => ic_cdk::println!("✅ Step 8 Complete: User position synced"),
    Err(e) => ic_cdk::println!("⚠️ Step 8 Warning: Position sync failed: {}", e),
}
// Supply operation continues successfully
```

### Common Error Scenarios

#### 1. User Has No EVM Address

**Error:** `"User does not have an EVM address"`

**Cause:** User hasn't generated an EVM address yet.

**Impact:** Position not created, but supply/withdraw still succeeds.

**Resolution:** User should generate EVM address before first supply.

#### 2. Auto-Sync Disabled

**Behavior:** Sync functions exit early with log message:

```
ℹ️ Position auto-sync is disabled, skipping synchronization
```

**Impact:** No position records created/updated.

#### 3. Invalid Amount Format

**Error:** `"Failed to parse amount: <details>"`

**Cause:** `amount_supplied` or `amount_withdrawn` is not a valid number.

**Impact:** Sync fails, but main operation succeeded.

#### 4. Position Not Found on Withdraw

**Behavior:** Logs warning but doesn't fail:

```
⚠️ No existing position found to update after withdrawal
   User may have withdrawn from a position not tracked in the system
```

**Impact:** No error thrown, gracefully handles missing position.

---

## Implementation Details

### Position Lifecycle

```
┌─────────────────────────────────────────────────────┐
│                  Position States                    │
├─────────────────────────────────────────────────────┤
│                                                     │
│  [Not Exists] ──supply──> [Created (size > 0)]     │
│                                                     │
│  [Created] ──supply──> [Updated (size increased)]  │
│                                                     │
│  [Created] ──withdraw──> [Updated (size decreased)]│
│                                                     │
│  [Created] ──withdraw(all)──> [Deleted]            │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### UserPosition Schema

```rust
pub struct UserPosition {
    pub position_id: String,          // Unique ID (e.g., "pos_abc123")
    pub user_principal: Principal,    // User's ICP principal
    pub user_evm_address: String,     // User's EVM address
    pub permissions_id: String,       // Associated permissions ID
    pub protocol: String,             // "AAVE" | "COMPOUND"
    pub asset: String,                // "USDC" | "LINK" | etc.
    pub token_address: String,        // EVM token contract address
    pub chain_id: u64,                // Network ID (e.g., 42161 for Arbitrum)
    pub position_size: String,        // Human-readable amount (e.g., "150.5")
    pub tracked: bool,                // Whether scheduler monitors this
    pub added_at: u64,                // Creation timestamp (nanoseconds)
    pub updated_at: u64,              // Last update timestamp (nanoseconds)
}
```

### Composite Key for Position Lookup

Positions are uniquely identified by:

```rust
(user_principal, protocol, asset, chain_id)
```

**Example:**
```
(
  principal "xxxxx-xxxxx",
  "COMPOUND",
  "USDC",
  42161
)
```

**Why not include `permissions_id`?**
- Simplifies position management
- Allows users to interact with same position using different permission sets
- One position per protocol/asset/chain combination per user

### Balance Calculation

Position size is calculated **mathematically** (not fetched on-chain):

**Supply:**
```rust
new_position_size = old_position_size + amount_supplied
```

**Withdraw:**
```rust
new_position_size = old_position_size - amount_withdrawn
if new_position_size <= 0.0001 {
    delete_position()
} else {
    update_position(new_position_size)
}
```

**Why mathematical calculation?**
- ✅ Faster (no additional RPC calls)
- ✅ Deterministic
- ✅ Matches the actual on-chain changes
- ⚠️ Assumes supply/withdraw amounts are accurate

### Integration Points

#### Compound Integration

**File:** `services/compound.rs`

**Functions Modified:**
- `supply_usdc_to_compound_with_permissions` (line 225-239)
- `withdraw_usdc_from_compound_with_permissions` (line 388-399)

#### AAVE Integration

**File:** `services/aave.rs`

**Functions Modified:**
- `supply_to_aave_with_permissions` (line 236-250)
- `withdraw_from_aave_with_permissions` (line 427-438)

#### Rebalance Integration

**File:** `services/rebalance.rs`

**Function Modified:**
- `execute_same_chain_same_asset` (lines 188-199, 223-240)

### Storage

Positions are stored in stable memory using `StableBTreeMap`:

```rust
pub static USER_POSITIONS_MAP: RefCell<
    StableBTreeMap<StorableString, StorableUserPosition, Memory>
> = ...
```

**Key:** `position_id` (e.g., "pos_abc123")
**Value:** `StorableUserPosition` wrapper around `UserPosition`

---

## Security Considerations

### Admin-Only Control

Only admin principals can enable/disable auto-sync:

```rust
fn admin_enable_position_auto_sync() -> Result<String, String> {
    is_admin()?;  // Checks caller against ADMIN_PRINCIPALS list
    apy_parser::enable_position_auto_sync()
}
```

### Position Ownership

Positions are tied to `user_principal`, ensuring users can only modify their own positions.

### Non-Destructive Failures

If position sync fails, the main operation (supply/withdraw) still succeeds, preventing fund loss.

---

## Troubleshooting

### Position Not Created After Supply

**Possible Causes:**
1. Auto-sync is disabled → Check with `is_position_auto_sync_enabled()`
2. User has no EVM address → User should generate one first
3. Sync failed → Check canister logs for error details

### Position Not Deleted After Full Withdraw

**Check:**
- Was auto-sync enabled during the withdraw?
- Look for warning logs in canister output
- Manually delete using `delete_user_position()` if needed

### Position Size Incorrect

**Possible Causes:**
- Auto-sync was disabled during some operations
- Manual position updates interfered with auto-sync
- Partial failures during rebalancing

**Solution:** Manually update using `update_user_position()` or re-sync by withdrawing and re-supplying.

---

## Future Enhancements

### Potential Improvements

1. **On-Chain Balance Verification**
   - Periodically fetch actual on-chain balances
   - Auto-correct discrepancies

2. **Per-User Auto-Sync Control**
   - Allow users to opt-in/opt-out individually
   - Add user preference in `UserPosition` or separate config

3. **Position History**
   - Track all position changes over time
   - Enable position balance auditing

4. **Cross-Chain Position Aggregation**
   - Sum positions across multiple chains
   - Unified view of user's total holdings per asset

5. **Automatic `tracked` Flag Management**
   - Auto-enable tracking for positions above threshold
   - Integration with scheduler for APY monitoring

---

## References

### Related Documentation

- [APY Parser Documentation](./docs/apy_parser.md) (if exists)
- [Scheduler Documentation](./docs/scheduler.md) (if exists)
- [User Positions API](./docs/user_positions_api.md) (if exists)

### Code Locations

- **Position Sync Module**: [src/services/position_sync.rs](src/yieldex-ic-wallet-manager-backend/src/services/position_sync.rs)
- **APY Parser Config**: [src/services/apy_parser.rs](src/yieldex-ic-wallet-manager-backend/src/services/apy_parser.rs)
- **Admin Endpoints**: [src/lib.rs](src/yieldex-ic-wallet-manager-backend/src/lib.rs) (lines 1500-1524)

### Git History

To see the full implementation:
```bash
git log --oneline --all --graph -- '**/position_sync.rs'
```

---

## License

This feature is part of the Yieldex IC Wallet Manager project.

---

**Last Updated:** 2025-10-22
**Version:** 1.0.0
**Status:** ✅ Production Ready
