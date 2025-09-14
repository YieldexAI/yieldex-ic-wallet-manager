# 🌟 Yieldex IC Wallet Manager - Demo Frontend

A modern React-based demo interface for the Yieldex IC Wallet Manager, showcasing cross-chain DeFi yield strategies powered by Internet Computer's Threshold ECDSA technology.

## 🚀 Quick Start

### Prerequisites
- Node.js 18+
- npm or yarn

### Installation

```bash
# Install dependencies
npm install

# Setup environment variables
cp .env.example .env
# Edit .env and add your WalletConnect Project ID

# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

### WalletConnect Setup

1. **Get Project ID:**
   - Go to [WalletConnect Cloud](https://cloud.walletconnect.com/)
   - Create a free account and new project
   - Copy your Project ID

2. **Configure Environment:**
   ```bash
   # In your .env file
   VITE_WALLETCONNECT_PROJECT_ID=your_actual_project_id_here
   ```

3. **Supported Wallets:**
   - MetaMask (browser extension)
   - WalletConnect compatible mobile wallets
   - Coinbase Wallet
   - Any injected EVM wallet

## 🎯 Demo Features

### 🔐 Wallet Integration
- **Mock WalletConnect** - Simulated wallet connection experience
- **IC Threshold ECDSA** - EVM address generation demo
- **Multi-Network Support** - Ethereum, Arbitrum, Polygon, BSC

### 💰 DeFi Strategies (Coming Soon)
- **Conservative Strategy** (5-6% APY): AAVE V3, Compound III, Venus
- **Moderate Strategy** (7-9% APY): Enhanced protocols with higher yields  
- **Aggressive Strategy** (15-25% APY): Morpho, Euler, cutting-edge protocols

### 📊 Real-Time Features
- **Live Balance Simulation** - Watch your investments grow
- **Cross-Protocol Rebalancing** - Automated yield optimization
- **Multi-Chain Portfolio** - Aggregate view across networks

## 🛠️ Technology Stack

- **React 18** + **TypeScript** - Modern React with full type safety
- **Framer Motion** - Smooth animations and transitions
- **Tailwind CSS** - Utility-first styling with Web3 design system
- **Zustand** - Lightweight state management
- **Vite** - Fast build tool and dev server

## 📁 Project Structure

```
src/
├── components/
│   ├── Dashboard/       # Portfolio and analytics
│   ├── Strategies/      # Yield strategy selection
│   ├── Wallet/          # Wallet connection & management
│   └── UI/             # Reusable UI components
├── hooks/              # Custom React hooks
├── mock/               # Demo data and API simulation
├── stores/             # Zustand state stores
├── styles/             # Global styles and themes
└── utils/              # Helper functions and utilities
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
# Demo mode (no real transactions)
VITE_DEMO_MODE=true

# API endpoints (mocked)
VITE_IC_CANISTER_ID=lla3p-uyaaa-aaaap-an2ea-cai
VITE_NETWORK_RPC_URLS=...
```

### Customization
- **Themes**: Modify `tailwind.config.js` for custom color schemes
- **Animations**: Adjust timing in `utils/animations.ts`
- **Mock Data**: Update `mock/` files for different demo scenarios

## 🎮 Demo Flow

1. **Landing** - Connect wallet simulation
2. **Network Selection** - Choose EVM network  
3. **Address Generation** - IC Threshold ECDSA demo
4. **Strategy Selection** - Choose yield strategy (coming soon)
5. **Portfolio Management** - Real-time balance tracking (coming soon)

## 🔒 Security Notes

- **Demo Only** - No real transactions or private keys
- **Simulated Data** - All balances and yields are mocked
- **Safe Testing** - Perfect for demonstrations and screenshots

## 🤝 Contributing

This is a demo interface for showcase purposes. For the actual protocol implementation, see the main Rust canister code.

## 📄 License

Part of the Yieldex Protocol ecosystem. See main project for license details.

---

**🌟 Built with ❤️ on Internet Computer**

*Making DeFi accessible, secure, and profitable for everyone*