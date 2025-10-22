use candid::Principal;
use alloy::primitives::{address, Address};
use crate::{PERMISSIONS_MAP, StorableString};
use crate::types::{Recommendation, ExecutionResult, RecommendationType};
use crate::services::{aave, compound};
use crate::services::rpc_service::{is_supported_chain, get_chain_name, SEPOLIA_CHAIN_ID, ARBITRUM_CHAIN_ID};

// =============================================================================
// Recommendation-based Rebalancing Functions
// =============================================================================

/// Normalize protocol name to internal format
fn normalize_protocol_name(protocol: &str) -> Result<&str, String> {
    match protocol.to_lowercase().as_str() {
        "aave-v3" | "aave" => Ok("AAVE"),
        "compound-v3" | "compound" => Ok("COMPOUND"),
        _ => Err(format!("Unknown protocol: {}", protocol))
    }
}

/// Get USDC address for a specific chain
fn get_usdc_address(chain_id: u64) -> Result<Address, String> {
    match chain_id {
        ARBITRUM_CHAIN_ID => {
            // Native USDC on Arbitrum
            Ok(address!("af88d065e77c8cC2239327C5EDb3A432268e5831"))
        },
        SEPOLIA_CHAIN_ID => {
            // USDC on Sepolia testnet
            Ok(address!("94a9D9AC8a22534E3FaCa9954e183B2c3736704F"))
        },
        _ => Err(format!("USDC not configured for chain_id: {}", chain_id))
    }
}

/// Extract transaction hash from result string
fn extract_tx_hash(result: &str) -> Option<String> {
    // Result strings typically contain "Transaction: 0x..." or similar
    if let Some(start) = result.find("0x") {
        let hash_part = &result[start..];
        // Take until first whitespace or end of string
        let end = hash_part.find(char::is_whitespace).unwrap_or(hash_part.len());
        Some(hash_part[..end.min(66)].to_string()) // 0x + 64 chars = 66
    } else {
        None
    }
}

/// Validate recommendation structure and parameters
pub fn validate_recommendation(recommendation: &Recommendation) -> Result<(), String> {
    ic_cdk::println!("🔍 Validating recommendation...");

    // Check that either protocols or assets are different (otherwise it's a no-op)
    let same_protocol = recommendation.from_protocol.eq_ignore_ascii_case(&recommendation.to_protocol);
    let same_asset = recommendation.asset.eq_ignore_ascii_case(&recommendation.to_asset);

    if same_protocol && same_asset {
        return Err("Source and target must differ: either different protocols or different assets".to_string());
    }

    // Validate protocols
    normalize_protocol_name(&recommendation.from_protocol)?;
    normalize_protocol_name(&recommendation.to_protocol)?;

    // Check position_size is valid
    let amount: f64 = recommendation.position_size.parse()
        .map_err(|_| format!("Invalid position_size: {}", recommendation.position_size))?;

    if amount <= 0.0 {
        return Err(format!("position_size must be positive, got: {}", amount));
    }

    // Validate cross-chain field
    if let Some(ref to_chain) = recommendation.to_chain {
        if to_chain != &recommendation.from_chain {
            return Err("Cross-chain transfers are not yet supported".to_string());
        }
    }

    ic_cdk::println!("✅ Recommendation validation successful");
    Ok(())
}

/// Execute withdraw from protocol
async fn execute_protocol_withdraw(
    protocol: &str,
    amount: String,
    permissions_id: String,
    user_principal: Principal,
    chain_id: u64
) -> Result<String, String> {
    let normalized_protocol = normalize_protocol_name(protocol)?;

    ic_cdk::println!("🏦 Executing withdraw from {} protocol...", normalized_protocol);

    match normalized_protocol {
        "AAVE" => {
            let usdc_addr = get_usdc_address(chain_id)?;
            aave::withdraw_from_aave_with_permissions(
                usdc_addr,
                "USDC".to_string(),
                amount,
                permissions_id,
                user_principal,
                chain_id
            ).await
        },
        "COMPOUND" => {
            compound::withdraw_usdc_from_compound_with_permissions(
                amount,
                permissions_id,
                user_principal
            ).await
        },
        _ => Err(format!("Unsupported protocol for withdraw: {}", protocol))
    }
}

/// Execute supply to protocol
async fn execute_protocol_supply(
    protocol: &str,
    amount: String,
    permissions_id: String,
    user_principal: Principal,
    chain_id: u64
) -> Result<String, String> {
    let normalized_protocol = normalize_protocol_name(protocol)?;

    ic_cdk::println!("🏛️ Executing supply to {} protocol...", normalized_protocol);

    match normalized_protocol {
        "AAVE" => {
            let usdc_addr = get_usdc_address(chain_id)?;
            aave::supply_to_aave_with_permissions(
                usdc_addr,
                "USDC".to_string(),
                amount,
                permissions_id,
                user_principal,
                chain_id
            ).await
        },
        "COMPOUND" => {
            compound::supply_usdc_to_compound_with_permissions(
                amount,
                permissions_id,
                user_principal
            ).await
        },
        _ => Err(format!("Unsupported protocol for supply: {}", protocol))
    }
}

/// Execute same-chain same-asset rebalance flow
async fn execute_same_chain_same_asset(
    recommendation: &Recommendation,
    permissions_id: String,
    user_principal: Principal,
    chain_id: u64
) -> Result<ExecutionResult, String> {
    ic_cdk::println!("🔄 Starting same-chain same-asset rebalance flow");
    ic_cdk::println!("  From: {} | To: {} | Amount: {} USDC",
        recommendation.from_protocol, recommendation.to_protocol, recommendation.position_size);

    let mut result = ExecutionResult {
        status: "pending".to_string(),
        withdraw_tx: None,
        swap_tx: None,
        supply_tx: None,
        amount_transferred: Some(recommendation.position_size.clone()),
        actual_gas_cost: None,
        error_details: None,
    };

    // Step 1: Withdraw from source protocol
    ic_cdk::println!("📤 Step 1: Withdrawing from {}...", recommendation.from_protocol);
    match execute_protocol_withdraw(
        &recommendation.from_protocol,
        recommendation.position_size.clone(),
        permissions_id.clone(),
        user_principal,
        chain_id
    ).await {
        Ok(withdraw_result) => {
            ic_cdk::println!("✅ Withdraw successful: {}", withdraw_result);
            result.withdraw_tx = extract_tx_hash(&withdraw_result);

            // Sync position after successful withdraw
            ic_cdk::println!("🔄 Syncing source position after withdrawal...");
            match crate::services::position_sync::sync_position_after_withdraw(
                user_principal,
                recommendation.from_protocol.clone(),
                recommendation.asset.clone(),
                chain_id,
                recommendation.position_size.clone(),
            ).await {
                Ok(_) => ic_cdk::println!("✅ Source position synced after withdrawal"),
                Err(e) => ic_cdk::println!("⚠️ Warning: Source position sync failed: {}", e),
            }
        },
        Err(e) => {
            ic_cdk::println!("❌ Withdraw failed: {}", e);
            result.status = "failed".to_string();
            result.error_details = Some(format!("Withdraw failed: {}", e));
            return Ok(result);
        }
    }

    // Step 2: Supply to target protocol
    ic_cdk::println!("📥 Step 2: Supplying to {}...", recommendation.to_protocol);
    match execute_protocol_supply(
        &recommendation.to_protocol,
        recommendation.position_size.clone(),
        permissions_id.clone(),
        user_principal,
        chain_id
    ).await {
        Ok(supply_result) => {
            ic_cdk::println!("✅ Supply successful: {}", supply_result);
            result.supply_tx = extract_tx_hash(&supply_result);
            result.status = "success".to_string();

            // Sync position after successful supply
            ic_cdk::println!("🔄 Syncing target position after supply...");
            let token_address = get_usdc_address(chain_id)
                .map(|addr| format!("0x{:x}", addr))
                .unwrap_or_else(|_| "unknown".to_string());

            match crate::services::position_sync::sync_position_after_supply(
                user_principal,
                permissions_id,
                recommendation.to_protocol.clone(),
                recommendation.to_asset.clone(),
                token_address,
                chain_id,
                recommendation.position_size.clone(),
            ).await {
                Ok(_) => ic_cdk::println!("✅ Target position synced after supply"),
                Err(e) => ic_cdk::println!("⚠️ Warning: Target position sync failed: {}", e),
            }
        },
        Err(e) => {
            ic_cdk::println!("❌ Supply failed: {}", e);
            result.status = "partial".to_string();
            result.error_details = Some(format!("Supply failed: {}. Funds withdrawn but not supplied.", e));
        }
    }

    ic_cdk::println!("🎉 Rebalance flow completed with status: {}", result.status);
    Ok(result)
}

/// Main recommendation execution function
pub async fn execute_recommendation(
    recommendation: Recommendation,
    permissions_id: String,
    user_principal: Principal
) -> Result<ExecutionResult, String> {
    ic_cdk::println!("🚀 Starting recommendation execution");
    ic_cdk::println!("  Asset: {} → {}", recommendation.asset, recommendation.to_asset);
    ic_cdk::println!("  Protocol: {} → {}", recommendation.from_protocol, recommendation.to_protocol);
    ic_cdk::println!("  Amount: {} USDC", recommendation.position_size);
    ic_cdk::println!("  User: {}", user_principal);

    // Step 1: Validate recommendation
    validate_recommendation(&recommendation)?;

    // Step 2: Get permissions and chain_id
    ic_cdk::println!("🔐 Getting permissions...");
    let permissions = PERMISSIONS_MAP.with(|map| {
        map.borrow()
            .get(&StorableString(permissions_id.clone()))
            .ok_or_else(|| "Permissions not found".to_string())
            .map(|p| p.0.clone())
    })?;

    // Check ownership
    if permissions.owner != user_principal {
        return Err("Not authorized to use these permissions".to_string());
    }

    let chain_id = permissions.chain_id;
    ic_cdk::println!("✅ Using chain_id: {} ({})",
        chain_id, get_chain_name(chain_id).unwrap_or("Unknown"));

    // Step 3: Validate chain support
    if !is_supported_chain(chain_id) {
        return Err(format!("Unsupported chain_id: {}", chain_id));
    }

    // Step 4: Determine and execute flow
    match recommendation.recommendation_type {
        RecommendationType::StandardTransfer => {
            // For same chain, same asset (USDC → USDC)
            if recommendation.asset == recommendation.to_asset {
                execute_same_chain_same_asset(
                    &recommendation,
                    permissions_id,
                    user_principal,
                    chain_id
                ).await
            } else {
                // Future: swap flow for different assets
                Err("Asset swap not yet supported".to_string())
            }
        }
        RecommendationType::CrossChainTransfer => {
            Err("Cross-chain transfers are not yet supported".to_string())
        }
    }
}