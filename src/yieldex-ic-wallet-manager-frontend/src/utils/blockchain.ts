import { NetworkType } from '@/types/transactions';

// Network configurations
export const NETWORKS = {
  ethereum: {
    id: 11155111, // Sepolia testnet
    name: 'Ethereum Sepolia',
    shortName: 'Sepolia',
    slug: 'ethereum',
    logo: '/networks/ethereum.svg',
    explorerUrl: 'https://sepolia.etherscan.io',
    rpcUrl: 'https://rpc.sepolia.org',
    nativeCurrency: {
      name: 'Sepolia Ether',
      symbol: 'SEP',
      decimals: 18
    }
  },
  arbitrum: {
    id: 42161, // Arbitrum One mainnet
    name: 'Arbitrum One',
    shortName: 'Arbitrum',
    slug: 'arbitrum',
    logo: '/networks/arbitrum.svg',
    explorerUrl: 'https://arbiscan.io',
    rpcUrl: 'https://arb1.arbitrum.io/rpc',
    nativeCurrency: {
      name: 'Ether',
      symbol: 'ETH',
      decimals: 18
    }
  },
  icp: {
    id: 0, // ICP doesn't use standard chain IDs
    name: 'Internet Computer',
    shortName: 'ICP',
    slug: 'icp',
    logo: '/networks/icp.svg',
    explorerUrl: 'https://dashboard.internetcomputer.org',
    rpcUrl: '', // Not applicable for ICP
    nativeCurrency: {
      name: 'ICP',
      symbol: 'ICP',
      decimals: 8
    }
  }
} as const;

// Explorer URL generators
export const getExplorerUrl = (network: NetworkType, hash: string, type: 'tx' | 'address' | 'block' = 'tx'): string => {
  const networkConfig = NETWORKS[network];

  switch (network) {
    case 'ethereum':
    case 'arbitrum':
      switch (type) {
        case 'tx':
          return `${networkConfig.explorerUrl}/tx/${hash}`;
        case 'address':
          return `${networkConfig.explorerUrl}/address/${hash}`;
        case 'block':
          return `${networkConfig.explorerUrl}/block/${hash}`;
        default:
          return `${networkConfig.explorerUrl}/tx/${hash}`;
      }

    case 'icp':
      if (type === 'tx') {
        return `${networkConfig.explorerUrl}/transaction/${hash}`;
      } else if (type === 'address') {
        return `${networkConfig.explorerUrl}/canister/${hash}`;
      }
      return `${networkConfig.explorerUrl}/transaction/${hash}`;

    default:
      return '';
  }
};

// Get network by chain ID
export const getNetworkByChainId = (chainId: number): NetworkType | null => {
  for (const [key, network] of Object.entries(NETWORKS)) {
    if (network.id === chainId) {
      return key as NetworkType;
    }
  }
  return null;
};

// Check if transaction hash is valid for network
export const isValidTransactionHash = (network: NetworkType, hash: string): boolean => {
  switch (network) {
    case 'ethereum':
    case 'arbitrum':
      // Ethereum transaction hashes are 66 characters (0x + 64 hex chars)
      return /^0x[a-fA-F0-9]{64}$/.test(hash);

    case 'icp':
      // ICP transaction hashes can vary in format
      return hash.length > 0;

    default:
      return false;
  }
};

// Format transaction hash for display
export const formatTransactionHash = (hash: string, length: number = 8): string => {
  if (hash.length <= length * 2) {
    return hash;
  }

  return `${hash.slice(0, length)}...${hash.slice(-length)}`;
};

// Get protocol information
export const PROTOCOLS = {
  AAVE: {
    name: 'AAVE V3',
    network: 'ethereum' as NetworkType,
    logo: '/protocols/aave.svg',
    website: 'https://aave.com',
    description: 'Decentralized lending protocol'
  },
  COMPOUND: {
    name: 'Compound V3',
    network: 'arbitrum' as NetworkType,
    logo: '/protocols/compound.svg',
    website: 'https://compound.finance',
    description: 'Algorithmic money markets'
  }
} as const;

// Get gas price estimation (mock data for demo)
export const getGasPriceEstimate = async (network: NetworkType): Promise<number> => {
  // In a real implementation, this would fetch from network APIs
  switch (network) {
    case 'ethereum':
      return 20; // gwei
    case 'arbitrum':
      return 0.1; // gwei (much lower on L2)
    default:
      return 0;
  }
};

// Calculate transaction fee
export const calculateTransactionFee = (gasUsed: number, gasPrice: number): number => {
  return (gasUsed * gasPrice) / 1e9; // Convert from gwei to ETH
};

// Network status checker
export const checkNetworkStatus = async (network: NetworkType): Promise<'online' | 'degraded' | 'offline'> => {
  // Mock implementation - in real app would ping network endpoints
  return 'online';
};

// Transaction status tracker
export type TransactionStatus = 'pending' | 'confirmed' | 'failed';

export const getTransactionStatus = async (
  network: NetworkType,
  hash: string
): Promise<TransactionStatus> => {
  // Mock implementation - in real app would query blockchain
  return 'confirmed';
};

// Add to wallet functionality
export const addNetworkToWallet = async (network: NetworkType): Promise<boolean> => {
  if (typeof window === 'undefined' || !window.ethereum) {
    return false;
  }

  const networkConfig = NETWORKS[network];

  try {
    await window.ethereum.request({
      method: 'wallet_addEthereumChain',
      params: [{
        chainId: `0x${networkConfig.id.toString(16)}`,
        chainName: networkConfig.name,
        nativeCurrency: networkConfig.nativeCurrency,
        rpcUrls: [networkConfig.rpcUrl],
        blockExplorerUrls: [networkConfig.explorerUrl]
      }]
    });
    return true;
  } catch (error) {
    console.error('Failed to add network to wallet:', error);
    return false;
  }
};

// Switch to network
export const switchToNetwork = async (network: NetworkType): Promise<boolean> => {
  if (typeof window === 'undefined' || !window.ethereum) {
    return false;
  }

  const networkConfig = NETWORKS[network];

  try {
    await window.ethereum.request({
      method: 'wallet_switchEthereumChain',
      params: [{ chainId: `0x${networkConfig.id.toString(16)}` }]
    });
    return true;
  } catch (error) {
    console.error('Failed to switch network:', error);
    return false;
  }
};