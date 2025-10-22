use std::cell::RefCell;
use std::collections::HashMap;
use alloy::primitives::Address;

/// Nonce cache entry with state tracking
#[derive(Debug, Clone, Copy, PartialEq)]
enum NonceState {
    /// Available for use
    Available(u64),
    /// Reserved for a pending transaction
    Reserved(u64),
}

// Global nonce cache: (address, chain_id) -> NonceState
thread_local! {
    static NONCE_CACHE: RefCell<HashMap<(String, u64), NonceState>> = RefCell::new(HashMap::new());
}

/// Generate cache key from address and chain_id
fn cache_key(address: Address, chain_id: u64) -> (String, u64) {
    (format!("{:x}", address), chain_id)
}

/// Get the next available nonce for an address on a specific chain
///
/// This function:
/// 1. Checks the local cache for a cached nonce
/// 2. If cache miss or invalidated, fetches fresh nonce from network
/// 3. Returns the next available nonce to use
///
/// NOTE: After getting a nonce, you MUST call either:
/// - `commit_nonce()` after successfully sending the transaction
/// - `rollback_nonce()` if the transaction failed to send
pub async fn get_next_nonce<T, N, P>(
    address: Address,
    provider: &P,
    chain_id: u64
) -> Result<u64, String>
where
    T: alloy::transports::Transport + Clone,
    N: alloy::network::Network,
    P: alloy::providers::Provider<T, N>,
{
    let key = cache_key(address, chain_id);

    // Try to get cached nonce
    let cached_nonce = NONCE_CACHE.with(|cache| {
        cache.borrow().get(&key).copied()
    });

    match cached_nonce {
        Some(NonceState::Available(nonce)) => {
            ic_cdk::println!("🔧 Using cached nonce: {} for address 0x{:x} on chain {}",
                nonce, address, chain_id);
            Ok(nonce)
        }
        Some(NonceState::Reserved(nonce)) => {
            // There's a reserved nonce, use the next one
            let next_nonce = nonce + 1;
            ic_cdk::println!("⚠️ Nonce {} is reserved, using next available: {} for address 0x{:x} on chain {}",
                nonce, next_nonce, address, chain_id);
            Ok(next_nonce)
        }
        None => {
            // Cache miss - fetch from network
            ic_cdk::println!("🌐 Fetching fresh nonce from network for address 0x{:x} on chain {}...",
                address, chain_id);

            let fresh_nonce = provider.get_transaction_count(address)
                .await
                .map_err(|e| format!("Failed to get nonce from network: {}", e))?;

            ic_cdk::println!("✅ Got fresh nonce from network: {} for address 0x{:x} on chain {}",
                fresh_nonce, address, chain_id);

            // Don't cache yet - let reserve_nonce or commit_nonce handle caching
            Ok(fresh_nonce)
        }
    }
}

/// Reserve a nonce before sending a transaction
///
/// Call this BEFORE sending a transaction to mark the nonce as "in-use".
/// This prevents other concurrent operations from using the same nonce.
///
/// You MUST follow up with either:
/// - `commit_nonce()` if send() succeeded
/// - `rollback_nonce()` if send() failed
pub fn reserve_nonce(address: Address, chain_id: u64, nonce: u64) {
    let key = cache_key(address, chain_id);

    NONCE_CACHE.with(|cache| {
        cache.borrow_mut().insert(key.clone(), NonceState::Reserved(nonce));
    });

    ic_cdk::println!("🔒 Reserved nonce {} for address 0x{:x} on chain {}",
        nonce, address, chain_id);
}

/// Commit a nonce after successfully sending a transaction
///
/// Call this AFTER send().await returns Ok(builder).
/// This updates the cache to the NEXT available nonce.
///
/// IMPORTANT: Call this even if the transaction reverts in the blockchain!
/// A reverted transaction still consumes the nonce.
pub fn commit_nonce(address: Address, chain_id: u64, used_nonce: u64) {
    let key = cache_key(address, chain_id);
    let next_nonce = used_nonce + 1;

    NONCE_CACHE.with(|cache| {
        cache.borrow_mut().insert(key.clone(), NonceState::Available(next_nonce));
    });

    ic_cdk::println!("✅ Committed nonce {}, next available: {} for address 0x{:x} on chain {}",
        used_nonce, next_nonce, address, chain_id);
}

/// Rollback a reserved nonce after failing to send a transaction
///
/// Call this AFTER send().await returns Err(e).
/// This releases the reserved nonce back to available state.
///
/// Only call this if the transaction FAILED TO SEND (e.g., gas estimation failed).
/// Do NOT call this if send() succeeded but the transaction reverted later.
pub fn rollback_nonce(address: Address, chain_id: u64, nonce: u64) {
    let key = cache_key(address, chain_id);

    NONCE_CACHE.with(|cache| {
        cache.borrow_mut().insert(key.clone(), NonceState::Available(nonce));
    });

    ic_cdk::println!("🔄 Rolled back nonce {} for address 0x{:x} on chain {}",
        nonce, address, chain_id);
}

/// Invalidate the nonce cache for a specific address on a chain
///
/// Call this when:
/// - You get a "nonce too low" error (cache is stale)
/// - You want to force fetching a fresh nonce from network
/// - You suspect the cache is out of sync
pub fn invalidate_cache(address: Address, chain_id: u64) {
    let key = cache_key(address, chain_id);

    NONCE_CACHE.with(|cache| {
        cache.borrow_mut().remove(&key);
    });

    ic_cdk::println!("🗑️ Invalidated nonce cache for address 0x{:x} on chain {}",
        address, chain_id);
}

/// Clear all nonce caches (useful for testing or reset)
#[allow(dead_code)]
pub fn clear_all_caches() {
    NONCE_CACHE.with(|cache| {
        cache.borrow_mut().clear();
    });

    ic_cdk::println!("🗑️ Cleared all nonce caches");
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    #[test]
    fn test_cache_key_generation() {
        let addr = address!("0x1234567890123456789012345678901234567890");
        let chain_id = 42161u64;
        let key = cache_key(addr, chain_id);

        assert_eq!(key.0, "1234567890123456789012345678901234567890");
        assert_eq!(key.1, 42161);
    }

    #[test]
    fn test_reserve_and_commit() {
        clear_all_caches();

        let addr = address!("0x1234567890123456789012345678901234567890");
        let chain_id = 42161u64;
        let key = cache_key(addr, chain_id);

        // Reserve nonce 10
        reserve_nonce(addr, chain_id, 10);

        let state = NONCE_CACHE.with(|cache| {
            cache.borrow().get(&key).copied()
        });
        assert_eq!(state, Some(NonceState::Reserved(10)));

        // Commit nonce 10
        commit_nonce(addr, chain_id, 10);

        let state = NONCE_CACHE.with(|cache| {
            cache.borrow().get(&key).copied()
        });
        assert_eq!(state, Some(NonceState::Available(11)));
    }

    #[test]
    fn test_reserve_and_rollback() {
        clear_all_caches();

        let addr = address!("0x1234567890123456789012345678901234567890");
        let chain_id = 42161u64;
        let key = cache_key(addr, chain_id);

        // Reserve nonce 10
        reserve_nonce(addr, chain_id, 10);

        let state = NONCE_CACHE.with(|cache| {
            cache.borrow().get(&key).copied()
        });
        assert_eq!(state, Some(NonceState::Reserved(10)));

        // Rollback nonce 10
        rollback_nonce(addr, chain_id, 10);

        let state = NONCE_CACHE.with(|cache| {
            cache.borrow().get(&key).copied()
        });
        assert_eq!(state, Some(NonceState::Available(10)));
    }

    #[test]
    fn test_invalidate_cache() {
        clear_all_caches();

        let addr = address!("0x1234567890123456789012345678901234567890");
        let chain_id = 42161u64;
        let key = cache_key(addr, chain_id);

        // Set a nonce
        reserve_nonce(addr, chain_id, 10);

        let state = NONCE_CACHE.with(|cache| {
            cache.borrow().get(&key).copied()
        });
        assert!(state.is_some());

        // Invalidate
        invalidate_cache(addr, chain_id);

        let state = NONCE_CACHE.with(|cache| {
            cache.borrow().get(&key).copied()
        });
        assert!(state.is_none());
    }
}
