import { PROTOCOLS } from './protocols';

export interface Strategy {
  id: string;
  name: string;
  description: string;
  risk: 'conservative' | 'moderate' | 'aggressive';
  expectedApy: number;
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
    name: 'OG Duo',
    description: 'Low-risk strategy combining AAVE V3 + Compound V3 protocols for diversified, stable returns',
    risk: 'conservative',
    expectedApy: 5.32, // AAVE V3 APY
    protocols: ['aave-v3', 'compound-v3'],
    supportedTokens: ['USDC', 'USDT', 'DAI'],
    isActive: true,
    totalDeposited: 25000000,
    performanceHistory: generatePerformanceHistory(5.32),
    features: [
      'AAVE V3 + Compound V3',
      'Dual protocol diversification',
      'Low volatility',
      'Automatic rebalancing',
      'Multi-chain coverage'
    ]
  },
  {
    id: 'moderate-growth',
    name: 'Moderate Growth',
    description: 'Balanced approach with higher yields through emerging protocols',
    risk: 'moderate',
    expectedApy: 8.20, // Average of Venus Boosted (7.82) + Spark (8.55) + Radiant (8.22)
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

// Mock user positions for demo - AAVE focused since transactions are from AAVE
export const MOCK_USER_POSITIONS: UserPosition[] = [
  {
    id: 'pos-1',
    strategyId: 'single-aave',
    amount: 4.74,
    token: 'DAI',
    entryDate: '2024-01-15',
    currentValue: 4.74,
    totalEarnings: 0.00,
    apy: 5.32,
    isActive: true
  },
  {
    id: 'pos-2',
    strategyId: 'single-aave',
    amount: 4.74,
    token: 'DAI',
    entryDate: '2024-02-01',
    currentValue: 4.74,
    totalEarnings: 0.00,
    apy: 5.32,
    isActive: true
  },
  {
    id: 'pos-3',
    strategyId: 'single-aave',
    amount: 31.49,
    token: 'USDC',
    entryDate: '2024-01-10',
    currentValue: 31.50,
    totalEarnings: 0.01,
    apy: 8.20,
    isActive: true
  },
  {
    id: 'pos-4',
    strategyId: 'single-aave',
    amount: 36.49,
    token: 'USDC',
    entryDate: '2024-03-01',
    currentValue: 36.49,
    totalEarnings: 0.00,
    apy: 5.32,
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