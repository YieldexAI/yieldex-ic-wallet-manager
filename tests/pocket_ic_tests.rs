use pocket_ic::PocketIcBuilder;
use std::fs;
use std::path::PathBuf;
use candid::{Principal, Decode, Encode, CandidType};
use serde::{Deserialize, Serialize};

// Define the principal ID for tests
const USER_PRINCIPAL: &str = "hfugy-ahqdz-5sbki-vky4l-xceci-3se5z-2cb7k-jxjuq-qidax-gd53f-nqe";
// Second principal for testing access control
const ANOTHER_PRINCIPAL: &str = "4qflw-v6eu4-gy2he-esqdb-xaihv-bne5s-vublq-6xzj7-ffcpk-vzroe-nqe";
// Base path to the WASM file (relative to the project root)
const WASM_PATH_RELATIVE: &str = ".dfx/local/canisters/yieldex-ic-wallet-manager-backend/yieldex-ic-wallet-manager-backend.wasm";

// Define our own versions of the structs for tests, which have the same fields
// Use these structs only for creating requests
#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq)]
struct TransferLimit {
    pub token_address: String,
    pub daily_limit: u64,
    pub max_tx_amount: u64,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct CreatePermissionsRequest {
    pub whitelisted_protocols: Vec<Protocol>,
    pub whitelisted_tokens: Vec<Token>,
    pub transfer_limits: Vec<TransferLimit>,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct UpdatePermissionsRequest {
    pub permissions_id: String,
    pub whitelisted_protocols: Option<Vec<Protocol>>,
    pub whitelisted_tokens: Option<Vec<Token>>,
    pub transfer_limits: Option<Vec<TransferLimit>>,
}

// Permissions struct for testing
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct Permissions {
    pub id: String,
    pub owner: Principal,
    pub whitelisted_protocols: Vec<Protocol>,
    pub whitelisted_tokens: Vec<Token>,
    pub transfer_limits: Vec<TransferLimit>,
    pub created_at: u64,
    pub updated_at: u64,
}

// Example Protocol and Token for tests
fn example_protocol() -> Protocol {
    Protocol {
        name: "AAVE".to_string(),
        address: "0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DdAE9".to_string(),
    }
}
fn example_token() -> Token {
    Token {
        name: "USDT".to_string(),
        address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
    }
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq)]
pub struct Protocol {
    pub name: String,
    pub address: String,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq)]
pub struct Token {
    pub name: String,
    pub address: String,
}

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

    // Helper function to setup a test environment with a canister
    fn setup_test_env() -> (pocket_ic::PocketIc, Principal) {
        // Initialize PocketIC with II subnet for ECDSA support
        let pic = PocketIcBuilder::new()
            .with_nns_subnet()
            .with_ii_subnet() // this subnet has threshold ECDSA keys
            .with_application_subnet()
            .build();
        
        // Create a canister on the app subnet
        let topology = pic.topology();
        let app_subnet = topology.get_app_subnets()[0];
        
        // Create a new canister
        let canister_id = pic.create_canister_on_subnet(None, None, app_subnet);
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
        
        (pic, canister_id)
    }

    #[test]
    fn test_get_evm_address() {
        // Initialize PocketIC with II subnet for ECDSA support
        let pic = PocketIcBuilder::new()
            .with_nns_subnet()
            .with_ii_subnet() // this subnet has threshold ECDSA keys
            .with_application_subnet()
            .build();
        
        // Create a canister on the app subnet
        let topology = pic.topology();
        let app_subnet = topology.get_app_subnets()[0];
        
        // Create a new canister
        let canister_id = pic.create_canister_on_subnet(None, None, app_subnet);
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
        
        // Arguments must be empty for get_evm_address
        let result = pic.query_call(
            canister_id,
            user_principal, // caller
            "get_evm_address",
            Encode!().unwrap() // empty arguments
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
        // Initialize PocketIC with II subnet for ECDSA support
        let pic = PocketIcBuilder::new()
            .with_nns_subnet()
            .with_ii_subnet() // this subnet has threshold ECDSA keys
            .with_application_subnet()
            .build();
        
        // Create a canister on the app subnet
        let topology = pic.topology();
        let app_subnet = topology.get_app_subnets()[0];
        
        // Create a new canister
        let canister_id = pic.create_canister_on_subnet(None, None, app_subnet);
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
                            Encode!().unwrap() // empty arguments
                        );
                        
                        match stored_result {
                            Ok(bytes) => {
                                let stored_address: Result<String, String> = Decode!(&bytes, Result<String, String>).expect("Failed to decode stored address");
                                assert_eq!(stored_address, Ok(address), "Stored address doesn't match generated address");
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
        // Initialize PocketIC with II subnet for ECDSA support
        let pic = PocketIcBuilder::new()
            .with_nns_subnet()
            .with_ii_subnet() // this subnet has threshold ECDSA keys
            .with_application_subnet()
            .build();
        
        // Create a canister on the app subnet
        let topology = pic.topology();
        let app_subnet = topology.get_app_subnets()[0];
        
        // Create a new canister
        let canister_id = pic.create_canister_on_subnet(None, None, app_subnet);
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
                let verified: Result<bool, String> = Decode!(&bytes, Result<bool, String>).expect("Failed to decode result");
                assert_eq!(verified, Err("User does not have an EVM address".to_string()), "User should not be verified before generating an address");
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
                            let verified_after: Result<bool, String> = Decode!(&verify_bytes, Result<bool, String>).expect("Failed to decode verification result");
                            assert_eq!(verified_after, Ok(true), "User should be verified after generating an address");
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

    // New tests for permissions management functions

    #[test]
    fn test_create_and_get_permissions() {
        // Initialize test environment
        let (pic, canister_id) = setup_test_env();
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        
        // First, generate EVM address (required for creating permissions)
        let gen_result = pic.update_call(
            canister_id,
            user_principal,
            "generate_evm_address",
            Encode!().unwrap()
        );
        assert!(gen_result.is_ok(), "Failed to generate EVM address");
        
        // Create a request for creating permissions with new Protocol and Token types
        let request = CreatePermissionsRequest {
            whitelisted_protocols: vec![example_protocol()],
            whitelisted_tokens: vec![example_token()],
            transfer_limits: vec![
                TransferLimit {
                    token_address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
                    daily_limit: 1000,
                    max_tx_amount: 100
                }
            ]
        };
        
        // Call create_permissions method
        let create_result = pic.update_call(
            canister_id,
            user_principal,
            "create_permissions",
            Encode!(&request).unwrap()
        );
        
        // Check the result
        match create_result {
            Ok(bytes) => {
                let permissions_result: Result<Permissions, String> = 
                    Decode!(&bytes, Result<Permissions, String>).expect("Failed to decode result");
                
                match permissions_result {
                    Ok(permissions) => {
                        println!("Created permissions with id: {}", permissions.id);
                        
                        // Check permissions data
                        assert_eq!(permissions.owner, user_principal);
                        assert_eq!(permissions.whitelisted_protocols.len(), 1);
                        assert_eq!(permissions.whitelisted_protocols[0].name, "AAVE");
                        assert_eq!(permissions.whitelisted_tokens.len(), 1);
                        assert_eq!(permissions.whitelisted_tokens[0].name, "USDT");
                        assert_eq!(permissions.transfer_limits.len(), 1);
                        
                        // Get permissions by ID
                        let get_result = pic.query_call(
                            canister_id,
                            user_principal,
                            "get_permissions",
                            Encode!(&permissions.id).unwrap()
                        );
                        
                        match get_result {
                            Ok(bytes) => {
                                let get_permissions_result: Result<Permissions, String> = 
                                    Decode!(&bytes, Result<Permissions, String>).expect("Failed to decode result");
                                
                                let get_permissions = get_permissions_result.expect("Failed to get permissions");
                                assert_eq!(get_permissions.id, permissions.id);
                                assert_eq!(get_permissions.whitelisted_protocols, permissions.whitelisted_protocols);
                                assert_eq!(get_permissions.whitelisted_tokens, permissions.whitelisted_tokens);
                            },
                            Err(e) => {
                                panic!("Failed to get permissions: {:?}", e);
                            }
                        }
                        
                        // Get all permissions for the user
                        let get_all_result = pic.query_call(
                            canister_id,
                            user_principal,
                            "get_all_permissions",
                            Encode!().unwrap()
                        );
                        
                        match get_all_result {
                            Ok(bytes) => {
                                let all_permissions: Result<Vec<Permissions>, String> = 
                                    Decode!(&bytes, Result<Vec<Permissions>, String>).expect("Failed to decode result");
                                
                                let permissions_list = all_permissions.expect("Failed to get all permissions");
                                assert_eq!(permissions_list.len(), 1);
                                assert_eq!(permissions_list[0].id, permissions.id);
                            },
                            Err(e) => {
                                panic!("Failed to get all permissions: {:?}", e);
                            }
                        }
                    },
                    Err(e) => {
                        panic!("Failed to create permissions: {}", e);
                    }
                }
            },
            Err(e) => {
                panic!("Failed to create permissions: {:?}", e);
            }
        }
    }
    
    #[test]
    fn test_update_and_delete_permissions() {
        // Initialize test environment
        let (pic, canister_id) = setup_test_env();
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        
        // First, generate EVM address
        let gen_result = pic.update_call(
            canister_id,
            user_principal,
            "generate_evm_address",
            Encode!().unwrap()
        );
        assert!(gen_result.is_ok(), "Failed to generate EVM address");
        
        // Create permissions
        let request = CreatePermissionsRequest {
            whitelisted_protocols: vec![example_protocol()],
            whitelisted_tokens: vec![example_token()],
            transfer_limits: vec![
                TransferLimit {
                    token_address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
                    daily_limit: 1000,
                    max_tx_amount: 100
                }
            ]
        };
        
        let create_result = pic.update_call(
            canister_id,
            user_principal,
            "create_permissions",
            Encode!(&request).unwrap()
        );
        
        let permissions = match create_result {
            Ok(bytes) => {
                let permissions_result: Result<Permissions, String> = 
                    Decode!(&bytes, Result<Permissions, String>).expect("Failed to decode result");
                
                permissions_result.expect("Failed to create permissions")
            },
            Err(e) => {
                panic!("Failed to create permissions: {:?}", e);
            }
        };
        
        let permissions_id = permissions.id.clone();
        
        // Update permissions
        let update_request = UpdatePermissionsRequest {
            permissions_id: permissions_id.clone(),
            whitelisted_protocols: Some(vec![example_protocol()]),
            whitelisted_tokens: Some(vec![example_token()]),
            transfer_limits: None
        };
        
        let update_result = pic.update_call(
            canister_id,
            user_principal,
            "update_permissions",
            Encode!(&update_request).unwrap()
        );
        
        // Check update result
        match update_result {
            Ok(bytes) => {
                let updated_permissions_result: Result<Permissions, String> = 
                    Decode!(&bytes, Result<Permissions, String>).expect("Failed to decode result");
                
                let updated_permissions = updated_permissions_result.expect("Failed to update permissions");
                
                // Check that the data has been updated
                assert_eq!(updated_permissions.id, permissions_id);
                assert_eq!(updated_permissions.whitelisted_protocols, vec![example_protocol()]);
                assert_eq!(updated_permissions.whitelisted_tokens, vec![example_token()]);
                assert_eq!(updated_permissions.transfer_limits, permissions.transfer_limits);
                assert!(updated_permissions.updated_at >= permissions.created_at);
                
                // Delete permissions
                let delete_result = pic.update_call(
                    canister_id,
                    user_principal,
                    "delete_permissions",
                    Encode!(&permissions_id).unwrap()
                );
                
                match delete_result {
                    Ok(bytes) => {
                        let delete_success: Result<bool, String> = 
                            Decode!(&bytes, Result<bool, String>).expect("Failed to decode result");
                        
                        assert_eq!(delete_success, Ok(true), "Deletion should return true");
                        
                        // Check that the permissions are actually deleted
                        let get_result = pic.query_call(
                            canister_id,
                            user_principal,
                            "get_permissions",
                            Encode!(&permissions_id).unwrap()
                        );
                        
                        match get_result {
                            Ok(bytes) => {
                                let get_permissions_result: Result<Permissions, String> = 
                                    Decode!(&bytes, Result<Permissions, String>).expect("Failed to decode result");
                                
                                assert!(get_permissions_result.is_err(), "Permissions should be deleted");
                            },
                            Err(e) => {
                                panic!("Unexpected error: {:?}", e);
                            }
                        }
                    },
                    Err(e) => {
                        panic!("Failed to delete permissions: {:?}", e);
                    }
                }
            },
            Err(e) => {
                panic!("Failed to update permissions: {:?}", e);
            }
        }
    }
    
    #[test]
    fn test_permissions_error_cases() {
        // Initialize test environment
        let (pic, canister_id) = setup_test_env();
        
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        let another_principal = Principal::from_text(ANOTHER_PRINCIPAL).expect("Invalid principal");
        
        // Test 1: Attempt to create permissions without EVM address
        let request = CreatePermissionsRequest {
            whitelisted_protocols: vec![example_protocol()],
            whitelisted_tokens: vec![example_token()],
            transfer_limits: vec![]
        };
        
        let create_result = pic.update_call(
            canister_id,
            user_principal,
            "create_permissions",
            Encode!(&request).unwrap()
        );
        
        match create_result {
            Ok(bytes) => {
                let result: Result<Permissions, String> = 
                    Decode!(&bytes, Result<Permissions, String>).expect("Failed to decode result");
                assert!(result.is_err(), "Should fail without EVM address");
                let error = result.unwrap_err();
                assert_eq!(error, "User does not have an EVM address");
            },
            Err(e) => {
                panic!("Unexpected rejection: {:?}", e);
            }
        }
        
        // Generate EVM address for user_principal
        let gen_result = pic.update_call(
            canister_id,
            user_principal,
            "generate_evm_address",
            Encode!().unwrap()
        );
        assert!(gen_result.is_ok(), "Failed to generate EVM address");
        
        // Create permissions
        let create_result = pic.update_call(
            canister_id,
            user_principal,
            "create_permissions",
            Encode!(&request).unwrap()
        );
        
        let permissions = match create_result {
            Ok(bytes) => {
                let permissions_result: Result<Permissions, String> = 
                    Decode!(&bytes, Result<Permissions, String>).expect("Failed to decode result");
                
                permissions_result.expect("Failed to create permissions")
            },
            Err(e) => {
                panic!("Failed to create permissions: {:?}", e);
            }
        };
        
        let permissions_id = permissions.id.clone();
        
        // Test 2: Attempt to access someone else's permissions
        let get_result = pic.query_call(
            canister_id,
            another_principal,  // Another user
            "get_permissions",
            Encode!(&permissions_id).unwrap()
        );
        
        match get_result {
            Ok(bytes) => {
                let result: Result<Permissions, String> = 
                    Decode!(&bytes, Result<Permissions, String>).expect("Failed to decode result");
                
                assert!(result.is_err(), "Should fail for another principal");
                let error = result.unwrap_err();
                assert!(error.contains("not found") || error.contains("permission"), 
                        "Expected permission error, got: {}", error);
            },
            Err(e) => {
                panic!("Unexpected rejection: {:?}", e);
            }
        }
        
        // Test 3: Attempt to update someone else's permissions
        let update_request = UpdatePermissionsRequest {
            permissions_id: permissions_id.clone(),
            whitelisted_protocols: Some(vec![example_protocol()]),
            whitelisted_tokens: None,
            transfer_limits: None
        };
        
        let update_result = pic.update_call(
            canister_id,
            another_principal,  // Another user
            "update_permissions",
            Encode!(&update_request).unwrap()
        );
        
        match update_result {
            Ok(bytes) => {
                let result: Result<Permissions, String> = 
                    Decode!(&bytes, Result<Permissions, String>).expect("Failed to decode result");
                
                assert!(result.is_err(), "Should fail for another principal");
                let error = result.unwrap_err();
                assert!(error.contains("not found") || error.contains("permission"), 
                        "Expected permission error, got: {}", error);
            },
            Err(e) => {
                panic!("Unexpected rejection: {:?}", e);
            }
        }
        
        // Test 4: Attempt to delete someone else's permissions
        let delete_result = pic.update_call(
            canister_id,
            another_principal,  // Another user
            "delete_permissions",
            Encode!(&permissions_id).unwrap()
        );
        
        match delete_result {
            Ok(bytes) => {
                let result: Result<bool, String> = 
                    Decode!(&bytes, Result<bool, String>).expect("Failed to decode result");
                
                assert!(result.is_err(), "Should fail for another principal");
                let error = result.unwrap_err();
                assert!(error.contains("not found") || error.contains("permission"), 
                        "Expected permission error, got: {}", error);
            },
            Err(e) => {
                panic!("Unexpected rejection: {:?}", e);
            }
        }
        
        // Test 5: Attempt to update non-existent permissions
        let nonexistent_id = "nonexistent_id".to_string();
        let update_request = UpdatePermissionsRequest {
            permissions_id: nonexistent_id.clone(),
            whitelisted_protocols: Some(vec![example_protocol()]),
            whitelisted_tokens: None,
            transfer_limits: None
        };
        
        let update_result = pic.update_call(
            canister_id,
            user_principal,
            "update_permissions",
            Encode!(&update_request).unwrap()
        );
        
        match update_result {
            Ok(bytes) => {
                let result: Result<Permissions, String> = 
                    Decode!(&bytes, Result<Permissions, String>).expect("Failed to decode result");
                
                assert!(result.is_err(), "Should fail for nonexistent ID");
                let error = result.unwrap_err();
                assert!(error.contains("not found"), 
                        "Expected 'not found' error, got: {}", error);
            },
            Err(e) => {
                panic!("Unexpected rejection: {:?}", e);
            }
        }
    }
}
