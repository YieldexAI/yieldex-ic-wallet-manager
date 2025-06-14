use candid::Principal;
use crate::{
    Permissions, ProtocolPermission, PERMISSIONS_MAP, StorableString, StorablePermissions, now
};

/// Проверить является ли caller владельцем permissions
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

/// Проверить daily limits и per-transaction limits для протокола
pub fn check_daily_limits(
    permissions: &Permissions,
    protocol_address: &str,
    amount: u64
) -> Result<bool, String> {
    let now = ic_cdk::api::time() / 1_000_000;
    
    for perm in &permissions.protocol_permissions {
        if perm.protocol_address == protocol_address {
            // Проверяем сброс daily limit
            if let Some(daily_limit) = perm.daily_limit {
                let today_start = now - (now % 86400); // Начало дня
                let mut used_today = perm.total_used_today;
                
                if perm.last_reset_date < today_start {
                    used_today = 0; // Сбрасываем счетчик
                }
                
                if used_today + amount > daily_limit {
                    return Err(format!("Daily limit exceeded. Used: {}, Limit: {}, Requested: {}", 
                        used_today, daily_limit, amount));
                }
            }
            
            // Проверяем per-transaction limit
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

/// Проверить разрешение на выполнение операции с протоколом
pub fn verify_protocol_permission(
    permissions_id: String, 
    protocol_address: String, 
    function_name: String,
    amount: u64,
    caller: Principal
) -> Result<bool, String> {
    // Проверяем ownership
    if let Err(e) = is_permissions_owner(&permissions_id, caller) {
        return Err(e);
    }
    
    // Получаем permissions
    let permissions = PERMISSIONS_MAP.with(|map| {
        map.borrow()
            .get(&StorableString(permissions_id.clone()))
            .map(|p| p.0.clone())
            .ok_or_else(|| "Permissions not found".to_string())
    })?;
    
    // Ищем protocol permission
    for perm in &permissions.protocol_permissions {
        if perm.protocol_address == protocol_address {
            // Проверяем разрешенные функции
            if !perm.allowed_functions.contains(&function_name) {
                return Err(format!("Function '{}' not allowed for protocol {}", function_name, protocol_address));
            }
            
            // Проверяем лимиты
            return check_daily_limits(&permissions, &protocol_address, amount);
        }
    }
    
    Err(format!("Protocol {} not found in permissions", protocol_address))
}

/// Добавить разрешение для протокола
pub fn add_protocol_permission(
    permissions_id: String,
    protocol_permission: ProtocolPermission,
    caller: Principal
) -> Result<bool, String> {
    // Проверяем ownership
    if let Err(e) = is_permissions_owner(&permissions_id, caller) {
        return Err(e);
    }
    
    // Получаем и обновляем permissions
    let mut permissions = PERMISSIONS_MAP.with(|map| {
        map.borrow()
            .get(&StorableString(permissions_id.clone()))
            .map(|p| p.0.clone())
            .ok_or_else(|| "Permissions not found".to_string())
    })?;
    
    // Проверяем, не существует ли уже permission для этого протокола
    for perm in &permissions.protocol_permissions {
        if perm.protocol_address == protocol_permission.protocol_address {
            return Err(format!("Protocol permission for {} already exists", protocol_permission.protocol_address));
        }
    }
    
    // Добавляем новое разрешение
    permissions.protocol_permissions.push(protocol_permission);
    permissions.updated_at = now();
    
    // Сохраняем обновленные permissions
    PERMISSIONS_MAP.with(|map| {
        map.borrow_mut().insert(
            StorableString(permissions_id), 
            StorablePermissions(permissions)
        );
    });
    
    Ok(true)
}

/// Обновить использованный лимит за сегодня
pub fn set_daily_usage(
    permissions_id: String,
    protocol_address: String,
    amount_used: u64,
    caller: Principal
) -> Result<bool, String> {
    // Проверяем ownership
    if let Err(e) = is_permissions_owner(&permissions_id, caller) {
        return Err(e);
    }
    
    // Получаем permissions
    let mut permissions = PERMISSIONS_MAP.with(|map| {
        map.borrow()
            .get(&StorableString(permissions_id.clone()))
            .map(|p| p.0.clone())
            .ok_or_else(|| "Permissions not found".to_string())
    })?;
    
    let now = ic_cdk::api::time() / 1_000_000;
    let today_start = now - (now % 86400); // Начало дня
    
    // Ищем и обновляем protocol permission
    for perm in &mut permissions.protocol_permissions {
        if perm.protocol_address == protocol_address {
            // Сбрасываем счетчик если начался новый день
            if perm.last_reset_date < today_start {
                perm.total_used_today = 0;
                perm.last_reset_date = today_start;
            }
            
            // Обновляем использованную сумму
            perm.total_used_today += amount_used;
            permissions.updated_at = now;
            
            // Сохраняем обновленные permissions
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

