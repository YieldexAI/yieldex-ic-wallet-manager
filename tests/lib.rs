// Testing Rust canisters on Internet Computer
// 
// For testing the now() function and ic_cdk API, a more reliable approach is
// to use dfx deploy and test the function directly via command line:
//
// dfx start --clean
// dfx deploy
// dfx canister call yieldex-ic-wallet-manager-backend now
//
// PocketIC has compatibility issues with latest dependency versions,
// so for simple unit testing it's better to use direct dfx calls

// Testing Rust canisters on Internet Computer using PocketIC
//
// To run the tests:
// 1. Install PocketIC server and export POCKET_IC_BIN variable
// 2. Run tests: cargo test -p yieldex-ic-wallet-manager-tests

#[cfg(test)]
pub mod pocket_ic_tests; 