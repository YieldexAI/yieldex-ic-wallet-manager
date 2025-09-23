# 🌟 Yieldex IC Wallet Manager - Frontend

A production-ready React-based **B2B demonstration interface** for wallet providers and consumer apps. Showcases real-time stablecoin portfolio management, cross-chain DeFi yield strategies, and live balance tracking powered by Internet Computer's Threshold ECDSA technology.

**🌐 Live Demo**: [https://app.yieldex.xyz/](https://app.yieldex.xyz/) - Experience the full B2B integration capabilities

## 🚀 Quick Start

### Prerequisites
- Node.js 18+
- npm or yarn
- Running IC local replica (for backend integration)

### Installation

```bash
# Install dependencies
npm install

# Setup environment variables
cp .env.example .env
# Edit .env and configure your settings

# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

### Environment Configuration

```bash
# .env file configuration
VITE_WALLETCONNECT_PROJECT_ID=your_walletconnect_project_id
VITE_IC_CANISTER_ID=lla3p-uyaaa-aaaap-an2ea-cai
VITE_NETWORK_RPC_URLS=your_rpc_endpoints
VITE_DEMO_MODE=false  # Set to true for demo mode
```

### Wallet Integration

1. **WalletConnect Setup:**
   - Get Project ID from [WalletConnect Cloud](https://cloud.walletconnect.com/)
   - Add to your `.env` file

2. **Supported Wallets:**
   - MetaMask (browser extension)
   - WalletConnect compatible mobile wallets
   - Coinbase Wallet
   - Any injected EVM wallet

## 🎯 Production Features

### 🔐 Wallet Integration
- **Real Wallet Connection** - MetaMask, WalletConnect, and injected wallets
- **IC Threshold ECDSA** - Keyless EVM address generation
- **Multi-Network Support** - Ethereum Sepolia, Arbitrum One, Base, BSC

### 💰 Live Stablecoin Portfolio
- **Real Balance Tracking** - Fetches actual wallet balances from blockchain
- **Multi-Stablecoin Support**:
  - **USDC** - USD Coin on Arbitrum & Base networks
  - **USDT** - Tether USD on multiple networks
  - **USDe** - Ethena stablecoin integration
  - **DAI** - MakerDAO stablecoin support
- **Auto-Refresh** - Real-time balance updates every 30 seconds

### 📊 Real-Time Features
- **Live Portfolio Value** - Real USD valuations from price feeds
- **Network-Specific Balances** - Per-network stablecoin distributions
- **Yield Opportunities** - "Start Earn" buttons for DeFi protocol integration
- **Transaction History** - Complete audit trail of all operations

## 🛠️ Technology Stack

- **React 18** + **TypeScript** - Modern React with full type safety
- **Framer Motion** - Smooth animations and transitions
- **Tailwind CSS** - Utility-first styling with Web3 design system
- **Zustand** - Lightweight state management for wallet and portfolio state
- **Vite** - Fast build tool and dev server
- **Wagmi + Viem** - Type-safe Ethereum interaction libraries
- **TanStack Query** - Powerful data fetching and caching
- **React Hook Form** - Performant form handling
- **Alchemy SDK** - Enhanced blockchain data APIs

## 📁 Project Structure

```
src/
├── components/
│   ├── Dashboard/       # Real-time portfolio analytics
│   ├── Strategies/      # DeFi yield strategy management
│   ├── Stablecoins/     # Stablecoin portfolio components
│   ├── Wallet/          # Wallet connection & management
│   ├── Navigation/      # App navigation and routing
│   ├── Layout/          # Page layouts and structure
│   └── UI/             # Reusable UI components & design system
├── contexts/           # React contexts (Toast notifications)
├── stores/             # Zustand state stores
│   ├── walletStore.ts  # Wallet connection and address state
│   ├── transactionStore.ts # Transaction history and status
│   └── strategyStore.ts # DeFi strategy state management
├── types/              # TypeScript type definitions
├── utils/              # Helper functions and utilities
├── styles/             # Global styles and Tailwind config
└── mock/               # Development mock data
```

## 🎨 Design System

### Color Palette
- **Primary**: Blue gradient (`from-primary-600 to-primary-700`)
- **Success**: Green variants for positive metrics
- **Warning**: Yellow/Orange for moderate risk
- **Danger**: Red variants for high risk

### Components
- **Glass Cards** - Glassmorphism design with backdrop blur
- **Cyber Buttons** - Futuristic button styles with hover effects
- **Protocol Badges** - Color-coded risk indicators
- **Network Badges** - Multi-chain network identification

### Animations
- **Page Transitions** - Smooth entry/exit animations
- **Card Hover** - Subtle lift and shadow effects  
- **Loading States** - Shimmer and pulse animations
- **Real-Time Updates** - Smooth number transitions

## 🔧 Configuration

### Environment Variables
```env
# Production/Demo mode toggle
VITE_DEMO_MODE=false  # false for live blockchain data

# IC Canister configuration
VITE_IC_CANISTER_ID=lla3p-uyaaa-aaaap-an2ea-cai

# WalletConnect configuration
VITE_WALLETCONNECT_PROJECT_ID=your_project_id

# RPC endpoints for multi-chain support
VITE_ARBITRUM_RPC_URL=https://arb1.arbitrum.io/rpc
VITE_BASE_RPC_URL=https://mainnet.base.org
VITE_ETHEREUM_RPC_URL=https://eth-mainnet.alchemyapi.io/v2/your-key
```

### Customization
- **Networks**: Add new chains in `utils/networks.ts`
- **Tokens**: Configure stablecoins in `mock/walletData.ts`
- **Themes**: Modify `tailwind.config.js` for custom color schemes
- **Portfolio Layout**: Update components in `components/Stablecoins/`

## 🎮 User Flow

1. **Wallet Connection** - Connect MetaMask or WalletConnect wallet
2. **Address Generation** - Generate IC-derived EVM address
3. **Portfolio Loading** - Real-time balance fetching from blockchain
4. **Stablecoin Management** - View and manage multi-chain stablecoins
5. **Yield Strategies** - Access DeFi earning opportunities
6. **Transaction Tracking** - Monitor all wallet activities


## 🏢 B2B Integration Showcase

This frontend demonstrates how wallet providers and consumer apps can integrate Yieldex's DeFi capabilities:

### 🎯 **For Wallet Providers**
```typescript
// Example: Integrate yield opportunities in wallet UI
import { YieldexSDK } from '@yieldex/sdk';

const yieldex = new YieldexSDK({
  partnerId: 'your-wallet-id',
  canisterId: 'lla3p-uyaaa-aaaap-an2ea-cai'
});

// Get user's yield opportunities
const opportunities = await yieldex.getYieldOpportunities(userAddress);
```

### 📱 **For Consumer Apps**
- **White-label components** - Embed portfolio widgets directly
- **API integration** - RESTful endpoints for custom implementations
- **Real-time updates** - WebSocket connections for live data
- **Custom branding** - Fully customizable UI components

### 🔧 **Integration Benefits**
- 🚀 **Rapid deployment** - 2-week integration timeline
- 🛡️ **Enterprise security** - IC threshold cryptography
- 📊 **Analytics dashboard** - Partner performance metrics
- 💰 **Revenue sharing** - Competitive partnership terms

## 🤝 Contributing

This frontend demonstrates B2B integration capabilities for wallet providers and consumer apps.

---

**🌟 Built with ❤️ on Internet Computer**

*Empowering wallet providers and consumer apps with institutional-grade DeFi infrastructure*