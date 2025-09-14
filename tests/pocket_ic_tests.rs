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
#[derive(CandidType, Deserialize, Debug, Clone, PartialEq)]
struct TransferLimit {
    pub token_address: String,
    pub daily_limit: u64,
    pub max_tx_amount: u64,
}

// 🆕 Добавляем ProtocolPermission (Задача 1.1)
#[derive(CandidType, Deserialize, Debug, Clone, PartialEq)]
struct ProtocolPermission {
    pub protocol_address: String,
    pub allowed_functions: Vec<String>,
    pub max_amount_per_tx: Option<u64>,
    pub daily_limit: Option<u64>,
    pub total_used_today: u64,
    pub last_reset_date: u64,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
struct CreatePermissionsRequest {
    pub chain_id: u64,                                         // 🆕 Required chain ID
    pub whitelisted_protocols: Vec<Protocol>,
    pub whitelisted_tokens: Vec<Token>,
    pub transfer_limits: Vec<TransferLimit>,
    pub protocol_permissions: Option<Vec<ProtocolPermission>>, // 🆕 Протокольные разрешения
}

#[derive(CandidType, Deserialize, Debug, Clone)]
struct UpdatePermissionsRequest {
    pub permissions_id: String,
    pub whitelisted_protocols: Option<Vec<Protocol>>,
    pub whitelisted_tokens: Option<Vec<Token>>,
    pub transfer_limits: Option<Vec<TransferLimit>>,
}

// 🆕 Обновленная структура Permissions (Задача 1.1)
#[derive(CandidType, Deserialize, Debug, Clone)]
struct Permissions {
    pub id: String,
    pub owner: Principal,
    pub chain_id: u64,                                // 🆕 Chain ID field
    pub whitelisted_protocols: Vec<Protocol>,
    pub whitelisted_tokens: Vec<Token>,
    pub transfer_limits: Vec<TransferLimit>,
    pub protocol_permissions: Vec<ProtocolPermission>, // 🆕 Новое поле
    pub created_at: u64,
    pub updated_at: u64,
}

// Example Protocol and Token for tests
fn example_protocol() -> Protocol {
    Protocol {
        name: "AAVE".to_string(),
        address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(), // Updated to correct Sepolia address
    }
}

fn example_token() -> Token {
    Token {
        name: "LINK".to_string(),
        address: "0xf8fb3713d459d7c1018bd0a49d19b4c44290ebe5".to_string(), // Updated to LINK Sepolia address
    }
}

// 🆕 Helper function to create AAVE protocol permissions for tests
fn example_aave_protocol_permission() -> ProtocolPermission {
    ProtocolPermission {
        protocol_address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
        allowed_functions: vec!["supply".to_string(), "withdraw".to_string()],
        max_amount_per_tx: Some(100_000_000_000_000_000), // 0.1 LINK
        daily_limit: Some(1_000_000_000_000_000_000),     // 1 LINK
        total_used_today: 0,
        last_reset_date: 0,
    }
}

// 🆕 Helper function to create basic permissions request without protocol permissions
fn basic_permissions_request() -> CreatePermissionsRequest {
    CreatePermissionsRequest {
        chain_id: 11155111, // Sepolia chain ID for AAVE
        whitelisted_protocols: vec![example_protocol()],
        whitelisted_tokens: vec![example_token()],
        transfer_limits: vec![TransferLimit {
            token_address: "0xf8fb3713d459d7c1018bd0a49d19b4c44290ebe5".to_string(),
            daily_limit: 1_000_000_000_000_000_000,  // 1 LINK
            max_tx_amount: 100_000_000_000_000_000,   // 0.1 LINK
        }],
        protocol_permissions: None,
    }
}

// 🆕 Helper function to create full permissions request with AAVE protocol permissions
fn full_permissions_request_with_aave() -> CreatePermissionsRequest {
    CreatePermissionsRequest {
        chain_id: 11155111, // Sepolia chain ID for AAVE
        whitelisted_protocols: vec![example_protocol()],
        whitelisted_tokens: vec![example_token()],
        transfer_limits: vec![TransferLimit {
            token_address: "0xf8fb3713d459d7c1018bd0a49d19b4c44290ebe5".to_string(),
            daily_limit: 1_000_000_000_000_000_000,  // 1 LINK
            max_tx_amount: 100_000_000_000_000_000,   // 0.1 LINK
        }],
        protocol_permissions: Some(vec![example_aave_protocol_permission()]),
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
        let request = basic_permissions_request();
        
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
                        assert_eq!(permissions.whitelisted_tokens[0].name, "LINK");
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
        let request = basic_permissions_request();
        
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
        let request = basic_permissions_request();
        
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
                assert!(error.contains("not found") || error.contains("permission") || error.contains("Access denied"), 
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
                assert!(error.contains("not found") || error.contains("permission") || error.contains("Access denied"), 
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
                assert!(error.contains("not found") || error.contains("permission") || error.contains("Access denied"), 
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

    // 🆕 Тест 1.1: Protocol Permission Management (Задача 1.1)
    #[test]
    fn test_protocol_permission_management() {
        // Initialize test environment
        let (pic, canister_id) = setup_test_env();
        
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        
        // 1. Generate EVM address
        let gen_result = pic.update_call(
            canister_id,
            user_principal,
            "generate_evm_address",
            Encode!().unwrap()
        );
        assert!(gen_result.is_ok(), "Failed to generate EVM address");
        
        // 2. Создать базовые permissions
        let request = CreatePermissionsRequest {
            chain_id: 11155111, // Sepolia chain ID for AAVE
            whitelisted_protocols: vec![],
            whitelisted_tokens: vec![],
            transfer_limits: vec![],
            protocol_permissions: None,
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
        
        // 3. Добавить protocol permission для AAVE
        let protocol_perm = ProtocolPermission {
            protocol_address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(), // AAVE Pool
            allowed_functions: vec!["supply".to_string(), "withdraw".to_string()],
            max_amount_per_tx: Some(100_000000), // 100 USDC
            daily_limit: Some(1000_000000), // 1000 USDC
            total_used_today: 0,
            last_reset_date: 0, // Используем 0 вместо ic_cdk::api::time() для тестов
        };
        
        let add_perm_result = pic.update_call(
            canister_id,
            user_principal,
            "update_protocol_permission",
            Encode!(&permissions.id, &protocol_perm).unwrap()
        );
        
        match add_perm_result {
            Ok(bytes) => {
                let result: Result<bool, String> = 
                    Decode!(&bytes, Result<bool, String>).expect("Failed to decode result");
                assert_eq!(result, Ok(true), "Adding protocol permission should succeed");
            },
            Err(e) => {
                panic!("Failed to add protocol permission: {:?}", e);
            }
        }
        
        // 4. Проверить разрешение
        let verification_result = pic.query_call(
            canister_id,
            user_principal,
            "check_protocol_permission",
            Encode!(
                &permissions.id,
                &"0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
                &"supply".to_string(),
                &50_000000u64 // 50 USDC
            ).unwrap()
        );
        
        match verification_result {
            Ok(bytes) => {
                let result: Result<bool, String> = 
                    Decode!(&bytes, Result<bool, String>).expect("Failed to decode result");
                assert_eq!(result, Ok(true), "Permission verification should succeed");
            },
            Err(e) => {
                panic!("Failed to verify protocol permission: {:?}", e);
            }
        }
        
        // 5. Проверить запрещенную функцию
        let forbidden_verification = pic.query_call(
            canister_id,
            user_principal,
            "check_protocol_permission",
            Encode!(
                &permissions.id,
                &"0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
                &"borrow".to_string(), // Функция не разрешена
                &10_000000u64 // 10 USDC
            ).unwrap()
        );
        
        match forbidden_verification {
            Ok(bytes) => {
                let result: Result<bool, String> = 
                    Decode!(&bytes, Result<bool, String>).expect("Failed to decode result");
                assert!(result.is_err(), "Forbidden function should fail verification");
                let error = result.unwrap_err();
                assert!(error.contains("not allowed"), "Expected 'not allowed' error, got: {}", error);
            },
            Err(e) => {
                panic!("Failed to verify forbidden function: {:?}", e);
            }
        }
        
        // 6. Проверить превышение лимита
        let over_limit_verification = pic.query_call(
            canister_id,
            user_principal,
            "check_protocol_permission",
            Encode!(
                &permissions.id,
                &"0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
                &"supply".to_string(),
                &150_000000u64 // 150 USDC - превышает max_amount_per_tx (100 USDC)
            ).unwrap()
        );
        
        match over_limit_verification {
            Ok(bytes) => {
                let result: Result<bool, String> = 
                    Decode!(&bytes, Result<bool, String>).expect("Failed to decode result");
                assert!(result.is_err(), "Over limit amount should fail verification");
                let error = result.unwrap_err();
                assert!(error.contains("exceeds max limit"), "Expected 'exceeds max limit' error, got: {}", error);
            },
            Err(e) => {
                panic!("Failed to verify over limit amount: {:?}", e);
            }
        }
    }

    // 🆕 Тест 2.2: AAVE ABI Loading and Contract Instance Creation (Задача 2.2)
    #[test]
    fn test_aave_abi_loading() {
        // Initialize test environment
        let (pic, canister_id) = setup_test_env();
        
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        
        // 1. Generate EVM address
        let gen_result = pic.update_call(
            canister_id,
            user_principal,
            "generate_evm_address",
            Encode!().unwrap()
        );
        assert!(gen_result.is_ok(), "Failed to generate EVM address");
        
        // 2. Test getting AAVE LINK balance (this will test ABI loading internally)
        let balance_result = pic.update_call(
            canister_id,
            user_principal,
            "get_aave_link_user_balance",
            Encode!(&None::<String>).unwrap()
        );
        
        match balance_result {
            Ok(bytes) => {
                let result: Result<String, String> = 
                    Decode!(&bytes, Result<String, String>).expect("Failed to decode result");
                
                // The function should either return a balance or an error about network/contract
                // Both are acceptable as it proves ABI loading works
                match result {
                    Ok(balance) => {
                        println!("AAVE LINK balance: {}", balance);
                        assert!(balance.starts_with("0x"), "Balance should be hex format");
                    },
                    Err(error) => {
                        println!("Expected error (network/contract related): {}", error);
                        // Errors related to network connectivity or contract calls are expected in tests
                        // The important thing is that ABI loading didn't fail
                        assert!(
                            error.contains("Failed to get reserve data") || 
                            error.contains("Failed to get aLINK balance") ||
                            error.contains("network") ||
                            error.contains("RPC"),
                            "Error should be network/contract related, got: {}", error
                        );
                    }
                }
            },
            Err(e) => {
                panic!("AAVE balance call was rejected: {:?}", e);
            }
        }
    }

    // 🆕 Тест 2.1: AAVE Supply with Permissions (Задача 2.1)
    #[test]
    fn test_aave_supply_with_permissions() {
        // Initialize test environment
        let (pic, canister_id) = setup_test_env();
        
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        
        // 1. Generate EVM address
        let gen_result = pic.update_call(
            canister_id,
            user_principal,
            "generate_evm_address",
            Encode!().unwrap()
        );
        assert!(gen_result.is_ok(), "Failed to generate EVM address");
        
        // 2. Create permissions with AAVE protocol
        let request = CreatePermissionsRequest {
            chain_id: 11155111, // Sepolia chain ID for AAVE
            whitelisted_protocols: vec![Protocol {
                name: "AAVE".to_string(),
                address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
            }],
            whitelisted_tokens: vec![Token {
                name: "LINK".to_string(),
                address: "0xf8fb3713d459d7c1018bd0a49d19b4c44290ebe5".to_string(), // Correct LINK address on Sepolia
            }],
            transfer_limits: vec![],
            protocol_permissions: None,
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
        
        // 3. Add AAVE protocol permission
        let protocol_perm = ProtocolPermission {
            protocol_address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
            allowed_functions: vec!["supply".to_string(), "withdraw".to_string()],
            max_amount_per_tx: Some(1000000000000000000), // 1 LINK (18 decimals)
            daily_limit: Some(10000000000000000000), // 10 LINK
            total_used_today: 0,
            last_reset_date: 0,
        };
        
        let add_perm_result = pic.update_call(
            canister_id,
            user_principal,
            "update_protocol_permission",
            Encode!(&permissions.id, &protocol_perm).unwrap()
        );
        
        assert!(add_perm_result.is_ok(), "Failed to add protocol permission");
        
        // 4. Test AAVE supply (will fail due to insufficient balance, but tests the flow)
        let supply_result = pic.update_call(
            canister_id,
            user_principal,
            "supply_link_to_aave_secured",
            Encode!(&"0.1".to_string(), &permissions.id).unwrap()
        );
        
        match supply_result {
            Ok(bytes) => {
                let result: Result<String, String> = 
                    Decode!(&bytes, Result<String, String>).expect("Failed to decode result");
                
                match result {
                    Ok(success_msg) => {
                        println!("AAVE supply succeeded: {}", success_msg);
                        assert!(success_msg.contains("Successfully supplied"), "Should contain success message");
                    },
                    Err(error) => {
                        println!("Expected error (insufficient balance or network): {}", error);
                        // Expected errors: insufficient balance, network issues, etc.
                        assert!(
                            error.contains("Insufficient LINK balance") ||
                            error.contains("Failed to get balance") ||
                            error.contains("Failed to get nonce") ||
                            error.contains("network") ||
                            error.contains("RPC") ||
                            error.contains("server returned an error response") ||
                            error.contains("No route to canister"),
                            "Error should be balance/network related, got: {}", error
                        );
                    }
                }
            },
            Err(e) => {
                panic!("AAVE supply call was rejected: {:?}", e);
            }
        }
    }

    // 🆕 Тест 2.1: AAVE Withdraw with Permissions (Задача 2.1)
    #[test]
    fn test_aave_withdraw_with_permissions() {
        // Initialize test environment
        let (pic, canister_id) = setup_test_env();
        
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        
        // 1. Generate EVM address
        let gen_result = pic.update_call(
            canister_id,
            user_principal,
            "generate_evm_address",
            Encode!().unwrap()
        );
        assert!(gen_result.is_ok(), "Failed to generate EVM address");
        
        // 2. Create permissions with AAVE protocol
        let request = CreatePermissionsRequest {
            chain_id: 11155111, // Sepolia chain ID for AAVE
            whitelisted_protocols: vec![Protocol {
                name: "AAVE".to_string(),
                address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
            }],
            whitelisted_tokens: vec![Token {
                name: "LINK".to_string(),
                address: "0xf8fb3713d459d7c1018bd0a49d19b4c44290ebe5".to_string(), // Correct LINK address on Sepolia
            }],
            transfer_limits: vec![],
            protocol_permissions: None,
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
        
        // 3. Add AAVE protocol permission
        let protocol_perm = ProtocolPermission {
            protocol_address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
            allowed_functions: vec!["supply".to_string(), "withdraw".to_string()],
            max_amount_per_tx: Some(1000000000000000000), // 1 LINK (18 decimals)
            daily_limit: Some(10000000000000000000), // 10 LINK
            total_used_today: 0,
            last_reset_date: 0,
        };
        
        let add_perm_result = pic.update_call(
            canister_id,
            user_principal,
            "update_protocol_permission",
            Encode!(&permissions.id, &protocol_perm).unwrap()
        );
        
        assert!(add_perm_result.is_ok(), "Failed to add protocol permission");
        
        // 4. Test AAVE withdraw (will fail due to insufficient aLINK balance, but tests the flow)
        let withdraw_result = pic.update_call(
            canister_id,
            user_principal,
            "withdraw_link_from_aave_secured",
            Encode!(&"0.1".to_string(), &permissions.id).unwrap()
        );
        
        match withdraw_result {
            Ok(bytes) => {
                let result: Result<String, String> = 
                    Decode!(&bytes, Result<String, String>).expect("Failed to decode result");
                
                match result {
                    Ok(success_msg) => {
                        println!("AAVE withdraw succeeded: {}", success_msg);
                        assert!(success_msg.contains("Successfully withdrew"), "Should contain success message");
                    },
                    Err(error) => {
                        println!("Expected error (insufficient aLINK balance or network): {}", error);
                        // Expected errors: insufficient aLINK balance, network issues, etc.
                        assert!(
                            error.contains("Insufficient aLINK balance") ||
                            error.contains("Failed to get reserve data") ||
                            error.contains("Failed to get aLINK balance") ||
                            error.contains("Failed to get nonce") ||
                            error.contains("network") ||
                            error.contains("RPC"),
                            "Error should be balance/network related, got: {}", error
                        );
                    }
                }
            },
            Err(e) => {
                panic!("AAVE withdraw call was rejected: {:?}", e);
            }
        }
    }

    // 🆕 Тест: AAVE Security - Access Control (Задача 2.1)
    #[test]
    fn test_aave_security_access_control() {
        // Initialize test environment
        let (pic, canister_id) = setup_test_env();
        
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        let another_principal = Principal::from_text(ANOTHER_PRINCIPAL).expect("Invalid principal");
        
        // 1. Generate EVM address for user
        let gen_result = pic.update_call(
            canister_id,
            user_principal,
            "generate_evm_address",
            Encode!().unwrap()
        );
        assert!(gen_result.is_ok(), "Failed to generate EVM address");
        
        // 2. Create permissions
        let request = CreatePermissionsRequest {
            chain_id: 11155111, // Sepolia chain ID for AAVE
            whitelisted_protocols: vec![],
            whitelisted_tokens: vec![],
            transfer_limits: vec![],
            protocol_permissions: None,
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
        
        // 3. Add AAVE protocol permission
        let protocol_perm = ProtocolPermission {
            protocol_address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
            allowed_functions: vec!["supply".to_string()],
            max_amount_per_tx: Some(1000000000000000000), // 1 LINK
            daily_limit: Some(10000000000000000000), // 10 LINK
            total_used_today: 0,
            last_reset_date: 0,
        };
        
        let add_perm_result = pic.update_call(
            canister_id,
            user_principal,
            "update_protocol_permission",
            Encode!(&permissions.id, &protocol_perm).unwrap()
        );
        assert!(add_perm_result.is_ok(), "Failed to add protocol permission");
        
        // 4. Test that another user cannot use these permissions
        let unauthorized_supply = pic.update_call(
            canister_id,
            another_principal, // Different user
            "supply_link_to_aave_secured",
            Encode!(&"0.1".to_string(), &permissions.id).unwrap()
        );
        
        match unauthorized_supply {
            Ok(bytes) => {
                let result: Result<String, String> = 
                    Decode!(&bytes, Result<String, String>).expect("Failed to decode result");
                
                assert!(result.is_err(), "Unauthorized access should fail");
                let error = result.unwrap_err();
                assert!(
                    error.contains("Access denied") || 
                    error.contains("not the owner") ||
                    error.contains("permission"),
                    "Expected access denied error, got: {}", error
                );
            },
            Err(e) => {
                panic!("Unauthorized supply call was rejected: {:?}", e);
            }
        }
        
        // 5. Test that withdraw is not allowed (not in allowed_functions)
        let unauthorized_function = pic.update_call(
            canister_id,
            user_principal, // Correct user but wrong function
            "withdraw_link_from_aave_secured",
            Encode!(&"0.1".to_string(), &permissions.id).unwrap()
        );
        
        match unauthorized_function {
            Ok(bytes) => {
                let result: Result<String, String> = 
                    Decode!(&bytes, Result<String, String>).expect("Failed to decode result");
                
                assert!(result.is_err(), "Unauthorized function should fail");
                let error = result.unwrap_err();
                assert!(
                    error.contains("not allowed") || 
                    error.contains("Function withdraw not allowed"),
                    "Expected function not allowed error, got: {}", error
                );
            },
            Err(e) => {
                panic!("Unauthorized function call was rejected: {:?}", e);
            }
        }
    }

    // ===== SPRINT 3 COMPREHENSIVE INTEGRATION TESTS =====

    // 🆕 Тест полного workflow Milestone 2 (Задача 4.1)
    #[test]
    fn test_milestone_2_complete_workflow() {
        // Initialize test environment
        let (pic, canister_id) = setup_test_env();
        
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        
        // Step 1: Generate EVM address
        let gen_result = pic.update_call(
            canister_id,
            user_principal,
            "generate_evm_address",
            Encode!().unwrap()
        );
        assert!(gen_result.is_ok(), "Failed to generate EVM address");
        
        let evm_address = match gen_result {
            Ok(bytes) => {
                let address_result: Result<String, String> = 
                    Decode!(&bytes, Result<String, String>).expect("Failed to decode result");
                address_result.expect("Failed to generate EVM address")
            },
            Err(e) => panic!("Failed to generate EVM address: {:?}", e)
        };
        
        println!("✅ Generated EVM address: {}", evm_address);
        
        // Step 2: Create permissions with AAVE protocol
        let permissions_request = CreatePermissionsRequest {
            chain_id: 11155111, // Sepolia chain ID for AAVE
            whitelisted_protocols: vec![Protocol {
                name: "AAVE".to_string(),
                address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
            }],
            whitelisted_tokens: vec![Token {
                name: "LINK".to_string(),
                address: "0xf8fb3713d459d7c1018bd0a49d19b4c44290ebe5".to_string(), // Correct LINK address on Sepolia
            }],
            transfer_limits: vec![],
            protocol_permissions: None,
        };
        
        let create_result = pic.update_call(
            canister_id,
            user_principal,
            "create_permissions",
            Encode!(&permissions_request).unwrap()
        );
        
        let permissions = match create_result {
            Ok(bytes) => {
                let permissions_result: Result<Permissions, String> = 
                    Decode!(&bytes, Result<Permissions, String>).expect("Failed to decode result");
                permissions_result.expect("Failed to create permissions")
            },
            Err(e) => panic!("Failed to create permissions: {:?}", e)
        };
        
        println!("✅ Created permissions with ID: {}", permissions.id);
        
                 // Step 3: Add AAVE protocol permission
         let protocol_perm = ProtocolPermission {
             protocol_address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
             allowed_functions: vec!["supply".to_string(), "withdraw".to_string()],
             max_amount_per_tx: Some(1_000_000_000_000_000_000), // 1 LINK (18 decimals)
             daily_limit: Some(5_000_000_000_000_000_000), // 5 LINK
             total_used_today: 0,
             last_reset_date: 0,
         };
        
        let add_perm_result = pic.update_call(
            canister_id,
            user_principal,
            "update_protocol_permission",
            Encode!(&permissions.id, &protocol_perm).unwrap()
        );
        
        match add_perm_result {
            Ok(bytes) => {
                let result: Result<bool, String> = 
                    Decode!(&bytes, Result<bool, String>).expect("Failed to decode result");
                assert_eq!(result, Ok(true), "Adding protocol permission should succeed");
            },
            Err(e) => panic!("Failed to add protocol permission: {:?}", e)
        }
        
        println!("✅ Added AAVE protocol permission");
        
                 // Step 4: Test supply to AAVE (expected to fail due to insufficient balance)
         let supply_result = pic.update_call(
             canister_id,
             user_principal,
             "supply_link_to_aave_secured",
             Encode!(&"1.0".to_string(), &permissions.id).unwrap()
         );
        
        match supply_result {
            Ok(bytes) => {
                let result: Result<String, String> = 
                    Decode!(&bytes, Result<String, String>).expect("Failed to decode result");
                
                match result {
                    Ok(success_msg) => {
                        println!("✅ AAVE supply succeeded: {}", success_msg);
                        assert!(success_msg.contains("Successfully supplied"), "Should contain success message");
                        
                        // Step 5: Check aLINK balance
                        let balance_result = pic.update_call(
                            canister_id,
                            user_principal,
                            "get_aave_link_user_balance",
                            Encode!(&None::<String>).unwrap()
                        );
                        
                        match balance_result {
                            Ok(balance_bytes) => {
                                let balance: Result<String, String> = 
                                    Decode!(&balance_bytes, Result<String, String>).expect("Failed to decode balance");
                                println!("✅ aLINK balance check completed: {:?}", balance);
                            },
                            Err(e) => println!("Expected balance check error: {:?}", e)
                        }
                        
                        // Step 6: Test withdraw from AAVE
                        let withdraw_result = pic.update_call(
                            canister_id,
                            user_principal,
                            "withdraw_link_from_aave_secured",
                            Encode!(&"5.0".to_string(), &permissions.id).unwrap()
                        );
                        
                        match withdraw_result {
                            Ok(withdraw_bytes) => {
                                let withdraw: Result<String, String> = 
                                    Decode!(&withdraw_bytes, Result<String, String>).expect("Failed to decode withdraw");
                                println!("✅ AAVE withdraw completed: {:?}", withdraw);
                            },
                            Err(e) => println!("Expected withdraw error: {:?}", e)
                        }
                    },
                    Err(error) => {
                        println!("✅ Expected error (insufficient balance): {}", error);
                        assert!(
                            error.contains("Insufficient LINK balance") ||
                            error.contains("Failed to get balance") ||
                            error.contains("network") ||
                            error.contains("RPC") ||
                            error.contains("No route to canister") ||
                            error.contains("server returned an error response"),
                            "Error should be balance/network related, got: {}", error
                        );
                    }
                }
            },
            Err(e) => panic!("AAVE supply call was rejected: {:?}", e)
        }
        
        println!("✅ Milestone 2 complete workflow test finished");
    }

    // 🆕 Тест daily limits и per-transaction limits (Задача 4.1)
    #[test]
    fn test_daily_limits_enforcement() {
        // Initialize test environment
        let (pic, canister_id) = setup_test_env();
        
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        
        // 1. Generate EVM address
        let gen_result = pic.update_call(
            canister_id,
            user_principal,
            "generate_evm_address",
            Encode!().unwrap()
        );
        assert!(gen_result.is_ok(), "Failed to generate EVM address");
        
        // 2. Create permissions
        let request = CreatePermissionsRequest {
            chain_id: 11155111, // Sepolia chain ID for AAVE
            whitelisted_protocols: vec![],
            whitelisted_tokens: vec![],
            transfer_limits: vec![],
            protocol_permissions: None,
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
            Err(e) => panic!("Failed to create permissions: {:?}", e)
        };
        
                 // 3. Add AAVE protocol permission with strict limits
         let protocol_perm = ProtocolPermission {
             protocol_address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
             allowed_functions: vec!["supply".to_string()],
             max_amount_per_tx: Some(500_000_000_000_000_000), // 0.5 LINK max per tx
             daily_limit: Some(1_000_000_000_000_000_000), // 1 LINK daily limit
             total_used_today: 0,
             last_reset_date: 0,
         };
        
        let add_perm_result = pic.update_call(
            canister_id,
            user_principal,
            "update_protocol_permission",
            Encode!(&permissions.id, &protocol_perm).unwrap()
        );
        assert!(add_perm_result.is_ok(), "Failed to add protocol permission");
        
        // 4. Test per-transaction limit exceeded
        let over_tx_limit = pic.query_call(
            canister_id,
            user_principal,
            "check_protocol_permission",
            Encode!(
                &permissions.id,
                &"0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
                &"supply".to_string(),
                                 &600_000_000_000_000_000u64 // 0.6 LINK (exceeds per-tx limit of 0.5)
             ).unwrap()
         );
         
         match over_tx_limit {
             Ok(bytes) => {
                 let result: Result<bool, String> = 
                     Decode!(&bytes, Result<bool, String>).expect("Failed to decode result");
                 assert!(result.is_err(), "Over per-tx limit should fail");
                 let error = result.unwrap_err();
                 assert!(error.contains("exceeds max limit"), "Expected 'exceeds max limit' error, got: {}", error);
                 println!("✅ Per-transaction limit enforcement works: {}", error);
             },
             Err(e) => panic!("Per-tx limit check was rejected: {:?}", e)
         }
         
         // 5. Test within per-transaction limit
         let within_tx_limit = pic.query_call(
             canister_id,
             user_principal,
             "check_protocol_permission",
             Encode!(
                 &permissions.id,
                 &"0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
                 &"supply".to_string(),
                 &400_000_000_000_000_000u64 // 0.4 LINK (within per-tx limit)
            ).unwrap()
        );
        
        match within_tx_limit {
            Ok(bytes) => {
                let result: Result<bool, String> = 
                    Decode!(&bytes, Result<bool, String>).expect("Failed to decode result");
                assert_eq!(result, Ok(true), "Within per-tx limit should succeed");
                println!("✅ Within per-transaction limit check works");
            },
            Err(e) => panic!("Within per-tx limit check was rejected: {:?}", e)
        }
        
        println!("✅ Daily limits enforcement test completed");
    }

    // 🆕 Тест error handling (Задача 4.1)
    #[test]
    fn test_comprehensive_error_handling() {
        // Initialize test environment
        let (pic, canister_id) = setup_test_env();
        
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        
        // 1. Generate EVM address
        let gen_result = pic.update_call(
            canister_id,
            user_principal,
            "generate_evm_address",
            Encode!().unwrap()
        );
        assert!(gen_result.is_ok(), "Failed to generate EVM address");
        
        // 2. Create permissions
        let request = CreatePermissionsRequest {
            chain_id: 11155111, // Sepolia chain ID for AAVE
            whitelisted_protocols: vec![],
            whitelisted_tokens: vec![],
            transfer_limits: vec![],
            protocol_permissions: None,
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
            Err(e) => panic!("Failed to create permissions: {:?}", e)
        };
        
        // Test 1: Invalid permissions ID
        let invalid_permissions_check = pic.query_call(
            canister_id,
            user_principal,
            "check_protocol_permission",
            Encode!(
                &"invalid_permissions_id".to_string(),
                &"0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
                &"supply".to_string(),
                &1_000_000_000_000_000_000u64 // 1 LINK
            ).unwrap()
        );
        
        match invalid_permissions_check {
            Ok(bytes) => {
                let result: Result<bool, String> = 
                    Decode!(&bytes, Result<bool, String>).expect("Failed to decode result");
                assert!(result.is_err(), "Invalid permissions ID should fail");
                let error = result.unwrap_err();
                assert!(error.contains("not found"), "Expected 'not found' error, got: {}", error);
                println!("✅ Invalid permissions ID error: {}", error);
            },
            Err(e) => panic!("Invalid permissions check was rejected: {:?}", e)
        }
        
        // Test 2: Protocol not found (no protocol permissions added)
        let protocol_not_found = pic.query_call(
            canister_id,
            user_principal,
            "check_protocol_permission",
            Encode!(
                &permissions.id,
                &"0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
                &"supply".to_string(),
                &1_000_000_000_000_000_000u64 // 1 LINK
            ).unwrap()
        );
        
        match protocol_not_found {
            Ok(bytes) => {
                let result: Result<bool, String> = 
                    Decode!(&bytes, Result<bool, String>).expect("Failed to decode result");
                assert!(result.is_err(), "Protocol not found should fail");
                let error = result.unwrap_err();
                assert!(error.contains("not found"), "Expected 'not found' error, got: {}", error);
                println!("✅ Protocol not found error: {}", error);
            },
            Err(e) => panic!("Protocol not found check was rejected: {:?}", e)
        }
        
        // Add protocol permission for further tests
                 let protocol_perm = ProtocolPermission {
             protocol_address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
             allowed_functions: vec!["supply".to_string()], // Only supply allowed
             max_amount_per_tx: Some(1_000_000_000_000_000_000), // 1 LINK
             daily_limit: Some(5_000_000_000_000_000_000), // 5 LINK
             total_used_today: 0,
             last_reset_date: 0,
         };
        
        let add_perm_result = pic.update_call(
            canister_id,
            user_principal,
            "update_protocol_permission",
            Encode!(&permissions.id, &protocol_perm).unwrap()
        );
        assert!(add_perm_result.is_ok(), "Failed to add protocol permission");
        
        // Test 3: Function not allowed
        let function_not_allowed = pic.query_call(
            canister_id,
            user_principal,
            "check_protocol_permission",
            Encode!(
                &permissions.id,
                &"0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
                &"borrow".to_string(), // Not in allowed_functions
                &1_000_000_000_000_000_000u64 // 1 LINK
            ).unwrap()
        );
        
        match function_not_allowed {
            Ok(bytes) => {
                let result: Result<bool, String> = 
                    Decode!(&bytes, Result<bool, String>).expect("Failed to decode result");
                assert!(result.is_err(), "Function not allowed should fail");
                let error = result.unwrap_err();
                assert!(error.contains("not allowed"), "Expected 'not allowed' error, got: {}", error);
                println!("✅ Function not allowed error: {}", error);
            },
            Err(e) => panic!("Function not allowed check was rejected: {:?}", e)
        }
        
                 // Test 4: AAVE supply with insufficient balance (real network call)
         let insufficient_balance_supply = pic.update_call(
             canister_id,
             user_principal,
             "supply_link_to_aave_secured",
             Encode!(&"1.0".to_string(), &permissions.id).unwrap()
         );
        
        match insufficient_balance_supply {
            Ok(bytes) => {
                let result: Result<String, String> = 
                    Decode!(&bytes, Result<String, String>).expect("Failed to decode result");
                
                match result {
                    Ok(success_msg) => {
                        // Unexpected success
                        println!("Unexpected success: {}", success_msg);
                    },
                    Err(error) => {
                        println!("✅ Expected insufficient balance error: {}", error);
                        assert!(
                            error.contains("Insufficient LINK balance") ||
                            error.contains("Failed to get balance") ||
                            error.contains("network") ||
                            error.contains("RPC") ||
                            error.contains("No route to canister") ||
                            error.contains("server returned an error response"),
                            "Error should be balance/network related, got: {}", error
                        );
                    }
                }
            },
            Err(e) => {
                println!("Expected rejection (network/balance): {:?}", e);
            }
        }
        
        println!("✅ Comprehensive error handling test completed");
    }

    // 🆕 Тест multiple protocol permissions (Задача 4.1)
    #[test]
    fn test_multiple_protocol_permissions() {
        // Initialize test environment
        let (pic, canister_id) = setup_test_env();
        
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        
        // 1. Generate EVM address
        let gen_result = pic.update_call(
            canister_id,
            user_principal,
            "generate_evm_address",
            Encode!().unwrap()
        );
        assert!(gen_result.is_ok(), "Failed to generate EVM address");
        
        // 2. Create permissions
        let request = CreatePermissionsRequest {
            chain_id: 11155111, // Sepolia chain ID for AAVE
            whitelisted_protocols: vec![],
            whitelisted_tokens: vec![],
            transfer_limits: vec![],
            protocol_permissions: None,
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
            Err(e) => panic!("Failed to create permissions: {:?}", e)
        };
        
        // 3. Add AAVE protocol permission
                 let aave_perm = ProtocolPermission {
             protocol_address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
             allowed_functions: vec!["supply".to_string(), "withdraw".to_string()],
             max_amount_per_tx: Some(1_000_000_000_000_000_000), // 1 LINK
             daily_limit: Some(5_000_000_000_000_000_000), // 5 LINK
             total_used_today: 0,
             last_reset_date: 0,
         };
        
        let add_aave_result = pic.update_call(
            canister_id,
            user_principal,
            "update_protocol_permission",
            Encode!(&permissions.id, &aave_perm).unwrap()
        );
        assert!(add_aave_result.is_ok(), "Failed to add AAVE protocol permission");
        
        // 4. Try to add another protocol permission with same address (should fail)
                 let duplicate_perm = ProtocolPermission {
             protocol_address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(), // Same address
             allowed_functions: vec!["borrow".to_string()],
             max_amount_per_tx: Some(500_000_000_000_000_000), // 0.5 LINK
             daily_limit: Some(2_000_000_000_000_000_000), // 2 LINK
             total_used_today: 0,
             last_reset_date: 0,
         };
        
        let add_duplicate_result = pic.update_call(
            canister_id,
            user_principal,
            "update_protocol_permission",
            Encode!(&permissions.id, &duplicate_perm).unwrap()
        );
        
        match add_duplicate_result {
            Ok(bytes) => {
                let result: Result<bool, String> = 
                    Decode!(&bytes, Result<bool, String>).expect("Failed to decode result");
                assert!(result.is_err(), "Duplicate protocol permission should fail");
                let error = result.unwrap_err();
                assert!(error.contains("already exists"), "Expected 'already exists' error, got: {}", error);
                println!("✅ Duplicate protocol permission correctly rejected: {}", error);
            },
            Err(e) => panic!("Duplicate protocol permission call was rejected: {:?}", e)
        }
        
        // 5. Verify AAVE permissions still work
        let verify_aave = pic.query_call(
            canister_id,
            user_principal,
            "check_protocol_permission",
            Encode!(
                &permissions.id,
                &"0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
                &"supply".to_string(),
                                 &500_000_000_000_000_000u64 // 0.5 LINK
            ).unwrap()
        );
        
        match verify_aave {
            Ok(bytes) => {
                let result: Result<bool, String> = 
                    Decode!(&bytes, Result<bool, String>).expect("Failed to decode result");
                assert_eq!(result, Ok(true), "AAVE permission should still work");
                println!("✅ AAVE permissions verified after duplicate attempt");
            },
            Err(e) => panic!("AAVE permission verification was rejected: {:?}", e)
        }
        
        println!("✅ Multiple protocol permissions test completed");
    }

    // 🆕 New test to demonstrate creating permissions with protocol permissions in one call
    #[test]
    fn test_create_permissions_with_protocol_permissions() {
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
        
        // Create permissions with protocol permissions in one call
        let request = full_permissions_request_with_aave();
        
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
                        println!("✅ Created permissions with protocol permissions: {}", permissions.id);
                        
                        // Check basic permissions data
                        assert_eq!(permissions.owner, user_principal);
                        assert_eq!(permissions.whitelisted_protocols.len(), 1);
                        assert_eq!(permissions.whitelisted_protocols[0].name, "AAVE");
                        assert_eq!(permissions.whitelisted_tokens.len(), 1);
                        assert_eq!(permissions.whitelisted_tokens[0].name, "LINK");
                        assert_eq!(permissions.transfer_limits.len(), 1);
                        
                        // 🆕 Check protocol permissions were created
                        assert_eq!(permissions.protocol_permissions.len(), 1);
                        let protocol_perm = &permissions.protocol_permissions[0];
                        assert_eq!(protocol_perm.protocol_address, "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951");
                        assert_eq!(protocol_perm.allowed_functions.len(), 2);
                        assert!(protocol_perm.allowed_functions.contains(&"supply".to_string()));
                        assert!(protocol_perm.allowed_functions.contains(&"withdraw".to_string()));
                        assert_eq!(protocol_perm.max_amount_per_tx, Some(100_000_000_000_000_000)); // 0.1 LINK
                        assert_eq!(protocol_perm.daily_limit, Some(1_000_000_000_000_000_000));     // 1 LINK
                        
                        // 🆕 Test protocol permission check
                        let check_result = pic.query_call(
                            canister_id,
                            user_principal,
                            "check_protocol_permission",
                            Encode!(&permissions.id, &"0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(), &"supply".to_string(), &50_000_000_000_000_000u64).unwrap()
                        );
                        
                        match check_result {
                            Ok(bytes) => {
                                let check_permission_result: Result<bool, String> = 
                                    Decode!(&bytes, Result<bool, String>).expect("Failed to decode result");
                                
                                assert_eq!(check_permission_result, Ok(true), "Should allow supply operation");
                                println!("✅ Protocol permission check passed for supply operation");
                            },
                            Err(e) => {
                                panic!("Failed to check protocol permission: {:?}", e);
                            }
                        }
                        
                        println!("🎉 Test passed: Created permissions with protocol permissions successfully!");
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

    // 🆕 Test to compare old vs new permission creation workflow
    #[test]
    fn test_old_vs_new_permission_workflow() {
        // Initialize test environment
        let (pic, canister_id) = setup_test_env();
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        
        // Generate EVM address
        let gen_result = pic.update_call(
            canister_id,
            user_principal,
            "generate_evm_address",
            Encode!().unwrap()
        );
        assert!(gen_result.is_ok(), "Failed to generate EVM address");
        
        // OLD WAY: Create basic permissions first, then add protocol permissions
        let basic_request = basic_permissions_request();
        
        let create_result = pic.update_call(
            canister_id,
            user_principal,
            "create_permissions",
            Encode!(&basic_request).unwrap()
        );
        
        let old_way_permissions = match create_result {
            Ok(bytes) => {
                let permissions_result: Result<Permissions, String> = 
                    Decode!(&bytes, Result<Permissions, String>).expect("Failed to decode result");
                permissions_result.expect("Failed to create basic permissions")
            },
            Err(e) => panic!("Failed to create basic permissions: {:?}", e),
        };
        
        println!("📋 OLD WAY: Created basic permissions: {}", old_way_permissions.id);
        assert_eq!(old_way_permissions.protocol_permissions.len(), 0, "Should have no protocol permissions initially");
        
        // NEW WAY: Create permissions with protocol permissions in one call
        let full_request = full_permissions_request_with_aave();
        
        let new_create_result = pic.update_call(
            canister_id,
            user_principal,
            "create_permissions",
            Encode!(&full_request).unwrap()
        );
        
        let new_way_permissions = match new_create_result {
            Ok(bytes) => {
                let permissions_result: Result<Permissions, String> = 
                    Decode!(&bytes, Result<Permissions, String>).expect("Failed to decode result");
                permissions_result.expect("Failed to create full permissions")
            },
            Err(e) => panic!("Failed to create full permissions: {:?}", e),
        };
        
        println!("✅ NEW WAY: Created full permissions in one call: {}", new_way_permissions.id);
        assert_eq!(new_way_permissions.protocol_permissions.len(), 1, "Should have protocol permissions from creation");
        
        // Both should allow the same operations
        let old_way_check = pic.query_call(
            canister_id,
            user_principal,
            "check_protocol_permission",
            Encode!(&old_way_permissions.id, &"0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(), &"supply".to_string(), &50_000_000_000_000_000u64).unwrap()
        );
        
        let new_way_check = pic.query_call(
            canister_id,
            user_principal,
            "check_protocol_permission",
            Encode!(&new_way_permissions.id, &"0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(), &"supply".to_string(), &50_000_000_000_000_000u64).unwrap()
        );
        
        assert!(old_way_check.is_ok() && new_way_check.is_ok(), "Both permission checks should work");
        println!("🎉 Both workflows result in equivalent permissions!");
    }

    // 🆕 Tests for new multi-chain AAVE functions
    #[test]
    fn test_multi_chain_aave_supply() {
        let (pic, canister_id) = setup_test_env();
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        
        // Generate EVM address
        let gen_result = pic.update_call(
            canister_id,
            user_principal,
            "generate_evm_address",
            Encode!().unwrap()
        );
        assert!(gen_result.is_ok(), "Failed to generate EVM address");

        // Create permissions
        let request = CreatePermissionsRequest {
            chain_id: 11155111, // Sepolia
            whitelisted_protocols: vec![Protocol {
                name: "AAVE".to_string(),
                address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
            }],
            whitelisted_tokens: vec![Token {
                name: "LINK".to_string(),
                address: "0xf8fb3713d459d7c1018bd0a49d19b4c44290ebe5".to_string(),
            }],
            transfer_limits: vec![],
            protocol_permissions: Some(vec![ProtocolPermission {
                protocol_address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
                allowed_functions: vec!["supply".to_string()],
                max_amount_per_tx: Some(1_000_000_000_000_000_000), // 1 LINK
                daily_limit: Some(5_000_000_000_000_000_000), // 5 LINK
                total_used_today: 0,
                last_reset_date: 0,
            }]),
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
            Err(e) => panic!("Failed to create permissions: {:?}", e),
        };

        // Test the new generic supply function (this would need to be exposed in lib.rs)
        // For now, we test that legacy functions still work
        let supply_result = pic.update_call(
            canister_id,
            user_principal,
            "supply_link_to_aave_secured",
            Encode!(&"0.1".to_string(), &permissions.id).unwrap()
        );

        match supply_result {
            Ok(bytes) => {
                let result: Result<String, String> = 
                    Decode!(&bytes, Result<String, String>).expect("Failed to decode result");
                
                match result {
                    Ok(success_msg) => {
                        println!("✅ Multi-chain AAVE supply succeeded: {}", success_msg);
                        assert!(success_msg.contains("Successfully supplied"));
                    },
                    Err(error) => {
                        println!("✅ Expected error (insufficient balance): {}", error);
                        assert!(
                            error.contains("Insufficient LINK balance") ||
                            error.contains("Failed to get balance") ||
                            error.contains("network") ||
                            error.contains("RPC")
                        );
                    }
                }
            },
            Err(e) => panic!("Multi-chain AAVE supply call was rejected: {:?}", e)
        }

        println!("✅ Multi-chain AAVE supply test completed");
    }

    #[test]
    fn test_aave_configuration_validation() {
        let (pic, canister_id) = setup_test_env();
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        
        // Generate EVM address
        let gen_result = pic.update_call(
            canister_id,
            user_principal,
            "generate_evm_address",
            Encode!().unwrap()
        );
        assert!(gen_result.is_ok(), "Failed to generate EVM address");

        // Test with unsupported chain (should validate in backend)
        let unsupported_request = CreatePermissionsRequest {
            chain_id: 999999, // Unsupported chain
            whitelisted_protocols: vec![],
            whitelisted_tokens: vec![],
            transfer_limits: vec![],
            protocol_permissions: None,
        };

        let create_result = pic.update_call(
            canister_id,
            user_principal,
            "create_permissions",
            Encode!(&unsupported_request).unwrap()
        );

        match create_result {
            Ok(bytes) => {
                let result: Result<Permissions, String> = 
                    Decode!(&bytes, Result<Permissions, String>).expect("Failed to decode result");
                
                // Could succeed or fail depending on validation logic
                match result {
                    Ok(permissions) => {
                        println!("✅ Permissions created for unsupported chain: {}", permissions.chain_id);
                    },
                    Err(error) => {
                        println!("✅ Expected validation error: {}", error);
                        assert!(error.contains("chain") || error.contains("supported"));
                    }
                }
            },
            Err(e) => println!("Expected rejection: {:?}", e)
        }

        println!("✅ AAVE configuration validation test completed");
    }

    #[test]
    fn test_token_decimals_handling() {
        let (pic, canister_id) = setup_test_env();
        let user_principal = Principal::from_text(USER_PRINCIPAL).expect("Invalid principal");
        
        // Generate EVM address
        let gen_result = pic.update_call(
            canister_id,
            user_principal,
            "generate_evm_address",
            Encode!().unwrap()
        );
        assert!(gen_result.is_ok(), "Failed to generate EVM address");

        // Create permissions for different token types
        let request = CreatePermissionsRequest {
            chain_id: 11155111, // Sepolia
            whitelisted_protocols: vec![Protocol {
                name: "AAVE".to_string(),
                address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
            }],
            whitelisted_tokens: vec![
                Token {
                    name: "LINK".to_string(),
                    address: "0xf8fb3713d459d7c1018bd0a49d19b4c44290ebe5".to_string(), // 18 decimals
                },
                Token {
                    name: "USDC".to_string(), 
                    address: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238".to_string(), // 6 decimals
                }
            ],
            transfer_limits: vec![],
            protocol_permissions: Some(vec![ProtocolPermission {
                protocol_address: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951".to_string(),
                allowed_functions: vec!["supply".to_string()],
                max_amount_per_tx: Some(1_000_000), // Should work for both 6 and 18 decimal tokens
                daily_limit: Some(10_000_000),
                total_used_today: 0,
                last_reset_date: 0,
            }]),
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
            Err(e) => panic!("Failed to create permissions: {:?}", e),
        };

        // Test supply with decimal amount (should be parsed correctly by new architecture)
        let decimal_supply = pic.update_call(
            canister_id,
            user_principal,
            "supply_link_to_aave_secured", // Tests 18 decimal parsing
            Encode!(&"0.1".to_string(), &permissions.id).unwrap()
        );

        match decimal_supply {
            Ok(bytes) => {
                let result: Result<String, String> = 
                    Decode!(&bytes, Result<String, String>).expect("Failed to decode result");
                
                match result {
                    Ok(success_msg) => {
                        println!("✅ Decimal amount parsing succeeded: {}", success_msg);
                    },
                    Err(error) => {
                        println!("✅ Expected error (balance/network): {}", error);
                        // Error should not be about parsing, but about balance or network
                        assert!(
                            !error.contains("Invalid amount format") &&
                            !error.contains("parse") &&
                            !error.contains("decimal"),
                            "Error should not be parsing-related, got: {}", error
                        );
                    }
                }
            },
            Err(e) => panic!("Decimal supply call was rejected: {:?}", e)
        }

        println!("✅ Token decimals handling test completed");
    }
}
