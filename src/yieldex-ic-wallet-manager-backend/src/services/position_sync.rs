use candid::Principal;
use crate::{USER_POSITIONS_MAP, PRINCIPAL_TO_ADDRESS_MAP, StorableString, StorablePrincipal, now};
use crate::types::{UserPosition, StorableUserPosition};
use crate::services::apy_parser;

// =============================================================================
// Position Synchronization Module
// =============================================================================
//
// This module provides automatic synchronization of UserPosition records
// when users interact with DeFi protocols (AAVE, Compound) through supply
// and withdraw operations.
//
// Key features:
// - Auto-create position on first supply
// - Auto-update position size on supply/withdraw
// - Auto-delete position when balance reaches zero
// - Global enable/disable flag in ApyParserConfig
// - Search positions by: user_principal + protocol + asset + chain_id
//

// =============================================================================
// Configuration Functions
// =============================================================================

/// Check if automatic position synchronization is enabled globally
pub fn is_auto_sync_enabled() -> bool {
    apy_parser::is_position_auto_sync_enabled()
}

// =============================================================================
// Position Lookup Functions
// =============================================================================

/// Find existing user position by user, protocol, asset, and chain
/// This searches across all positions without requiring permissions_id
pub fn find_user_position(
    user: Principal,
    protocol: &str,
    asset: &str,
    chain_id: u64
) -> Option<UserPosition> {
    ic_cdk::println!("🔍 Searching for position: user={}, protocol={}, asset={}, chain_id={}",
        user, protocol, asset, chain_id);

    USER_POSITIONS_MAP.with(|map| {
        let borrowed = map.borrow();
        let result = borrowed
            .iter()
            .find(|(_, position)| {
                let p = &position.0;
                p.user_principal == user &&
                p.protocol.eq_ignore_ascii_case(protocol) &&
                p.asset.eq_ignore_ascii_case(asset) &&
                p.chain_id == chain_id
            })
            .map(|(_, position)| position.0.clone());

        match &result {
            Some(pos) => ic_cdk::println!("✅ Found existing position: {}", pos.position_id),
            None => ic_cdk::println!("ℹ️ No existing position found"),
        }

        result
    })
}

// =============================================================================
// Position Synchronization Functions
// =============================================================================

/// Synchronize position after successful supply operation
/// Creates new position if doesn't exist, otherwise updates existing position
pub async fn sync_position_after_supply(
    user_principal: Principal,
    permissions_id: String,
    protocol: String,
    asset: String,
    token_address: String,
    chain_id: u64,
    amount_supplied: String, // Human-readable amount (e.g., "100.5")
) -> Result<(), String> {
    // Check if auto-sync is enabled
    if !is_auto_sync_enabled() {
        ic_cdk::println!("ℹ️ Position auto-sync is disabled, skipping synchronization");
        return Ok(());
    }

    ic_cdk::println!("🔄 Syncing position after supply: protocol={}, asset={}, amount={}",
        protocol, asset, amount_supplied);

    // Parse supplied amount
    let amount_f64: f64 = amount_supplied.parse()
        .map_err(|e| format!("Failed to parse amount: {}", e))?;

    if amount_f64 <= 0.0 {
        return Err("Supply amount must be positive".to_string());
    }

    // Check if position already exists
    match find_user_position(user_principal, &protocol, &asset, chain_id) {
        Some(mut existing_position) => {
            // Update existing position
            ic_cdk::println!("📝 Updating existing position: {}", existing_position.position_id);

            let old_size: f64 = existing_position.position_size.parse()
                .unwrap_or(0.0);
            let new_size = old_size + amount_f64;

            existing_position.position_size = format!("{}", new_size);
            existing_position.updated_at = now();

            // Save updated position
            USER_POSITIONS_MAP.with(|map| {
                map.borrow_mut().insert(
                    StorableString(existing_position.position_id.clone()),
                    StorableUserPosition(existing_position.clone())
                );
            });

            ic_cdk::println!("✅ Position updated: {} → {} (added {})",
                old_size, new_size, amount_f64);
            Ok(())
        }
        None => {
            // Create new position
            ic_cdk::println!("➕ Creating new position for user {}", user_principal);

            // Get user's EVM address
            let user_evm_address = PRINCIPAL_TO_ADDRESS_MAP.with(|map| {
                map.borrow()
                    .get(&StorablePrincipal(user_principal))
                    .map(|addr| addr.0.clone())
                    .ok_or_else(|| "User does not have an EVM address".to_string())
            })?;

            // Generate unique position ID
            let position_id = generate_position_id().await;
            let timestamp = now();

            let new_position = UserPosition {
                position_id: position_id.clone(),
                user_principal,
                user_evm_address,
                permissions_id,
                protocol,
                asset,
                token_address,
                chain_id,
                position_size: amount_supplied.clone(),
                tracked: false, // Default to false, user can enable later
                added_at: timestamp,
                updated_at: timestamp,
            };

            // Save new position
            USER_POSITIONS_MAP.with(|map| {
                map.borrow_mut().insert(
                    StorableString(position_id.clone()),
                    StorableUserPosition(new_position.clone())
                );
            });

            ic_cdk::println!("✅ New position created: {} with size {}",
                position_id, amount_supplied);
            Ok(())
        }
    }
}

/// Synchronize position after successful withdraw operation
/// Updates position size, deletes position if balance reaches zero
pub async fn sync_position_after_withdraw(
    user_principal: Principal,
    protocol: String,
    asset: String,
    chain_id: u64,
    amount_withdrawn: String, // Human-readable amount (e.g., "50.5")
) -> Result<(), String> {
    // Check if auto-sync is enabled
    if !is_auto_sync_enabled() {
        ic_cdk::println!("ℹ️ Position auto-sync is disabled, skipping synchronization");
        return Ok(());
    }

    ic_cdk::println!("🔄 Syncing position after withdraw: protocol={}, asset={}, amount={}",
        protocol, asset, amount_withdrawn);

    // Parse withdrawn amount
    let amount_f64: f64 = amount_withdrawn.parse()
        .map_err(|e| format!("Failed to parse amount: {}", e))?;

    if amount_f64 <= 0.0 {
        return Err("Withdraw amount must be positive".to_string());
    }

    // Find existing position
    match find_user_position(user_principal, &protocol, &asset, chain_id) {
        Some(mut existing_position) => {
            ic_cdk::println!("📝 Updating position after withdrawal: {}", existing_position.position_id);

            let old_size: f64 = existing_position.position_size.parse()
                .unwrap_or(0.0);
            let new_size = old_size - amount_f64;

            ic_cdk::println!("💰 Position size change: {} → {} (withdrawn {})",
                old_size, new_size, amount_f64);

            // If new size is zero or negative, delete the position
            if new_size <= 0.0001 { // Use small epsilon for floating point comparison
                ic_cdk::println!("🗑️ Position balance reached zero, deleting position: {}",
                    existing_position.position_id);

                USER_POSITIONS_MAP.with(|map| {
                    map.borrow_mut().remove(&StorableString(existing_position.position_id.clone()));
                });

                ic_cdk::println!("✅ Position deleted successfully");
            } else {
                // Update position with new size
                existing_position.position_size = format!("{}", new_size);
                existing_position.updated_at = now();

                USER_POSITIONS_MAP.with(|map| {
                    map.borrow_mut().insert(
                        StorableString(existing_position.position_id.clone()),
                        StorableUserPosition(existing_position.clone())
                    );
                });

                ic_cdk::println!("✅ Position updated with new size: {}", new_size);
            }

            Ok(())
        }
        None => {
            // No existing position found - this is a warning, not an error
            ic_cdk::println!("⚠️ No existing position found to update after withdrawal");
            ic_cdk::println!("   User may have withdrawn from a position not tracked in the system");
            Ok(())
        }
    }
}

// =============================================================================
// Helper Functions
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
