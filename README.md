<div align="center">

# 🌟 Yieldex IC Wallet Manager

**The Next-Generation Cross-Chain DeFi Wallet Powered by Internet Computer**

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Internet Computer](https://img.shields.io/badge/Internet%20Computer-29ABE2?style=for-the-badge&logo=internetcomputer&logoColor=white)](https://internetcomputer.org/)
[![DeFi](https://img.shields.io/badge/DeFi-FF6B35?style=for-the-badge)](https://defillama.com/)
[![AAVE](https://img.shields.io/badge/AAVE-B6509E?style=for-the-badge&logo=aave&logoColor=white)](https://aave.com/)

*Revolutionizing DeFi with AI-powered cross-chain yield optimization through Internet Computer's ChainFusion technology*

[🚀 **Live Demo**](https://app.yieldex.xyz/) • [⚙️ **IC Canister**](https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=lla3p-uyaaa-aaaap-an2ea-cai) • [📖 **Documentation**](./CLAUDE.md)

</div>

---

## 🎯 **What is Yieldex?**

Yieldex IC Wallet Manager is a **full-stack DeFi platform** combining a Rust-based Internet Computer canister backend with a modern React frontend. Built on IC's revolutionary **ChainFusion** technology, it enables seamless cross-chain stablecoin portfolio management and yield optimization without bridges, private key management, or centralized intermediaries.

**🖥️ Live Demo**: [https://app.yieldex.xyz/](https://app.yieldex.xyz/) - Real-time multi-chain stablecoin portfolio
**⚙️ Backend Canister**: [IC Network](https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=lla3p-uyaaa-aaaap-an2ea-cai) - Production DeFi operations

### 🌈 **Why Yieldex?**

| Traditional DeFi            | 🆚   | Yieldex IC Wallet Manager                        |
| --------------------------- | ---- | ------------------------------------------------ |
| 🔐 Manage private keys      | ➡️ | 🛡️**Threshold ECDSA** (No keys to lose!) |
| 🌉 Trust bridges            | ➡️ | ⚡**Native ChainFusion**                   |
| 📊 Manual yield farming     | ➡️ | 🤖**AI-Powered Optimization**              |
| 💸 High gas fees            | ➡️ | 💰**Optimized Nonce Management**           |
| 🔒 Single-chain limitations | ➡️ | 🌐**True Cross-Chain Freedom**             |

---

## ✨ **Cutting-Edge Features**

### 🏦 **Smart Wallet Infrastructure**

- 🔑 **Zero-Knowledge Address Generation** - Unique EVM addresses via IC threshold ECDSA
- 👤 **Principal-Based Identity** - Seamless user verification and management
- 🛡️ **Advanced Permissions System** - Granular control over protocols and token operations
- 📈 **Real-Time Portfolio Tracking** - Monitor all your DeFi positions in one place


### 🏛️ **DeFi Protocol Integration**

- 🏪 **AAVE V3** - Supply, withdraw, and earn yield on Sepolia testnet ✅ **LIVE**
- 🏗️ **Compound III** - Supply and withdraw USDC on Arbitrum One mainnet ✅ **LIVE**
- 🔄 **Cross-Protocol Rebalancing** - Automated token migration between AAVE and Compound *(coming soon)*
- 🦄 **Uniswap V3** - Automated market making and token swaps *(coming soon)*

### 🔐 **Enterprise-Grade Security**

- 🎯 **Threshold Cryptography** - Distributed key management with no single point of failure
- 🛡️ **Access Control Matrix** - Role-based permissions with daily limits
- 📊 **Transaction Monitoring** - Real-time security checks and anomaly detection
- 🔍 **Audit Trail** - Complete transaction history and compliance tracking

---

## 🏗️ **Technical Architecture**

### 🌐 **Yieldex B2B Architecture for Wallets & Consumer Apps**

```
                                   ┌──────────┐                                        ┌──────────┐                          ┌─────────────┐
                                   │ Partner  │                                        │ Yieldex  │                          │EVM Networks │
                                   │ Wallet/  │                                        │ Canister │                          │Multi-Chain  │
       User via                    │Consumer  │                                        │(B2B API) │                          │DeFi Protocols│
       Partner App                 │   App    │                                        └────┬─────┘                          └──────┬──────┘
        │     EVM Wallet Login          ┌┴┐                                                 │                                       │
        │ ─────────────────────────────>│ │                                                 │                                       │
        │                               │ │────┐                                            │                                       │
        │                               │ │    │ Integrate Yieldex                          │                                       │
        │                               │ │    │ DeFi SDK/API                               │                                       │
        │                               │ │<───┘                                            │                                       │
        │                               │ │                                                 │                                       │
        │                               │ │         partner_generate_address()         ┌┴┐                                          │
        │                               │ │ ──────────────────────────────────────────>│ │                                          │
        │                               │ │                                            │ │────┐                                     │
        │                               │ │                                            │ │    │ B2B Principal Mapping               │
        │                               │ │                                            │ │    │ + IC Threshold ECDSA                │
        │                               │ │                                            │ │<───┘ Address Generation                  │
        │                               │ │                                            └┬┘                                          │
        │                               │ │          OK, partner_evm_address            │                                           │
        │                               │ │ <─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─                                              │
        │                               │ │                                             │                                           │
        │                               │ │      partner_get_portfolio_data()        ┌┴┐                                            │
        │                               │ │ ──────────────────────────────────────────>│ │                                          │
        │                               │ │                                            │ │       Multi-Chain RPC Calls             ┌┴┐
        │                               │ │                                            │ │ ───────────────────────────────────────>│ │
        │                               │ │                                            │ │                                         │ │
        │                               │ │                                            │ │         Real Balances + Yields          │ │
        │                               │ │                                            │ │ <─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│ │
        │                               │ │                                            └┬┘                                         └┬┘
        │                               │ │    OK, standardized_portfolio_data          │                                           │
        │                               │ │ <─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─                                              │
        │                               │ │                                             │                                           │
        │                               │ │   partner_execute_defi_strategy()        ┌┴┐                                            │
        │                               │ │ ──────────────────────────────────────────>│ │                                          │
        │                               │ │                                            │ │────┐                                     │
        │                               │ │                                            │ │    │ B2B Permissions +                   │
        │                               │ │                                            │ │    │ Strategy Validation                 │
        │                               │ │                                            │ │<───┘                                     │
        │                               │ │                                            │ │                                          │
        │                               │ │                                            │ │    Execute DeFi Operations              ┌┴┐
        │                               │ │                                            │ │ ───────────────────────────────────────>│ │
        │                               │ │                                            │ │                                         │ │
        │                               │ │                                            │ │    Multi-Protocol Execution             │ │
        │                               │ │                                            │ │    (AAVE, Compound, Uniswap)            │ │
        │                               │ │                                            │ │ ───────────────────────────────────────>│ │
        │                               │ │                                            │ │                                         │ │
        │                               │ │                                            │ │        Transaction Results              │ │
        │                               │ │                                            │ │ <─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│ │
        │                               │ │                                            └┬┘                                         └┬┘
        │                               │ │      Success, standardized_response         │                                           │
        │                               │ │ <─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─                                              │
        │                               │ │                                             │                                           │
        │                               │ │   partner_auto_rebalance_portfolio()     ┌┴┐                                            │
        │                               │ │ ──────────────────────────────────────────>│ │                                          │
        │                               │ │                                            │ │────┐                                     │
        │                               │ │                                            │ │    │ AI-Powered Cross-Protocol           │
        │                               │ │                                            │ │    │ Optimization Algorithm              │
        │                               │ │                                            │ │<───┘                                     │
        │                               │ │                                            │ │                                          │
        │                               │ │                                            │ │    Atomic Multi-Step Execution          ┌┴┐
        │                               │ │                                            │ │ ───────────────────────────────────────>│ │
        │                               │ │                                            │ │                                         │ │
        │                               │ │                                            │ │    Cross-Protocol Rebalancing           │ │
        │                               │ │                                            │ │    (Withdraw + Supply + Optimize)       │ │
        │                               │ │                                            │ │ ───────────────────────────────────────>│ │
        │                               │ │                                            │ │                                         │ │
        │                               │ │                                            │ │         Optimized Results               │ │
        │                               │ │                                            │ │ <─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│ │
        │                               │ │                                            └┬┘                                         └┬┘
        │                               │ │    Success, optimized_portfolio_state       │                                           │
        │                               │ │ <─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │                                           │
        │                               └┬┘                                             │                                           │ 
        │  Enhanced DeFi experience      │                                              │                                           │
        │  via Partner App integration   │                                              │                                           │
        │ <─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─                                                │                                           │
      User via                     ┌────┴─────┐                                   ┌─────┴────┐                          ┌───────────┴────┐
      Partner App                  │ Partner  │                                   │ Yieldex  │                          │EVM Networks    │
                                   │ Wallet/  │                                   │ Canister │                          │Multi-Chain     │
                                   │Consumer  │                                   │(B2B API) │                          │DeFi Protocols  │
                                   │   App    │                                   └──────────┘                          └────────────────┘
                                   └──────────┘
```

### 🔧 **B2B Component Overview**

**Partner Applications (Wallets & Consumer Apps)**
- 🔗 EVM wallet login integration
- 📱 Native DeFi features via Yieldex SDK
- 🎯 White-label yield strategy embedding
- 📊 Seamless portfolio integration in existing UI

**Yieldex IC Canister (B2B API)**
- 🔐 Partner-specific principal mapping
- 🌐 Standardized multi-chain API responses
- 🏦 Protocol-agnostic DeFi integrations
- 🛡️ B2B permissions & compliance framework
- 🤖 AI-powered portfolio optimization
- ⚡ Atomic cross-protocol transactions

**Multi-Chain DeFi Protocols**
- **Ethereum Sepolia**: AAVE V3, development testing
- **Arbitrum One**: Compound III, native USDC yields
- **Base**: Growing stablecoin ecosystem
- **Future**: Polygon, Optimism, and emerging L2s

### 🏢 **B2B Integration Benefits**

**For Wallet Providers:**
- 💰 Additional revenue streams through DeFi yields
- 🔒 Enhanced security via IC threshold cryptography
- 📈 Increased user engagement and retention
- 🛠️ Easy API integration, no complex DeFi infrastructure

**For Consumer Apps:**
- 🎯 Add DeFi features without regulatory complexity
- 🚀 Fast time-to-market with proven protocols
- 📊 Real-time portfolio analytics out-of-the-box
- 🔧 Customizable user experience and branding

<details>
<summary>📁 <strong>Project Structure</strong></summary>

```
yieldex-ic-wallet-manager/
├── 🚀 src/
│   ├── yieldex-ic-wallet-manager-backend/     # Rust IC Canister
│   │   ├── src/
│   │   │   ├── 🛠️ services/                  # Modular service architecture
│   │   │   │   ├── 🏦 aave.rs                # AAVE V3 integration
│   │   │   │   ├── 🏗️ compound.rs           # Compound III integration
│   │   │   │   ├── 🔄 rebalance.rs           # Cross-protocol rebalancing
│   │   │   │   ├── 💰 get_balance_*.rs       # Multi-token balance checking
│   │   │   │   ├── 📤 transfer_*.rs          # Token transfer operations
│   │   │   │   ├── ✅ approve_*.rs           # Token approval management
│   │   │   │   ├── 🔐 permissions.rs         # Access control system
│   │   │   │   └── 🌐 rpc_service.rs         # Multi-chain RPC management
│   │   │   ├── 📦 types/                     # Type definitions
│   │   │   │   ├── 📋 mod.rs                 # Type module exports
│   │   │   │   ├── 🔒 permissions.rs         # Permission structures
│   │   │   │   ├── 💡 recommendation.rs      # Rebalance recommendation types
│   │   │   │   └── 💾 storable.rs            # Stable storage wrappers
│   │   │   ├── 📄 abi/                       # Smart contract ABIs
│   │   │   └── 🧠 lib.rs                     # Core canister logic
│   │   └── ⚙️ Cargo.toml
│   └── yieldex-ic-wallet-manager-frontend/    # React Frontend
│       ├── src/
│       │   ├── 📊 components/                # UI components
│       │   │   ├── Dashboard/                # Portfolio analytics
│       │   │   ├── Stablecoins/              # Stablecoin portfolio
│       │   │   ├── Wallet/                   # Wallet connection
│       │   │   └── UI/                       # Design system
│       │   ├── 🔧 stores/                    # State management
│       │   ├── 🎯 types/                     # TypeScript definitions
│       │   └── 🛠️ utils/                     # Helper functions
│       ├── 📋 package.json                   # Dependencies
│       └── ⚙️ vite.config.ts                # Build configuration
├── 🧪 tests/                                 # Comprehensive test suite
├── 📚 CLAUDE.md                              # Development guide
└── 🔧 dfx.json                              # IC deployment config
```

</details>

---

## 🚀 **Quick Start Guide**

### 🛠️ **Prerequisites**

Ensure you have these tools installed:

```bash
# Install Rust (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install DFX SDK (latest version)
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"

# Install PocketIC for testing
# Follow: https://github.com/dfinity/ic/tree/master/packages/pocket-ic
```

### ⚡ **Full-Stack Setup**

```bash
# 1️⃣ Clone the repository
git clone https://github.com/your-org/yieldex-ic-wallet-manager.git
cd yieldex-ic-wallet-manager

# 2️⃣ Backend Setup (IC Canister)
dfx start --background          # Start IC local replica
dfx build                       # Build Rust canister
dfx deploy                      # Deploy canister locally

# 3️⃣ Frontend Setup (React App)
cd src/yieldex-ic-wallet-manager-frontend
npm install                     # Install dependencies
cp .env.example .env           # Configure environment
npm run dev                    # Start development server at http://localhost:3000

# 🎉 Full-stack DeFi platform ready!
```

### 🌐 **Production Deployment**

```bash
# Backend: Deploy to IC Mainnet
dfx deploy --network ic

# Frontend: Build for production
cd src/yieldex-ic-wallet-manager-frontend
npm run build
npm run preview                # Preview production build
```

### 📱 **Frontend Features**

The React frontend provides a modern interface for interacting with the IC canister:

- **🔗 Real Wallet Integration** - MetaMask, WalletConnect support
- **📊 Live Stablecoin Portfolio** - Real-time balance tracking across chains
- **🌐 Multi-Chain Support** - Arbitrum, Base, Ethereum networks
- **💰 Yield Opportunities** - Direct integration with DeFi protocols
- **📱 Responsive Design** - Mobile-first with glassmorphism UI
- **⚡ Fast Performance** - Vite build system with TypeScript

---

## 🎮 **Interactive API Playground**

### 🏦 **Wallet Management**

<details>
<summary>🔑 <strong>Generate Your Quantum-Secure EVM Address</strong></summary>

```bash
# ✨ Create your unique EVM address using IC threshold ECDSA
dfx canister call yieldex-ic-wallet-manager-backend generate_evm_address --ic   

# 📋 Retrieve your stored address
dfx canister call yieldex-ic-wallet-manager-backend get_evm_address --ic

# ✅ Verify user registration
dfx canister call yieldex-ic-wallet-manager-backend verify_user '(principal "rdmx6-jaaaa-aaaah-qcaiq-cai")' --ic
```

</details>

### 💰 **Portfolio Dashboard**

<details>
<summary>📊 <strong>Real-Time Balance Monitoring</strong></summary>

```bash
# 💎 Check your ETH balance
dfx canister call yieldex-ic-wallet-manager-backend get_eth_balance '(null)' --ic

# 🏦 Monitor USDC holdings
dfx canister call yieldex-ic-wallet-manager-backend get_usdc_balance '(null)' --ic

# 🔗 Track LINK positions
dfx canister call yieldex-ic-wallet-manager-backend get_link_balance '(null)' --ic

# 🌊 View WETH portfolio
dfx canister call yieldex-ic-wallet-manager-backend get_weth_token_balance '(null)' --ic
```

</details>

### 🏛️ **AAVE V3 Integration**

<details>
<summary>🏪 <strong>Supply & Earn Yield on AAVE</strong></summary>

```bash
# 📈 Supply LINK to AAVE and start earning yield
dfx canister call yieldex-ic-wallet-manager-backend supply_link_to_aave_secured '("0.1", "your-permissions-id")' --ic

# 💰 Withdraw your LINK + earned yield
dfx canister call yieldex-ic-wallet-manager-backend withdraw_link_from_aave_secured '("0.1", "your-permissions-id")' --ic

# 📊 Check your aLINK balance (AAVE interest-bearing tokens)
dfx canister call yieldex-ic-wallet-manager-backend get_aave_link_user_balance '(null)' --ic
```

**Real Example Output:**

```bash
🎉 Success! Transaction: 0x89816f5f8262ec38436c68f2c1dd87ab12506b56b14136c15c89ae4ab5551295
✅ Successfully withdrew 0.1 LINK from AAVE with earned yield!
```

</details>

### 🏗️ **Compound III Integration**

<details>
<summary>🏦 <strong>Supply & Earn on Compound (Arbitrum)</strong></summary>

```bash
# 💰 Supply USDC to Compound III and start earning yield
dfx canister call yieldex-ic-wallet-manager-backend supply_usdc_to_compound_secured '("1.0", "your-permissions-id")' --ic

# 💸 Withdraw your USDC + earned yield
dfx canister call yieldex-ic-wallet-manager-backend withdraw_usdc_from_compound_secured '("1.0", "your-permissions-id")' --ic

# 📊 Check your Compound USDC balance
dfx canister call yieldex-ic-wallet-manager-backend get_compound_usdc_user_balance '(null, 42161)' --ic

# 🌐 Check supported chains
dfx canister call yieldex-ic-wallet-manager-backend get_supported_chains --ic
```

**Real Example Output:**

```bash
🎉 Success! ✅ Successfully supplied 1.0 USDC to Compound! Transaction: 0xabc123...
💰 Now earning yield on Arbitrum One mainnet with native USDC!
```

</details>

### 🔄 **Cross-Protocol Rebalancing**

<details>
<summary>⚡ <strong>Automated Yield Optimization via Recommendation Engine</strong></summary>

The rebalancing system allows seamless migration of assets between DeFi protocols to optimize yield. The system uses a **recommendation-based approach** with structured validation and atomic execution.

#### **🎯 Recommendation Structure**

A recommendation defines the complete rebalancing operation:
- **Asset & Protocol Migration**: Move tokens between AAVE and Compound
- **Yield Optimization**: Target higher APY opportunities
- **Safety Validation**: Pre-execution checks for permissions and balances
- **Atomic Execution**: Either fully succeeds or safely reverts

#### **📋 Execute a Recommendation**

```bash
# 🔄 Rebalance 0.5 USDC from AAVE to Compound
dfx canister call yieldex-ic-wallet-manager-backend execute_recommendation '(
  record {
    asset = "USDC";
    to_asset = "USDC";
    from_chain = "Arbitrum";
    to_chain = null;
    from_protocol = "aave-v3";
    to_protocol = "compound-v3";
    current_apy = 3.5;
    target_apy = 4.2;
    estimated_profit = 0.0035;
    gas_cost = 0.50;
    position_size = "0.5";
    pool_id = null;
    recommendation_type = variant { StandardTransfer };
    swap_details = null;
  },
  "your-permissions-id"
)' --ic

# 🔄 Rebalance from Compound back to AAVE
dfx canister call yieldex-ic-wallet-manager-backend execute_recommendation '(
  record {
    asset = "USDC";
    to_asset = "USDC";
    from_chain = "Arbitrum";
    to_chain = null;
    from_protocol = "compound-v3";
    to_protocol = "aave-v3";
    current_apy = 4.2;
    target_apy = 3.5;
    estimated_profit = 0.0035;
    gas_cost = 0.50;
    position_size = "0.5";
    pool_id = null;
    recommendation_type = variant { StandardTransfer };
    swap_details = null;
  },
  "your-permissions-id"
)' --ic

# ✅ Validate a recommendation without executing
dfx canister call yieldex-ic-wallet-manager-backend validate_recommendation_input '(
  record {
    asset = "USDC";
    to_asset = "USDC";
    from_chain = "Arbitrum";
    to_chain = null;
    from_protocol = "aave-v3";
    to_protocol = "compound-v3";
    current_apy = 3.5;
    target_apy = 4.2;
    estimated_profit = 0.0035;
    gas_cost = 0.50;
    position_size = "0.5";
    pool_id = null;
    recommendation_type = variant { StandardTransfer };
    swap_details = null;
  }
)' --ic
```

#### **🔧 Recommendation Parameters**

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `asset` | String | Source token symbol | `"USDC"` |
| `to_asset` | String | Target token symbol | `"USDC"` |
| `from_chain` | String | Source blockchain | `"Arbitrum"` |
| `to_chain` | Option\<String\> | Target chain (null for same-chain) | `null` |
| `from_protocol` | String | Source protocol | `"aave-v3"` |
| `to_protocol` | String | Target protocol | `"compound-v3"` |
| `current_apy` | f64 | Current APY (%) | `3.5` |
| `target_apy` | f64 | Target APY (%) | `4.2` |
| `estimated_profit` | f64 | Expected profit (USD) | `0.0035` |
| `gas_cost` | f64 | Estimated gas (USD) | `0.50` |
| `position_size` | String | Amount to rebalance | `"0.5"` |
| `pool_id` | Option\<String\> | Specific pool ID | `null` |
| `recommendation_type` | Enum | Transfer type | `StandardTransfer` or `CrossChainTransfer` |
| `swap_details` | Option\<SwapDetails\> | Swap info (if needed) | `null` |

#### **🎯 Execution Flow**

1. **Validation Phase**:
   - ✅ Verify protocols differ or assets differ
   - ✅ Check position_size is positive
   - ✅ Validate protocol names
   - ✅ Confirm permissions exist and match owner
   - ✅ Ensure sufficient balance

2. **Withdrawal Phase**:
   - 📤 Withdraw tokens from source protocol (AAVE/Compound)
   - 🔍 Record transaction hash
   - ⚡ Update nonce cache

3. **Supply Phase**:
   - 📥 Supply tokens to target protocol
   - 🔓 Approve tokens if needed (auto-handled)
   - 🔍 Record transaction hash
   - ⚡ Update usage limits

4. **Result**:
   ```rust
   ExecutionResult {
     status: "success" | "partial" | "failed",
     withdraw_tx: Option<String>,  // Withdraw transaction hash
     swap_tx: Option<String>,       // Swap tx (if applicable)
     supply_tx: Option<String>,     // Supply transaction hash
     amount_transferred: Option<String>,
     actual_gas_cost: Option<f64>,
     error_details: Option<String>
   }
   ```

#### **🔒 Permission Requirements**

To execute rebalancing, your permissions must include:

```bash
# Both protocols must be whitelisted
whitelisted_protocols = vec {
  record { name = "AAVE V3"; address = "0x794a61358D6845594F94dc1DB02A252b5b4814aD" };
  record { name = "Compound V3"; address = "0x9c4ec768c28520b50860ea7a15bd7213a9ff58bf" };
}

# Both protocols need supply/withdraw permissions
protocol_permissions = vec {
  record {
    protocol_address = "0x794a61358D6845594F94dc1DB02A252b5b4814aD";
    allowed_functions = vec { "supply"; "withdraw" };
    max_amount_per_tx = opt (20_000_000 : nat64);  // 20 USDC
    daily_limit = opt (50_000_000 : nat64);        // 50 USDC
  };
  record {
    protocol_address = "0x9c4ec768c28520b50860ea7a15bd7213a9ff58bf";
    allowed_functions = vec { "supply"; "withdraw" };
    max_amount_per_tx = opt (20_000_000 : nat64);
    daily_limit = opt (50_000_000 : nat64);
  };
}
```

#### **📊 Real Example Output**

```bash
variant {
  Ok = record {
    status = "success";
    withdraw_tx = opt "0xdac5cb9c111a472a889c0d54fd5a118d16b87537651d1d219602f94769c2f8de";
    swap_tx = null;
    supply_tx = opt "0x89816f5f8262ec38436c68f2c1dd87ab12506b56b14136c15c89ae4ab5551295";
    amount_transferred = opt "0.5";
    actual_gas_cost = null;
    error_details = null;
  }
}

🎉 Rebalance completed successfully!
📤 Withdrawn from AAVE: 0xdac5cb9c...
📥 Supplied to Compound: 0x89816f5f...
💰 Optimizing yield on 0.5 USDC!
```

#### **⚠️ Important Notes**

- **Nonce Management**: System uses fresh nonces for approval transactions to prevent conflicts
- **Gas Optimization**: Transactions are batched when possible to reduce costs
- **Atomic Safety**: If supply fails after successful withdraw, funds remain in wallet (status: `partial`)
- **Cross-Chain**: Currently supports same-chain transfers only (`to_chain` must be null)
- **Supported Routes**: AAVE ↔ Compound on Arbitrum for USDC

</details>

### 🔐 **Advanced Permission System**

<details>
<summary>🛡️ <strong>Create Secure DeFi Permissions</strong></summary>

```bash
# 🏗️ Create sophisticated permission structure for AAVE (Sepolia)
dfx canister call yieldex-ic-wallet-manager-backend create_permissions '(record {
  chain_id = 11155111;
  whitelisted_protocols = vec {
    record { name = "AAVE"; address = "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951" }
  };
  whitelisted_tokens = vec {
    record { name = "LINK"; address = "0xf8fb3713d459d7c1018bd0a49d19b4c44290ebe5" }
  };
  transfer_limits = vec {
    record {
      token_address = "0xf8fb3713d459d7c1018bd0a49d19b4c44290ebe5";
      daily_limit = 1000000000000000000;
      max_tx_amount = 100000000000000000
    }
  };
  protocol_permissions = opt vec {
    record {
      protocol_address = "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951";
      allowed_functions = vec { "supply"; "withdraw" };
      max_amount_per_tx = opt 100000000000000000;
      daily_limit = opt 1000000000000000000;
      total_used_today = 0;
      last_reset_date = 0;
    }
  };
})' --ic

# 🏗️ Create permission structure for Compound (Arbitrum)
dfx canister call yieldex-ic-wallet-manager-backend create_permissions '(record {
  chain_id = 42161;
  whitelisted_protocols = vec {
    record { name = "Compound"; address = "0x9c4ec768c28520b50860ea7a15bd7213a9ff58bf" }
  };
  whitelisted_tokens = vec {
    record { name = "USDC"; address = "0xaf88d065e77c8cc2239327c5edb3a432268e5831" }
  };
  transfer_limits = vec {
    record {
      token_address = "0xaf88d065e77c8cc2239327c5edb3a432268e5831";
      daily_limit = 1000000000;
      max_tx_amount = 1000000
    }
  };
  protocol_permissions = opt vec {
    record {
      protocol_address = "0x9c4ec768c28520b50860ea7a15bd7213a9ff58bf";
      allowed_functions = vec { "supply"; "withdraw" };
      max_amount_per_tx = opt 1000000;
      daily_limit = opt 1000000000;
      total_used_today = 0;
      last_reset_date = 0;
    }
  };
})' --ic

# 📋 View your permission configuration
dfx canister call yieldex-ic-wallet-manager-backend get_all_permissions --ic
```

</details>

### 💸 **Lightning-Fast Transfers**

<details>
<summary>⚡ <strong>Multi-Token Transfer Operations</strong></summary>

```bash
# 🚀 Send ETH with human-readable amounts
dfx canister call yieldex-ic-wallet-manager-backend send_eth_human_readable '("0x742d35Cc6639C0532fEb5aEE70c28C83e4C5d50b", "0.001")' --ic

# 🔗 Transfer LINK tokens
dfx canister call yieldex-ic-wallet-manager-backend transfer_link_human_readable '("0x742d35Cc6639C0532fEb5aEE70c28C83e4C5d50b", "1.5")' --ic

# 🏦 Send USDC stablecoins
dfx canister call yieldex-ic-wallet-manager-backend approve_usdc_human_readable '("0x742d35Cc6639C0532fEb5aEE70c28C83e4C5d50b", "100.50")' --ic
```

</details>

---

## 🧪 **Comprehensive Testing Suite**

### 🚀 **Run All Tests**

```bash
# 🧪 Execute full PocketIC test suite
cd tests && RUST_BACKTRACE=1 cargo test -- --nocapture

# ✅ Test coverage includes:
# - EVM address generation & security
# - Multi-token operations
# - AAVE integration workflows
# - Permission system validation
# - Cross-chain transaction flows
```

### 📊 **Test Categories**

- 🔐 **Security Tests** - Threshold ECDSA, access control, permission validation
- 💰 **DeFi Integration** - AAVE supply/withdraw, yield calculations
- 🌐 **Cross-Chain** - ChainFusion operations, nonce management
- 🛡️ **Error Handling** - Edge cases, network failures, invalid inputs

---

## 🌐 **Supported Networks & Assets**

### 🏪 **Live Integrations**

| Network                    | Status     | Protocols               | Assets                |
| -------------------------- | ---------- | ----------------------- | --------------------- |
| **Ethereum Sepolia** | 🟢 Live    | AAVE V3                 | ETH, USDC, LINK, WETH |
| **Arbitrum One**     | 🟢 Live    | Compound III            | ETH, Native USDC      |
| **IC Mainnet**       | 🟢 Live    | Native                  | Cycles, ICP           |
| **Ethereum Mainnet** | 🟡 Coming  | AAVE, Uniswap, Compound | All ERC-20            |
| **Polygon**          | 🔵 Planned | AAVE, QuickSwap         | All ERC-20            |

### 💎 **Asset Registry**

| Token          | Network                | Contract Address                               | Decimals | DeFi Protocols |
| -------------- | ---------------------- | ---------------------------------------------- | -------- | -------------- |
| **LINK** | Ethereum Sepolia       | `0xf8fb3713d459d7c1018bd0a49d19b4c44290ebe5` | 18       | AAVE V3 ✅     |
| **USDC** | Ethereum Sepolia       | `0x1c7d4b196cb0c7b01d743fbc6116a902379c7238` | 6        | AAVE V3 🔄     |
| **USDC** | Arbitrum One           | `0xaf88d065e77c8cc2239327c5edb3a432268e5831` | 6        | Compound III ✅|
| **WETH** | Ethereum Sepolia       | `0x7b79995e5f793a07bc00c21412e50ecae098e7f9` | 18       | Uniswap V3 🔄  |

---

## 🎯 **Roadmap to DeFi Dominance**

### 🚀 **Phase 1: Foundation** *(Completed)*

- [X] ✅ IC Threshold ECDSA Integration
- [X] ✅ Multi-token Wallet Operations
- [X] ✅ AAVE V3 Supply/Withdraw
- [X] ✅ Compound III Integration (Arbitrum)
- [X] ✅ Multi-Chain Architecture (Sepolia + Arbitrum)
- [X] ✅ Cross-Protocol Rebalancing Engine
- [X] ✅ Chain-Specific Permission System
- [X] ✅ Comprehensive Testing Suite

### 🌟 **Phase 2: Expansion** *(Q2 2025)*

- [ ] 🦄 Uniswap V3 Integration
- [ ] 🤖 AI-Powered Yield Optimization
- [ ] 📊 Advanced Analytics Dashboard
- [ ] 🌐 Ethereum Mainnet Deployment
- [ ] 🔧 Advanced Rebalancing Strategies

### 🚀 **Phase 3: Revolution** *(Q3 2025)*

- [ ] 🌈 Extended Multi-Chain Support (Polygon, Base)
- [ ] 🤖 Autonomous Yield Strategies
- [ ] 📱 Mobile SDK
- [ ] 🏛️ Institutional Features
- [ ] 🔄 Cross-Chain Arbitrage

---

## 🛡️ **Security & Audits**

### 🔒 **Security Architecture**

- **🎯 Threshold ECDSA** - Distributed key generation with no single point of failure
- **🛡️ IC-Native Security** - Built on Internet Computer's battle-tested infrastructure
- **📊 Transaction Monitoring** - Real-time anomaly detection and fraud prevention
- **🔍 Access Control** - Granular permissions with time-based restrictions

### 🏆 **Audit Status**

| Component        | Status             | Auditor      | Report                                                                                     |
| ---------------- | ------------------ | ------------ | ------------------------------------------------------------------------------------------ |
| Core Canister    | 🟡 Pending         | TBD          | Q2 2025                                                                                    |
| AAVE Integration | 🟡 Internal Review | Yieldex Team | ✅ Complete                                                                                |
| Threshold ECDSA  | 🟢 IC Verified     | DFINITY      | [View Report](https://internetcomputer.org/docs/current/developer-docs/integrations/t-ecdsa/) |

### ⚠️ **Known Issues**

| Issue                          | Status         | Network  | Description                                       |
| ------------------------------ | -------------- | -------- | ------------------------------------------------- |
| **IC Consensus Timeout**      | 🔧 In Progress | Arbitrum | `HttpOutcallError: No consensus could be reached` |

**Current Focus:** Resolving Arbitrum transaction consensus issues where IC replicas return different responses during HTTP outcalls. This affects write operations (supply/withdraw) while read operations work normally. Investigation underway for RPC endpoint optimization and transaction reliability improvements.

---

## 💡 **Advanced Use Cases**

### 🏛️ **For Institutions**

- **Treasury Management** - Automated yield optimization for DAO treasuries
- **Risk Management** - Sophisticated position monitoring and rebalancing
- **Compliance** - Audit trails and regulatory reporting tools

### 👤 **For Retail Users**

- **Set & Forget** - Automated DeFi strategies with minimal maintenance
- **Security First** - No private keys, no seed phrases, no worries
- **Cross-Chain Freedom** - Access multiple chains from a single interface

### 🏗️ **For Developers**

- **SDK Integration** - Embed DeFi capabilities into any application
- **Custom Strategies** - Build sophisticated yield farming algorithms
- **Instant Deployment** - Launch on IC with global CDN distribution

---

### 🌟 **Join the DeFi Revolution**

[![https://x.com/YieldexAi](https://img.shields.io/badge/Twitter-1DA1F2?style=for-the-badge&logo=twitter&logoColor=white)](https://x.com/YieldexAi)
[![@apy_info_bot](https://img.shields.io/badge/Telegram-2CA5E0?style=for-the-badge&logo=telegram&logoColor=white)](https://t.me/apy_info_bot)
[![GitHub](https://img.shields.io/badge/GitHub-100000?style=for-the-badge&logo=github&logoColor=white)](https://github.com/YieldexAI)

**🚀 [Get Started Now](https://lla3p-uyaaa-aaaap-an2ea-cai.icp0.io) • 📖 [Read the Docs](https://yieldex.gitbook.io/yieldex-docs) • **

</div>

### 🏆 **Contributors**

Thanks to all the amazing developers making DeFi accessible to everyone!

<!-- Add contributor images here -->

---

## 📚 **Technical Resources**

- 🔗 [Internet Computer Rust Development](https://internetcomputer.org/docs/current/developer-docs/backend/rust/)
- ⚡ [ic-alloy Documentation](https://github.com/ic-alloy/ic-alloy)
- 🧪 [PocketIC Testing Framework](https://github.com/dfinity/ic/tree/master/packages/pocket-ic)
- 🔐 [Threshold ECDSA Guide](https://internetcomputer.org/docs/current/developer-docs/integrations/t-ecdsa/)
- 🛠️ [DFX SDK Documentation](https://internetcomputer.org/docs/current/developer-docs/setup/install)

---

## 📄 **License & Legal**

This project is part of the **Yieldex Protocol** - pioneering the future of AI-driven, cross-chain yield optimization.

**🚨 Important:** This software is provided for educational and research purposes. Always conduct your own research and consider the risks before using DeFi protocols.

---

<div align="center">

### 🌟 **Built with ❤️ on Internet Computer**

**Making DeFi accessible, secure, and profitable for everyone**

[⭐ Star this repo](https://github.com/yieldex/yieldex-ic-wallet-manager) • [🔄 Fork &amp; Contribute](https://github.com/yieldex/yieldex-ic-wallet-manager/fork) • [🐛 Report Issues](https://github.com/yieldex/yieldex-ic-wallet-manager/issues)

---

*© 2025 Yieldex Protocol. The future of DeFi is here.* 🚀

</div>
