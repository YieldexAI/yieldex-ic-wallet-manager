import { PROTOCOLS } from './protocols';

export interface Strategy {
  id: string;
  name: string;
  description: string;
  risk: 'conservative' | 'moderate' | 'aggressive';
  expectedApy: number;
  minDeposit: number;
  maxDeposit: number;
  protocols: string[]; // Protocol IDs
  supportedTokens: string[];
  isActive: boolean;
  totalDeposited: number;
  performanceHistory: PerformanceDataPoint[];
  features: string[];
}

export interface PerformanceDataPoint {
  date: string;
  apy: number;
  tvl: number;
  earnings: number;
}

export interface UserPosition {
  id: string;
  strategyId: string;
  amount: number;
  token: string;
  entryDate: string;
  currentValue: number;
  totalEarnings: number;
  apy: number;
  isActive: boolean;
}

// Generate historical performance data
const generatePerformanceHistory = (baseApy: number, days: number = 30): PerformanceDataPoint[] => {
  const history: PerformanceDataPoint[] = [];
  const now = new Date();
  
  for (let i = days; i >= 0; i--) {
    const date = new Date(now.getTime() - i * 24 * 60 * 60 * 1000);
    const variance = (Math.random() - 0.5) * 0.02; // ±1% variance
    const apy = Math.max(0, baseApy + variance);
    
    history.push({
      date: date.toISOString().split('T')[0],
      apy: parseFloat(apy.toFixed(2)),
      tvl: Math.floor(Math.random() * 10000000 + 5000000),
      earnings: Math.floor(Math.random() * 50000 + 10000)
    });
  }
  
  return history;
};

// Yield Strategies
export const STRATEGIES: Strategy[] = [
  {
    id: 'single-aave',
    name: 'Single AAVE',
    description: 'Low-risk strategy focused on battle-tested AAVE V3 protocol with consistent returns',
    risk: 'conservative',
    expectedApy: 5.32, // AAVE V3 APY
    minDeposit: 100,
    maxDeposit: 1000000,
    protocols: ['aave-v3'],
    supportedTokens: ['USDC', 'USDT', 'DAI'],
    isActive: true,
    totalDeposited: 25000000,
    performanceHistory: generatePerformanceHistory(5.32),
    features: [
      'Battle-tested protocols',
      'Consistent returns',
      'Low volatility',
      'Automatic rebalancing',
      'Multi-chain diversification'
    ]
  },
  {
    id: 'moderate-growth',
    name: 'Moderate Growth',
    description: 'Balanced approach with higher yields through emerging protocols',
    risk: 'moderate',
    expectedApy: 8.20, // Average of Venus Boosted (7.82) + Spark (8.55) + Radiant (8.22)
    minDeposit: 500,
    maxDeposit: 500000,
    protocols: ['venus-boosted', 'spark-protocol', 'radiant-capital'],
    supportedTokens: ['USDC', 'USDT', 'DAI'],
    isActive: true,
    totalDeposited: 12000000,
    performanceHistory: generatePerformanceHistory(8.20),
    features: [
      'Enhanced yield opportunities',
      'Cross-chain optimization',
      'Active rebalancing',
      'MakerDAO backing (Spark)',
      'Moderate risk profile'
    ]
  },
  {
    id: 'aggressive-max',
    name: 'Aggressive Max Yield',
    description: 'High-risk, high-reward strategy targeting maximum returns',
    risk: 'aggressive',
    expectedApy: 22.17, // Average of Morpho (18.50) + Euler (22.80) + Fluid (25.20)
    minDeposit: 1000,
    maxDeposit: 100000,
    protocols: ['morpho-blue', 'euler-protocol', 'fluid-protocol'],
    supportedTokens: ['USDC', 'USDT', 'DAI'],
    isActive: true,
    totalDeposited: 5500000,
    performanceHistory: generatePerformanceHistory(22.17),
    features: [
      'Maximum yield potential',
      'Peer-to-peer optimization',
      'Advanced risk management',
      'Dynamic rebalancing',
      'Early protocol access'
    ]
  }
];

// Mock user positions for demo
export const MOCK_USER_POSITIONS: UserPosition[] = [
  {
    id: 'pos-1',
    strategyId: 'single-aave',
    amount: 5000,
    token: 'USDC',
    entryDate: '2024-01-15',
    currentValue: 5463.62,
    totalEarnings: 463.62,
    apy: 5.32,
    isActive: true
  },
  {
    id: 'pos-2',
    strategyId: 'moderate-growth',
    amount: 2500,
    token: 'USDT',
    entryDate: '2024-02-01',
    currentValue: 2843.16,
    totalEarnings: 343.16,
    apy: 8.20,
    isActive: true
  },
  {
    id: 'pos-3',
    strategyId: 'single-aave',
    amount: 10300,
    token: 'USDT',
    entryDate: '2024-01-10',
    currentValue: 10301.15,
    totalEarnings: 1.15,
    apy: 5.32,
    isActive: true
  },
  {
    id: 'pos-4',
    strategyId: 'aggressive-defi',
    amount: 27252.30,
    token: 'USDC',
    entryDate: '2024-03-01',
    currentValue: 27264.56,
    totalEarnings: 12.26,
    apy: 22.17,
    isActive: true
  },
  {
    id: 'pos-5',
    strategyId: 'moderate-growth',
    amount: 27252.30,
    token: 'USDC',
    entryDate: '2024-03-15',
    currentValue: 27256.54,
    totalEarnings: 4.24,
    apy: 8.20,
    isActive: true
  },
  {
    id: 'pos-6',
    strategyId: 'moderate-growth',
    amount: 27252.30,
    token: 'USDC',
    entryDate: '2024-04-01',
    currentValue: 27252.30,
    totalEarnings: 0,
    apy: 8.20,
    isActive: true
  }
];

// Real-time balance simulation
export const simulateBalanceGrowth = (
  initialAmount: number,
  apy: number,
  startDate: Date = new Date()
): number => {
  const now = new Date();
  const timeDiffHours = (now.getTime() - startDate.getTime()) / (1000 * 60 * 60);
  const hourlyRate = apy / 100 / (365 * 24);
  return initialAmount * Math.pow(1 + hourlyRate, timeDiffHours);
};

// Helper functions
export const getStrategyById = (id: string): Strategy | undefined => {
  return STRATEGIES.find(s => s.id === id);
};

export const getStrategiesByRisk = (risk: 'conservative' | 'moderate' | 'aggressive'): Strategy[] => {
  return STRATEGIES.filter(s => s.risk === risk && s.isActive);
};

export const calculateStrategyTVL = (strategy: Strategy): number => {
  return strategy.protocols.reduce((total, protocolId) => {
    const protocol = PROTOCOLS.find(p => p.id === protocolId);
    if (!protocol) return total;

    return total + protocol.tvl;
  }, 0);
};

export const getProtocolsForStrategy = (strategyId: string) => {
  const strategy = getStrategyById(strategyId);
  if (!strategy) return [];

  return strategy.protocols.map(protocolId => {
    const protocol = PROTOCOLS.find(p => p.id === protocolId);
    return protocol;
  }).filter(Boolean);
};