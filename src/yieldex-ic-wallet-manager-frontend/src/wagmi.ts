import { createConfig, http } from 'wagmi';
import { mainnet, arbitrum, polygon, bsc } from 'wagmi/chains';
import { injected, coinbaseWallet, walletConnect } from 'wagmi/connectors';

// WalletConnect Project ID from environment variables  
const walletConnectProjectId = import.meta.env.VITE_WALLETCONNECT_PROJECT_ID || '35e5c985ae085dffa05f73486ba83a41';

// Configure chains for our DeFi app
export const chains = [mainnet, arbitrum, polygon, bsc] as const;

export const wagmiConfig = createConfig({
  chains,
  connectors: [
    // Generic injected wallets (will detect MetaMask, Trust Wallet, etc automatically)
    injected(),

    // WalletConnect v2 for mobile wallets
    walletConnect({
      projectId: walletConnectProjectId,
      metadata: {
        name: 'Your stable way to earn',
        description: 'Next-Generation Cross-Chain DeFi Wallet powered by Internet Computer',
        url: 'https://yieldex.ai',
        icons: ['https://yieldex.ai/icon.png'],
      },
    }),

    // Coinbase Wallet
    coinbaseWallet({
      appName: 'Your stable way to earn',
      appLogoUrl: 'https://yieldex.ai/icon.png',
    }),
  ],
  transports: {
    [mainnet.id]: http(),
    [arbitrum.id]: http(),
    [polygon.id]: http(),
    [bsc.id]: http(),
  },
});