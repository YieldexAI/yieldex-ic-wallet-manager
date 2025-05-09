# yieldex-ic-wallet-manager

A Rust-based Internet Computer (IC) canister for managing EVM addresses and user permissions, with full test coverage using [PocketIC](https://github.com/dfinity/ic/tree/master/packages/pocket-ic) for local testing.

## Features
- **EVM Address Management**: Generate and store EVM addresses for IC principals using ECDSA.
- **User Verification**: Verify users based on EVM address existence.
- **Permissions Management**: Create, update, query, and delete user permissions (protocols, tokens, transfer limits).
- **Comprehensive Rust Tests**: All core logic is covered by Rust tests using PocketIC, including ECDSA key mocking and access control scenarios.

## Project Structure
```
/ ├── src/
│   └── yieldex-ic-wallet-manager-backend/   # Rust canister code
├── tests/                                   # Rust integration tests (PocketIC)
│   └── pocket_ic_tests.rs
├── README.md
├── Cargo.toml
└── ...
```

## Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (recommended: latest stable)
- [DFX SDK](https://internetcomputer.org/docs/current/developer-docs/setup/install)
- [PocketIC](https://github.com/dfinity/ic/tree/master/packages/pocket-ic) (as a Rust dependency)

## Building the Canister

```bash
# From the project root
dfx start --background
dfx build
```

This will produce the WASM file at:
```
.dfx/local/canisters/yieldex-ic-wallet-manager-backend/yieldex-ic-wallet-manager-backend.wasm
```

## Running Rust Tests with PocketIC

All core logic is tested in `tests/pocket_ic_tests.rs` using PocketIC. These tests cover:
- EVM address generation and retrieval
- User verification
- Permissions CRUD (create, read, update, delete)
- Access control and error scenarios

To run the tests:

```bash
cd tests
RUST_BACKTRACE=1 cargo test -- --nocapture
```

You can change the test principal IDs in `pocket_ic_tests.rs` to simulate different users.

## Notes
- The canister uses the ECDSA key name `dfx_test_key` for compatibility with PocketIC's II subnet.
- All test structures are defined in the test file for compatibility with Candid serialization.
- No frontend or npm scripts are included; this project is backend/canister and Rust-test focused.

## Useful Links
- [Internet Computer Rust Canister Development](https://internetcomputer.org/docs/current/developer-docs/backend/rust/)
- [PocketIC Rust Testing](https://github.com/dfinity/ic/tree/master/packages/pocket-ic)
- [ic-cdk](https://docs.rs/ic-cdk)
- [Candid](https://internetcomputer.org/docs/current/developer-docs/backend/candid/)
