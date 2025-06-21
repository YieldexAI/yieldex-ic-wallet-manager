use candid::Principal;
use crate::{
    Permissions, ProtocolPermission, PERMISSIONS_MAP, StorableString, StorablePermissions, now
};

/// Check if caller is the owner of permissions
pub fn is_permissions_owner(permissions_id: &str, caller: Principal) -> Result<bool, String> {
    PERMISSIONS_MAP.with(|map| {
        map.borrow()
            .get(&StorableString(permissions_id.to_string()))
            .map_or(
                Err("Permissions not found".to_string()),
                |p| {
                    if p.0.owner == caller {
                        Ok(true)
                    } else {
                        Err("Access denied: not the owner".to_string())
                    }
                }
            )
    })
}

/// Check daily limits and per-transaction limits for protocol
pub fn check_daily_limits(
    permissions: &Permissions,
    protocol_address: &str,
    amount: u64
) -> Result<bool, String> {
    let now = ic_cdk::api::time() / 1_000_000;
    
    for perm in &permissions.protocol_permissions {
        if perm.protocol_address == protocol_address {
            // Check daily limit reset
            if let Some(daily_limit) = perm.daily_limit {
                let today_start = now - (now % 86400); // Start of day
                let mut used_today = perm.total_used_today;
                
                if perm.last_reset_date < today_start {
                    used_today = 0; // Reset counter
                }
                
                if used_today + amount > daily_limit {
                    return Err(format!("Daily limit exceeded. Used: {}, Limit: {}, Requested: {}", 
                        used_today, daily_limit, amount));
                }
            }
            
            // Check per-transaction limit
            if let Some(max_tx) = perm.max_amount_per_tx {
                if amount > max_tx {
                    return Err(format!("Transaction amount {} exceeds max limit {}", amount, max_tx));
                }
            }
            
            return Ok(true);
        }
    }
    
    Err("Protocol not found in permissions".to_string())
}

/// Check permission to perform protocol operation
pub fn verify_protocol_permission(
    permissions_id: String, 
    protocol_address: String, 
    function_name: String,
    amount: u64,
    caller: Principal
) -> Result<bool, String> {
    // Check ownership
    if let Err(e) = is_permissions_owner(&permissions_id, caller) {
        return Err(e);
    }
    
    // Get permissions
    let permissions = PERMISSIONS_MAP.with(|map| {
        map.borrow()
            .get(&StorableString(permissions_id.clone()))
            .map(|p| p.0.clone())
            .ok_or_else(|| "Permissions not found".to_string())
    })?;
    
    // Search for protocol permission
    for perm in &permissions.protocol_permissions {
        if perm.protocol_address == protocol_address {
            // Check allowed functions
            if !perm.allowed_functions.contains(&function_name) {
                return Err(format!("Function '{}' not allowed for protocol {}", function_name, protocol_address));
            }
            
            // Check
            return check_daily_limits(&permissions, &protocol_address, amount);
        }
    }
    
    Err(format!("Protocol {} not found in permissions", protocol_address))
}

/// Add permission for protocol
pub fn add_protocol_permission(
    permissions_id: String,
    protocol_permission: ProtocolPermission,
    caller: Principal
) -> Result<bool, String> {
    // Check ownership
    if let Err(e) = is_permissions_owner(&permissions_id, caller) {
        return Err(e);
    }
    
    // Get and update permissions
    let mut permissions = PERMISSIONS_MAP.with(|map| {
        map.borrow()
            .get(&StorableString(permissions_id.clone()))
            .map(|p| p.0.clone())
            .ok_or_else(|| "Permissions not found".to_string())
    })?;
    
    // Check if permission already exists for this protocol
    for perm in &permissions.protocol_permissions {
        if perm.protocol_address == protocol_permission.protocol_address {
            return Err(format!("Protocol permission for {} already exists", protocol_permission.protocol_address));
        }
    }
    
    // Add new permission
    permissions.protocol_permissions.push(protocol_permission);
    permissions.updated_at = now();
    
    // Save updated permissions
    PERMISSIONS_MAP.with(|map| {
        map.borrow_mut().insert(
            StorableString(permissions_id), 
            StorablePermissions(permissions)
        );
    });
    
    Ok(true)
}

/// Update used limit for today
pub fn set_daily_usage(
    permissions_id: String,
    protocol_address: String,
    amount_used: u64,
    caller: Principal
) -> Result<bool, String> {
    // Check ownership
    if let Err(e) = is_permissions_owner(&permissions_id, caller) {
        return Err(e);
    }
    
    // Get permissions
    let mut permissions = PERMISSIONS_MAP.with(|map| {
        map.borrow()
            .get(&StorableString(permissions_id.clone()))
            .map(|p| p.0.clone())
            .ok_or_else(|| "Permissions not found".to_string())
    })?;
    
    let now = ic_cdk::api::time() / 1_000_000;
    let today_start = now - (now % 86400); // Start of day
    
    // Search and update protocol permission
    for perm in &mut permissions.protocol_permissions {
        if perm.protocol_address == protocol_address {
            // Reset counter if new day started
            if perm.last_reset_date < today_start {
                perm.total_used_today = 0;
                perm.last_reset_date = today_start;
            }
            
            // Update used amount
            perm.total_used_today += amount_used;
            permissions.updated_at = now;
            
            // Save updated permissions
            PERMISSIONS_MAP.with(|map| {
                map.borrow_mut().insert(
                    StorableString(permissions_id), 
                    StorablePermissions(permissions)
                );
            });
            
            return Ok(true);
        }
    }
    
    Err(format!("Protocol {} not found in permissions", protocol_address))
}

