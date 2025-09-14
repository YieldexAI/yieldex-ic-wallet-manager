use candid::Principal;
use crate::{PERMISSIONS_MAP, StorableString};
use crate::services::{aave, compound};
use crate::services::rpc_service::{is_supported_chain, get_chain_name, SEPOLIA_CHAIN_ID, ARBITRUM_CHAIN_ID, BASE_CHAIN_ID, OPTIMISM_CHAIN_ID};

/// Unified rebalance interface that can move tokens between any supported protocols
/// 
/// This function provides a scalable approach by calling protocol-specific supply/withdraw methods
/// directly without needing separate functions for each direction.
pub async fn rebalance_tokens(
    amount: String,
    source_protocol: String,    // "AAVE" | "COMPOUND"
    target_protocol: String,    // "AAVE" | "COMPOUND"
    token: String,              // "USDC" | "LINK"
    permissions_id: String,
    user_principal: Principal
) -> Result<String, String> {
    ic_cdk::println!("🔄 Starting rebalance: {} {} from {} to {} for principal {}", 
                    amount, token, source_protocol, target_protocol, user_principal);
    
    // 1. Validate permissions and get chain_id
    ic_cdk::println!("✅ Step 1: Validating permissions...");
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
    ic_cdk::println!("✅ Step 1 Complete: Using chain_id: {} ({})", 
                    chain_id, get_chain_name(chain_id).unwrap_or("Unknown"));
    
    // 2. Validate chain support
    if !is_supported_chain(chain_id) {
        return Err(format!("Unsupported chain_id: {}", chain_id));
    }
    
    // 3. Validate protocols and token combination
    ic_cdk::println!("✅ Step 2: Validating rebalance route...");
    validate_rebalance_route(&source_protocol, &target_protocol, &token, chain_id)?;
    ic_cdk::println!("✅ Step 2 Complete: Route validated");
    
    // 4. Execute rebalance using generic protocol operations
    ic_cdk::println!("✅ Step 3: Executing rebalance...");
    
    // Step 3a: Withdraw from source protocol
    ic_cdk::println!("🏦 Step 3a: Withdrawing {} {} from {}...", amount, token, source_protocol);
    let withdraw_result = execute_protocol_withdraw(
        &source_protocol, 
        &token, 
        amount.clone(), 
        permissions_id.clone(), 
        user_principal
    ).await?;
    ic_cdk::println!("✅ Step 3a Complete: {}", withdraw_result);
    
    // Step 3b: Supply to target protocol
    ic_cdk::println!("🏛️ Step 3b: Supplying {} {} to {}...", amount, token, target_protocol);
    let supply_result = execute_protocol_supply(
        &target_protocol, 
        &token, 
        amount.clone(), 
        permissions_id, 
        user_principal
    ).await?;
    ic_cdk::println!("✅ Step 3b Complete: {}", supply_result);
    
    let success_message = format!(
        "✅ Successfully rebalanced {} {} from {} to {}! Withdraw: {} | Supply: {}",
        amount, token, source_protocol, target_protocol, withdraw_result, supply_result
    );
    
    ic_cdk::println!("🎉 Rebalance completed successfully");
    Ok(success_message)
}

/// Generic protocol withdraw operation - calls the appropriate protocol's withdraw function
async fn execute_protocol_withdraw(
    protocol: &str,
    token: &str,
    amount: String,
    permissions_id: String,
    user_principal: Principal
) -> Result<String, String> {
    match (protocol.to_uppercase().as_str(), token.to_uppercase().as_str()) {
        ("AAVE", "LINK") => {
            aave::withdraw_link_from_aave_with_permissions(
                amount, permissions_id, user_principal
            ).await
        },
        ("COMPOUND", "USDC") => {
            compound::withdraw_usdc_from_compound_with_permissions(
                amount, permissions_id, user_principal
            ).await
        },
        ("AAVE", "USDC") => {
            Err("AAVE USDC operations not implemented yet. Currently only LINK is supported in AAVE.".to_string())
        },
        ("COMPOUND", "LINK") => {
            Err("Compound LINK operations not implemented yet. Currently only USDC is supported in Compound.".to_string())
        },
        _ => {
            Err(format!("Unsupported protocol-token combination: {} - {}", protocol, token))
        }
    }
}

/// Generic protocol supply operation - calls the appropriate protocol's supply function
async fn execute_protocol_supply(
    protocol: &str,
    token: &str,
    amount: String,
    permissions_id: String,
    user_principal: Principal
) -> Result<String, String> {
    match (protocol.to_uppercase().as_str(), token.to_uppercase().as_str()) {
        ("AAVE", "LINK") => {
            aave::supply_link_to_aave_with_permissions(
                amount, permissions_id, user_principal
            ).await
        },
        ("COMPOUND", "USDC") => {
            compound::supply_usdc_to_compound_with_permissions(
                amount, permissions_id, user_principal
            ).await
        },
        ("AAVE", "USDC") => {
            Err("AAVE USDC operations not implemented yet. Currently only LINK is supported in AAVE.".to_string())
        },
        ("COMPOUND", "LINK") => {
            Err("Compound LINK operations not implemented yet. Currently only USDC is supported in Compound.".to_string())
        },
        _ => {
            Err(format!("Unsupported protocol-token combination: {} - {}", protocol, token))
        }
    }
}

/// Validate that the rebalance route is supported
fn validate_rebalance_route(
    source_protocol: &str,
    target_protocol: &str,
    token: &str,
    chain_id: u64
) -> Result<(), String> {
    // Validate protocols
    let valid_protocols = ["AAVE", "COMPOUND"];
    if !valid_protocols.contains(&source_protocol.to_uppercase().as_str()) {
        return Err(format!("Invalid source protocol: {}. Supported: {:?}", source_protocol, valid_protocols));
    }
    if !valid_protocols.contains(&target_protocol.to_uppercase().as_str()) {
        return Err(format!("Invalid target protocol: {}. Supported: {:?}", target_protocol, valid_protocols));
    }
    
    // Validate token
    let valid_tokens = ["USDC", "LINK"];
    if !valid_tokens.contains(&token.to_uppercase().as_str()) {
        return Err(format!("Invalid token: {}. Supported: {:?}", token, valid_tokens));
    }
    
    // Validate that source and target are different
    if source_protocol.eq_ignore_ascii_case(target_protocol) {
        return Err("Source and target protocols must be different".to_string());
    }
    
    // Validate chain-specific protocol support
    match (source_protocol.to_uppercase().as_str(), target_protocol.to_uppercase().as_str(), chain_id) {
        ("AAVE", _, ARBITRUM_CHAIN_ID | BASE_CHAIN_ID | OPTIMISM_CHAIN_ID) => {
            return Err("AAVE is not supported on this chain. AAVE is only available on Sepolia.".to_string())
        },
        (_, "AAVE", ARBITRUM_CHAIN_ID | BASE_CHAIN_ID | OPTIMISM_CHAIN_ID) => {
            return Err("AAVE is not supported on this chain. AAVE is only available on Sepolia.".to_string())
        },
        ("COMPOUND", _, SEPOLIA_CHAIN_ID | BASE_CHAIN_ID | OPTIMISM_CHAIN_ID) => {
            return Err("Compound is not supported on this chain. Compound is only available on Arbitrum.".to_string())
        },
        (_, "COMPOUND", SEPOLIA_CHAIN_ID | BASE_CHAIN_ID | OPTIMISM_CHAIN_ID) => {
            return Err("Compound is not supported on this chain. Compound is only available on Arbitrum.".to_string())
        },
        _ => {}
    }
    
    // Validate specific token-protocol combinations
    validate_protocol_token_support(source_protocol, token)?;
    validate_protocol_token_support(target_protocol, token)?;
    
    Ok(())
}

/// Validate that a protocol supports a specific token
fn validate_protocol_token_support(protocol: &str, token: &str) -> Result<(), String> {
    match (protocol.to_uppercase().as_str(), token.to_uppercase().as_str()) {
        ("AAVE", "LINK") => Ok(()),
        ("COMPOUND", "USDC") => Ok(()),
        ("AAVE", "USDC") => {
            Err("AAVE USDC operations not implemented yet. Currently only LINK is supported in AAVE.".to_string())
        },
        ("COMPOUND", "LINK") => {
            Err("Compound LINK operations not implemented yet. Currently only USDC is supported in Compound.".to_string())
        },
        _ => {
            Err(format!("Unsupported protocol-token combination: {} - {}", protocol, token))
        }
    }
}

/// Get supported rebalance routes for a specific chain
pub fn get_supported_rebalance_routes(chain_id: u64) -> Vec<(String, String, String)> {
    let mut routes = Vec::new();
    
    // Check each possible combination
    let protocols = ["AAVE", "COMPOUND"];
    let tokens = ["USDC", "LINK"];
    
    for &source in &protocols {
        for &target in &protocols {
            for &token in &tokens {
                if source != target {
                    if validate_rebalance_route(source, target, token, chain_id).is_ok() {
                        routes.push((
                            source.to_string(),
                            target.to_string(), 
                            token.to_string()
                        ));
                    }
                }
            }
        }
    }
    
    routes
}

/// Get rebalance route status for a specific chain
pub fn get_rebalance_route_status(
    source_protocol: &str,
    target_protocol: &str,
    token: &str,
    chain_id: u64
) -> String {
    match validate_rebalance_route(source_protocol, target_protocol, token, chain_id) {
        Ok(_) => "Supported".to_string(),
        Err(reason) => format!("Not supported: {}", reason)
    }
}

/// Get all protocol-token combinations supported on a chain
pub fn get_protocol_token_support(chain_id: u64) -> Vec<(String, String)> {
    let mut combinations = Vec::new();
    
    match chain_id {
        SEPOLIA_CHAIN_ID => {
            combinations.push(("AAVE".to_string(), "LINK".to_string()));
        },
        ARBITRUM_CHAIN_ID => {
            combinations.push(("COMPOUND".to_string(), "USDC".to_string()));
        },
        _ => {}
    }
    
    combinations
}