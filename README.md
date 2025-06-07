# yieldex-ic-wallet-manager

A comprehensive Rust-based Internet Computer (IC) canister for managing EVM addresses, token interactions, and DeFi operations on Ethereum Sepolia testnet using the ic-alloy library with threshold ECDSA signatures.

## 🚀 Features

### Core Wallet Management
- **EVM Address Generation**: Generate unique EVM addresses for IC principals using threshold ECDSA
- **User Verification**: Verify users based on EVM address existence
- **Permissions Management**: Create, update, query, and delete user permissions for protocols and tokens

### Token Operations
- **Balance Checking**: Get balances for ETH, USDC, LINK, and WETH tokens
- **Token Transfers**: Send LINK tokens and ETH with nonce management
- **Token Approvals**: Approve spending for USDC and WETH tokens for DeFi protocols

### ETH/WETH Operations
- **ETH Wrapping**: Convert ETH to WETH (Wrapped ETH) for DeFi compatibility
- **ETH Unwrapping**: Convert WETH back to ETH
- **WETH Management**: Full WETH balance, approval, and transfer functionality

### Message Signing
- **Arbitrary Message Signing**: Sign any message using threshold ECDSA
- **Hash Signing**: Sign 32-byte hashes directly

### Supported Networks & Tokens
- **Network**: Ethereum Sepolia Testnet
- **Tokens**: 
  - ETH (Native)
  - USDC: `0x1c7d4b196cb0c7b01d743fbc6116a902379c7238`
  - LINK: `0x779877A7B0D9E8603169DdbD7836e478b4624789`
  - WETH: `0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9`

## 🏗️ Project Structure

```
yieldex-ic-wallet-manager/
├── src/
│   └── yieldex-ic-wallet-manager-backend/   # Rust canister code
│       ├── src/
│       │   ├── services/                    # Service modules
│       │   │   ├── get_balance.rs          # ETH balance checking
│       │   │   ├── get_balance_usdc.rs     # USDC balance checking
│       │   │   ├── get_balance_link.rs     # LINK balance checking
│       │   │   ├── transfer_link.rs        # LINK token transfers
│       │   │   ├── send_eth.rs             # ETH sending
│       │   │   ├── approve_usdc.rs         # USDC approvals
│       │   │   ├── approve_weth.rs         # WETH approvals
│       │   │   ├── wrap_eth.rs             # ETH wrapping/unwrapping
│       │   │   └── sign_message.rs         # Message signing
│       │   ├── abi/                        # Contract ABIs
│       │   └── lib.rs                      # Main canister logic
│       └── Cargo.toml
├── tests/                                   # Rust integration tests (PocketIC)
│   └── pocket_ic_tests.rs
├── project_docs/                            # Documentation
├── README.md
└── dfx.json
```

## 🛠️ Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [DFX SDK](https://internetcomputer.org/docs/current/developer-docs/setup/install) (latest version)
- [PocketIC](https://github.com/dfinity/ic/tree/master/packages/pocket-ic) (for testing)

## 🚀 Quick Start

### 1. Build the Canister

```bash
# Start DFX
dfx start --background

# Build the canister
dfx build
```

### 2. Deploy to Local Network

```bash
# Deploy to local replica
dfx deploy
```

### 3. Deploy to IC Mainnet

```bash
# Deploy to IC mainnet
dfx deploy --network ic
```

## 📋 API Reference

### EVM Address Management

```bash
# Generate EVM address for caller
dfx canister call yieldex-ic-wallet-manager-backend generate_evm_address

# Get stored EVM address
dfx canister call yieldex-ic-wallet-manager-backend get_evm_address

# Verify if user has EVM address
dfx canister call yieldex-ic-wallet-manager-backend verify_user '(principal "your-principal-id")'
```

### Balance Checking

```bash
# Get ETH balance (your address)
dfx canister call yieldex-ic-wallet-manager-backend get_eth_balance '(null)'

# Get ETH balance (specific address)
dfx canister call yieldex-ic-wallet-manager-backend get_eth_balance '(opt "0x1234...")'

# Get USDC balance
dfx canister call yieldex-ic-wallet-manager-backend get_usdc_balance '(null)'

# Get LINK balance
dfx canister call yieldex-ic-wallet-manager-backend get_link_balance '(null)'

# Get WETH balance
dfx canister call yieldex-ic-wallet-manager-backend get_weth_token_balance '(null)'
```

### Token Transfers

```bash
# Send ETH (raw wei amount)
dfx canister call yieldex-ic-wallet-manager-backend send_eth_tokens '("0x1234...", "1000000000000000000")'

# Send ETH (human-readable amount)
dfx canister call yieldex-ic-wallet-manager-backend send_eth_human_readable '("0x1234...", "0.001")'

# Transfer LINK tokens (raw amount)
dfx canister call yieldex-ic-wallet-manager-backend transfer_link_tokens '("0x1234...", "1000000000000000000")'

# Transfer LINK tokens (human-readable)
dfx canister call yieldex-ic-wallet-manager-backend transfer_link_human_readable '("0x1234...", "1.5")'
```

### Token Approvals

```bash
# Approve USDC spending (raw amount)
dfx canister call yieldex-ic-wallet-manager-backend approve_usdc_spending '("0x1234...", "1000000")'

# Approve USDC spending (human-readable)
dfx canister call yieldex-ic-wallet-manager-backend approve_usdc_human_readable '("0x1234...", "100.0")'

# Approve WETH spending
dfx canister call yieldex-ic-wallet-manager-backend approve_weth_spending '("0x1234...", "1000000000000000000")'

# Approve WETH for Uniswap
dfx canister call yieldex-ic-wallet-manager-backend approve_weth_for_uniswap_trading '"1000000000000000000"'

# Revoke approvals (set to 0)
dfx canister call yieldex-ic-wallet-manager-backend revoke_usdc_spending_approval '"0x1234..."'
dfx canister call yieldex-ic-wallet-manager-backend revoke_weth_spending_approval '"0x1234..."'

# Check allowances
dfx canister call yieldex-ic-wallet-manager-backend get_usdc_allowance_info '(null, "0x1234...")'
dfx canister call yieldex-ic-wallet-manager-backend get_weth_allowance_info '(null, "0x1234...")'
```

### ETH Wrapping/Unwrapping

```bash
# Wrap ETH to WETH (raw amount)
dfx canister call yieldex-ic-wallet-manager-backend wrap_eth_tokens '"1000000000000000000"'

# Wrap ETH to WETH (human-readable)
dfx canister call yieldex-ic-wallet-manager-backend wrap_eth_human_readable '"0.1"'

# Unwrap WETH to ETH (raw amount)
dfx canister call yieldex-ic-wallet-manager-backend unwrap_weth_tokens '"1000000000000000000"'

# Unwrap WETH to ETH (human-readable)
dfx canister call yieldex-ic-wallet-manager-backend unwrap_weth_human_readable '"0.1"'

# Get WETH balance for wrapping operations
dfx canister call yieldex-ic-wallet-manager-backend get_weth_balance_for_wrapping '(null)'
```

### Message Signing

```bash
# Sign arbitrary message
dfx canister call yieldex-ic-wallet-manager-backend sign_arbitrary_message '"Hello, World!"'

# Sign message and return with signer address
dfx canister call yieldex-ic-wallet-manager-backend sign_message_with_signer_address '"Hello, World!"'

# Sign 32-byte hash
dfx canister call yieldex-ic-wallet-manager-backend sign_32_byte_hash '"0x1234567890abcdef..."'
```

### Permissions Management

```bash
# Create permissions
dfx canister call yieldex-ic-wallet-manager-backend create_permissions '(record {
  whitelisted_protocols = vec { record { name = "AAVE"; address = "0x1234..." } };
  whitelisted_tokens = vec { record { name = "USDT"; address = "0x5678..." } };
  transfer_limits = vec { record { token_address = "0x5678..."; daily_limit = 1000; max_tx_amount = 100 } };
})'

# Get permissions by ID
dfx canister call yieldex-ic-wallet-manager-backend get_permissions '"permission-id"'

# Get all permissions for caller
dfx canister call yieldex-ic-wallet-manager-backend get_all_permissions

# Update permissions
dfx canister call yieldex-ic-wallet-manager-backend update_permissions '(record {
  permissions_id = "permission-id";
  whitelisted_protocols = opt vec { record { name = "AAVE"; address = "0x1234..." } };
  whitelisted_tokens = null;
  transfer_limits = null;
})'

# Delete permissions
dfx canister call yieldex-ic-wallet-manager-backend delete_permissions '"permission-id"'
```

## 🧪 Testing

### Run Rust Tests with PocketIC

All core functionality is tested using PocketIC:

```bash
cd tests
RUST_BACKTRACE=1 cargo test -- --nocapture
```

Tests cover:
- EVM address generation and retrieval
- User verification
- Permissions CRUD operations
- Access control and error scenarios
- ECDSA key mocking

## 🔧 Technical Architecture

### IC-Alloy Integration
- Uses ic-alloy library for Ethereum interactions via ICP
- Implements ICP threshold ECDSA for transaction signing (no private keys stored)
- Manual nonce management to optimize RPC calls
- ABI contract interactions using `sol!` macro

### Derivation Paths
- Creates unique EVM addresses per user using Principal IDs
- Uses threshold ECDSA with derivation paths for hierarchical deterministic wallet generation
- No private keys are stored; everything is derived from IC's threshold signatures

### Error Handling
- Comprehensive Result unwrapping and error propagation
- Balance checking before transactions
- Nonce caching to reduce RPC calls from 6 to 4 per transaction

### Security Features
- Threshold cryptography ensures no single point of failure
- Access control for permissions management
- Input validation and sanitization
- Slippage protection for DeFi operations

## 📊 Contract Addresses (Sepolia Testnet)

| Token | Address | Decimals |
|-------|---------|----------|
| LINK | `0x779877A7B0D9E8603169DdbD7836e478b4624789` | 18 |
| USDC | `0x1c7d4b196cb0c7b01d743fbc6116a902379c7238` | 6 |
| WETH | `0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9` | 18 |

## 🔗 Useful Links

- [Internet Computer Rust Development](https://internetcomputer.org/docs/current/developer-docs/backend/rust/)
- [ic-alloy Documentation](https://github.com/ic-alloy/ic-alloy)
- [PocketIC Testing](https://github.com/dfinity/ic/tree/master/packages/pocket-ic)
- [Threshold ECDSA](https://internetcomputer.org/docs/current/developer-docs/integrations/t-ecdsa/)
- [DFX SDK](https://internetcomputer.org/docs/current/developer-docs/setup/install)

## 📄 License

This project is part of the Yieldex protocol - an AI-driven, cross-chain yield optimization platform.
