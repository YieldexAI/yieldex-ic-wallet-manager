import { StablecoinConfig, NetworkConfig } from './types';

// Network configurations for Alchemy
export const NETWORKS: Record<string, NetworkConfig> = {
  ethereum: {
    id: 1,
    name: 'Ethereum',
    slug: 'ethereum',
    alchemyNetwork: 'eth-mainnet',
    rpcUrl: `https://eth-mainnet.g.alchemy.com/v2/${import.meta.env.VITE_ALCHEMY_API_KEY}`
  },
  arbitrum: {
    id: 42161,
    name: 'Arbitrum One',
    slug: 'arbitrum',
    alchemyNetwork: 'arb-mainnet',
    rpcUrl: `https://arb-mainnet.g.alchemy.com/v2/${import.meta.env.VITE_ALCHEMY_API_KEY}`
  },
  polygon: {
    id: 137,
    name: 'Polygon',
    slug: 'polygon',
    alchemyNetwork: 'polygon-mainnet',
    rpcUrl: `https://polygon-mainnet.g.alchemy.com/v2/${import.meta.env.VITE_ALCHEMY_API_KEY}`
  },
  base: {
    id: 8453,
    name: 'Base',
    slug: 'base',
    alchemyNetwork: 'base-mainnet',
    rpcUrl: `https://base-mainnet.g.alchemy.com/v2/${import.meta.env.VITE_ALCHEMY_API_KEY}`
  }
};

// Stablecoin contract addresses across networks
export const STABLECOINS: Record<string, StablecoinConfig> = {
  USDT: {
    symbol: 'USDT',
    name: 'Tether USD',
    decimals: 6,
    logo: '/usdt.svg',
    contracts: {
      ethereum: '0xdAC17F958D2ee523a2206206994597C13D831ec7',
      arbitrum: '0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9',
      polygon: '0xc2132D05D31c914a87C6611C10748AEb04B58e8F',
      base: '0xfde4C96c8593536E31F229EA8f37b2ADa2699bb2'
    }
  },
  USDC: {
    symbol: 'USDC',
    name: 'USD Coin',
    decimals: 6,
    logo: '/usdc.svg',
    contracts: {
      ethereum: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
      arbitrum: '0xaf88d065e77c8cC2239327C5EDb3A432268e5831',
      polygon: '0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174',
      base: '0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913'
    }
  },
  USDe: {
    symbol: 'USDe',
    name: 'Ethena USDe',
    decimals: 18,
    logo: '/ethena-usde-usde-logo.svg',
    contracts: {
      ethereum: '0x4c9EDD5852cd905f086C759E8383e09bff1E68B3',
      arbitrum: '0x5d3a1Ff2b6BAb83b63cd9AD0787074081a52ef34'
      // USDe might not be available on all networks
    }
  },
  DAI: {
    symbol: 'DAI',
    name: 'Dai Stablecoin',
    decimals: 18,
    logo: '/dai.svg',
    contracts: {
      ethereum: '0x6B175474E89094C44Da98b954EedeAC495271d0F',
      arbitrum: '0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1',
      polygon: '0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063',
      base: '0x50c5725949A6F0c72E6C4a641F24049A917DB0Cb'
    }
  }
};

// Get all stablecoin contracts for a specific network
export const getStablecoinContractsForNetwork = (networkSlug: string): string[] => {
  const contracts: string[] = [];

  Object.values(STABLECOINS).forEach(stablecoin => {
    if (stablecoin.contracts[networkSlug]) {
      contracts.push(stablecoin.contracts[networkSlug]);
    }
  });

  return contracts;
};

// Get stablecoin config by contract address
export const getStablecoinByContract = (contractAddress: string, networkSlug: string): StablecoinConfig | undefined => {
  return Object.values(STABLECOINS).find(stablecoin =>
    stablecoin.contracts[networkSlug]?.toLowerCase() === contractAddress.toLowerCase()
  );
};

// Default networks to fetch balances from
export const DEFAULT_NETWORKS = ['ethereum', 'arbitrum', 'polygon', 'base'];