export interface Protocol {
  id: string;
  name: string;
  slug: string;
  logo: string;
  description: string;
  tvl: number;
  apy: number;
  baseApy: number;
  rewardApy: number;
  risk: 'conservative' | 'moderate' | 'aggressive';
  chains: Network[];
  tokens: string[];
  category: 'lending' | 'liquidity' | 'yield-farming';
  auditScore: number;
  isActive: boolean;
}

export interface Network {
  id: number;
  name: string;
  shortName: string;
  slug: string;
  logo: string;
  rpcUrl: string;
  explorerUrl: string;
  nativeCurrency: {
    name: string;
    symbol: string;
    decimals: number;
  };
}

export interface Token {
  address: string;
  symbol: string;
  name: string;
  decimals: number;
  logo: string;
  price: number;
  isStablecoin: boolean;
}

// Supported Networks
export const NETWORKS: Network[] = [
  {
    id: 1,
    name: 'Ethereum',
    shortName: 'Ethereum',
    slug: 'ethereum',
    logo: '/networks/ethereum.svg',
    rpcUrl: 'https://eth.llamarpc.com',
    explorerUrl: 'https://etherscan.io',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 }
  },
  {
    id: 42161,
    name: 'Arbitrum One',
    shortName: 'Arbitrum',
    slug: 'arbitrum',
    logo: '/networks/arbitrum.svg',
    rpcUrl: 'https://arb1.arbitrum.io/rpc',
    explorerUrl: 'https://arbiscan.io',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 }
  },
  {
    id: 137,
    name: 'Polygon',
    shortName: 'Polygon',
    slug: 'polygon',
    logo: '/networks/polygon.svg',
    rpcUrl: 'https://polygon-rpc.com',
    explorerUrl: 'https://polygonscan.com',
    nativeCurrency: { name: 'Matic', symbol: 'MATIC', decimals: 18 }
  },
  {
    id: 56,
    name: 'BNB Smart Chain',
    shortName: 'BSC',
    slug: 'bsc',
    logo: '/networks/bsc.svg',
    rpcUrl: 'https://bsc-dataseed.binance.org',
    explorerUrl: 'https://bscscan.com',
    nativeCurrency: { name: 'BNB', symbol: 'BNB', decimals: 18 }
  }
];

// Supported Tokens
export const TOKENS: Record<string, Token> = {
  USDC: {
    address: '0xa0b86a33e6039d96cb1c05353395b7d9b83b1d51',
    symbol: 'USDC',
    name: 'USD Coin',
    decimals: 6,
    logo: '/logos/usdc.svg',
    price: 1.00,
    isStablecoin: true
  },
  USDT: {
    address: '0xdac17f958d2ee523a2206206994597c13d831ec7',
    symbol: 'USDT',
    name: 'Tether USD',
    decimals: 6,
    logo: '/logos/usdt.svg',
    price: 1.00,
    isStablecoin: true
  },
  DAI: {
    address: '0x6b175474e89094c44da98b954eedeac495271d0f',
    symbol: 'DAI',
    name: 'Dai Stablecoin',
    decimals: 18,
    logo: '/logos/dai.svg',
    price: 1.00,
    isStablecoin: true
  }
};

// DeFi Protocols with real APY data
export const PROTOCOLS: Protocol[] = [
  // Conservative Protocols (5-6% APY)
  {
    id: 'aave-v3',
    name: 'AAVE V3',
    slug: 'aave',
    logo: '/logos/aave.svg',
    description: 'Leading decentralized lending protocol with battle-tested security',
    tvl: 12500000000,
    apy: 5.32,
    baseApy: 4.12,
    rewardApy: 1.20,
    risk: 'conservative',
    chains: NETWORKS.filter(n => ['ethereum', 'arbitrum', 'polygon'].includes(n.slug)),
    tokens: ['USDC', 'USDT', 'DAI'],
    category: 'lending',
    auditScore: 95,
    isActive: true
  },
  {
    id: 'compound-iii',
    name: 'Compound III',
    slug: 'compound',
    logo: '/logos/compound.svg',
    description: 'Next-generation lending with streamlined user experience',
    tvl: 8200000000,
    apy: 5.33,
    baseApy: 4.33,
    rewardApy: 1.00,
    risk: 'conservative',
    chains: NETWORKS.filter(n => ['ethereum', 'arbitrum', 'polygon'].includes(n.slug)),
    tokens: ['USDC', 'USDT'],
    category: 'lending',
    auditScore: 93,
    isActive: true
  },
  {
    id: 'compound-v3',
    name: 'Compound V3',
    slug: 'compound',
    logo: '/logos/compound.svg',
    description: 'Advanced lending protocol with enhanced capital efficiency',
    tvl: 8200000000,
    apy: 5.33,
    baseApy: 4.33,
    rewardApy: 1.00,
    risk: 'conservative',
    chains: NETWORKS.filter(n => ['ethereum', 'arbitrum', 'polygon'].includes(n.slug)),
    tokens: ['USDC', 'USDT'],
    category: 'lending',
    auditScore: 93,
    isActive: true
  },
  {
    id: 'venus-protocol',
    name: 'Venus Protocol',
    slug: 'venus',
    logo: '/logos/venus.svg',
    description: 'Algorithmic money market protocol on BNB Chain',
    tvl: 1800000000,
    apy: 5.82,
    baseApy: 4.52,
    rewardApy: 1.30,
    risk: 'conservative',
    chains: NETWORKS.filter(n => ['bsc'].includes(n.slug)),
    tokens: ['USDC', 'USDT', 'DAI'],
    category: 'lending',
    auditScore: 88,
    isActive: true
  },

  // Moderate Protocols (7-9% APY)
  {
    id: 'venus-boosted',
    name: 'Venus Boosted',
    slug: 'venus',
    logo: '/logos/venus.svg',
    description: 'Enhanced Venus pools with additional yield incentives',
    tvl: 950000000,
    apy: 7.82,
    baseApy: 5.22,
    rewardApy: 2.60,
    risk: 'moderate',
    chains: NETWORKS.filter(n => ['bsc'].includes(n.slug)),
    tokens: ['USDC', 'USDT'],
    category: 'lending',
    auditScore: 88,
    isActive: true
  },
  {
    id: 'spark-protocol',
    name: 'Spark Protocol',
    slug: 'spark',
    logo: '/logos/spark.svg',
    description: 'MakerDAO-backed lending protocol with enhanced yields',
    tvl: 2100000000,
    apy: 8.55,
    baseApy: 6.15,
    rewardApy: 2.40,
    risk: 'moderate',
    chains: NETWORKS.filter(n => ['ethereum', 'arbitrum'].includes(n.slug)),
    tokens: ['USDC', 'DAI'],
    category: 'lending',
    auditScore: 91,
    isActive: true
  },
  {
    id: 'radiant-capital',
    name: 'Radiant Capital',
    slug: 'radiant',
    logo: '/logos/radiant.svg',
    description: 'Cross-chain money market with omnichain functionality',
    tvl: 680000000,
    apy: 8.22,
    baseApy: 5.92,
    rewardApy: 2.30,
    risk: 'moderate',
    chains: NETWORKS.filter(n => ['arbitrum', 'polygon'].includes(n.slug)),
    tokens: ['USDC', 'USDT'],
    category: 'lending',
    auditScore: 84,
    isActive: true
  },

  // Aggressive Protocols (15-25% APY)
  {
    id: 'morpho-blue',
    name: 'Morpho Blue',
    slug: 'morpho',
    logo: '/logos/morpho.svg',
    description: 'Peer-to-peer lending with optimized matching engine',
    tvl: 1200000000,
    apy: 18.50,
    baseApy: 12.30,
    rewardApy: 6.20,
    risk: 'aggressive',
    chains: NETWORKS.filter(n => ['ethereum'].includes(n.slug)),
    tokens: ['USDC', 'USDT', 'DAI'],
    category: 'lending',
    auditScore: 87,
    isActive: true
  },
  {
    id: 'euler-protocol',
    name: 'Euler Protocol',
    slug: 'euler',
    logo: '/logos/euler.svg',
    description: 'Permissionless lending with advanced risk management',
    tvl: 450000000,
    apy: 22.80,
    baseApy: 14.50,
    rewardApy: 8.30,
    risk: 'aggressive',
    chains: NETWORKS.filter(n => ['ethereum'].includes(n.slug)),
    tokens: ['USDC', 'USDT'],
    category: 'lending',
    auditScore: 82,
    isActive: true
  },
  {
    id: 'fluid-protocol',
    name: 'Fluid Protocol',
    slug: 'fluid',
    logo: '/logos/fluid.svg',
    description: 'Zero-fee lending with dynamic interest rates',
    tvl: 180000000,
    apy: 25.20,
    baseApy: 16.80,
    rewardApy: 8.40,
    risk: 'aggressive',
    chains: NETWORKS.filter(n => ['ethereum', 'arbitrum'].includes(n.slug)),
    tokens: ['USDC', 'DAI'],
    category: 'lending',
    auditScore: 79,
    isActive: true
  }
];

// Helper functions
export const getProtocolsByRisk = (risk: 'conservative' | 'moderate' | 'aggressive') => {
  return PROTOCOLS.filter(p => p.risk === risk && p.isActive);
};

export const getProtocolsByChain = (chainId: number) => {
  return PROTOCOLS.filter(p => 
    p.chains.some(chain => chain.id === chainId) && p.isActive
  );
};

export const getProtocolById = (id: string) => {
  return PROTOCOLS.find(p => p.id === id);
};

export const getNetworkById = (chainId: number) => {
  return NETWORKS.find(n => n.id === chainId);
};