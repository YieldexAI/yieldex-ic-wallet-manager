use pocket_ic::PocketIc;
use std::fs;
use std::path::PathBuf;
use candid::{Principal, Decode, Encode};

// Define the principal ID for tests
const USER_PRINCIPAL: &str = "2vxsx-fae";
// Base path to the WASM file (relative to the project root)
const WASM_PATH_RELATIVE: &str = ".dfx/local/canisters/yieldex-ic-wallet-manager-backend/yieldex-ic-wallet-manager-backend.wasm";

#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper function to get the absolute path to the WASM file
    fn get_wasm_path() -> PathBuf {
        // Get the current directory (tests directory)
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        // Navigate up one level to the project root
        let project_root = current_dir.parent().expect("Failed to get project root directory");
        project_root.join(WASM_PATH_RELATIVE)
    }

    #[test]
    fn test_get_evm_address() {
        // Initialize PocketIC
        let pic = PocketIc::new();
        
        // Create a new canister
        let canister_id = pic.create_canister();
        pic.add_cycles(canister_id, 2_000_000_000_000);
        
        // Get and check the WASM file path
        let wasm_path = get_wasm_path();
        if !wasm_path.exists() {
            panic!("WASM file not found at path: {}. Make sure to run 'dfx build' first.", 
                   wasm_path.display());
        }
        
        // Load WASM file
        let wasm = fs::read(&wasm_path).expect("Could not read WASM file");
        
        // Install the canister
        pic.install_canister(canister_id, wasm, vec![], None);
        
        // Call get_evm_address() query method
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        
        // Аргументы должны быть пустыми для get_evm_address
        let result = pic.query_call(
            canister_id,
            user_principal, // caller
            "get_evm_address",
            Encode!().unwrap() // пустые аргументы
        );
        
        match result {
            Ok(bytes) => {
                let address: Option<String> = Decode!(&bytes, Option<String>).expect("Failed to decode result");
                // Should be None initially
                assert_eq!(address, None, "Initial address should be None");
                println!("Initial EVM address correctly returned as None");
            },
            Err(e) => {
                panic!("Query rejected: {:?}", e);
            }
        }
    }

    #[test]
    fn test_generate_and_get_evm_address() {
        // Initialize PocketIC
        let pic = PocketIc::new();
        
        // Create a new canister
        let canister_id = pic.create_canister();
        pic.add_cycles(canister_id, 2_000_000_000_000);
        
        // Get and check the WASM file path
        let wasm_path = get_wasm_path();
        if !wasm_path.exists() {
            panic!("WASM file not found at path: {}. Make sure to run 'dfx build' first.", 
                   wasm_path.display());
        }
        
        // Load WASM file
        let wasm = fs::read(&wasm_path).expect("Could not read WASM file");
        
        // Install the canister
        pic.install_canister(canister_id, wasm, vec![], None);
        
        // Convert string principal to Principal type
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        
        // Call generate_evm_address function
        let args = Encode!().unwrap(); // Empty arguments
        let result = pic.update_call(
            canister_id,
            user_principal,  // caller
            "generate_evm_address",
            args
        );
        
        match result {
            Ok(bytes) => {
                let address: Result<String, String> = Decode!(&bytes, Result<String, String>).expect("Failed to decode result");
                match address {
                    Ok(address) => {
                        println!("EVM address: {}", address);
                        
                        assert!(address.starts_with("0x"), "Address should start with 0x");
                        assert_eq!(address.len(), 42, "Address should be 42 characters (0x + 40 hex)");
                        
                        // Now check the stored address
                        let stored_result = pic.query_call(
                            canister_id,
                            user_principal,  // caller
                            "get_evm_address",
                            Encode!().unwrap() // пустые аргументы
                        );
                        
                        match stored_result {
                            Ok(bytes) => {
                                let stored_address: Option<String> = Decode!(&bytes, Option<String>).expect("Failed to decode stored address");
                                assert_eq!(stored_address, Some(address), "Stored address doesn't match generated address");
                            },
                            Err(e) => {
                                panic!("Get EVM address query rejected: {:?}", e);
                            }
                        }
                    },
                    Err(e) => {
                        panic!("Function returned error: {}", e);
                    }
                }
            },
            Err(e) => {
                panic!("Update rejected: {:?}", e);
            }
        }
    }

    #[test]
    fn test_verify_user() {
        // Initialize PocketIC
        let pic = PocketIc::new();
        
        // Create a new canister
        let canister_id = pic.create_canister();
        pic.add_cycles(canister_id, 2_000_000_000_000);
        
        // Get and check the WASM file path
        let wasm_path = get_wasm_path();
        if !wasm_path.exists() {
            panic!("WASM file not found at path: {}. Make sure to run 'dfx build' first.", 
                   wasm_path.display());
        }
        
        // Load WASM file
        let wasm = fs::read(&wasm_path).expect("Could not read WASM file");
        
        // Install the canister
        pic.install_canister(canister_id, wasm, vec![], None);
        
        // Convert string principal to Principal type
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        
        // Check user verification before generating address
        let args = Encode!(&user_principal).unwrap();
        let result = pic.query_call(
            canister_id,
            user_principal,  // caller
            "verify_user",
            args.clone()
        );
        
        match result {
            Ok(bytes) => {
                let verified: bool = Decode!(&bytes, bool).expect("Failed to decode result");
                assert_eq!(verified, false, "User should not be verified before generating an address");
                
                // Now generate an address
                let gen_result = pic.update_call(
                    canister_id,
                    user_principal,
                    "generate_evm_address",
                    Encode!().unwrap()
                );
                
                if gen_result.is_ok() {
                    // Check verification again after generating address
                    let verify_result = pic.query_call(
                        canister_id,
                        user_principal,
                        "verify_user",
                        args
                    );
                    
                    match verify_result {
                        Ok(verify_bytes) => {
                            let verified_after: bool = Decode!(&verify_bytes, bool).expect("Failed to decode verification result");
                            assert_eq!(verified_after, true, "User should be verified after generating an address");
                        },
                        Err(e) => {
                            panic!("Verification query rejected: {:?}", e);
                        }
                    }
                } else {
                    panic!("Failed to generate address: {:?}", gen_result.err());
                }
            },
            Err(e) => {
                panic!("Verification query rejected: {:?}", e);
            }
        }
    }
}
