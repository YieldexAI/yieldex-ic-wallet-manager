export interface WalletBalance {
  token: string;
  balance: number;
  balanceFormatted: string;
  value: number; // USD value
  network: string;
}

export interface Transaction {
  id: string;
  type: 'deposit' | 'withdraw' | 'earn' | 'rebalance';
  status: 'pending' | 'confirmed' | 'failed';
  amount: number;
  token: string;
  network: string;
  protocol?: string;
  strategy?: string;
  timestamp: string;
  txHash?: string;
  gasUsed?: number;
  gasPrice?: number;
}

export interface WalletState {
  isConnected: boolean;
  address: string;
  evmAddress: string;
  principal: string;
  networkId: number;
  balances: WalletBalance[];
  totalPortfolioValue: number;
  transactions: Transaction[];
}

// Mock wallet addresses
export const MOCK_ADDRESSES = {
  metamask: '0x742d35Cc6639C0532fEb5aEE70c28C83e4C5d50b',
  evmGenerated: '0x8Ba1f109551bD432803012645Hac136c8ce3681A',
  icPrincipal: 'rdmx6-jaaaa-aaaah-qcaiq-cai'
};

// Mock balances across different networks
export const MOCK_BALANCES: WalletBalance[] = [
  // Ethereum balances
  {
    token: 'USDC',
    balance: 12500.75,
    balanceFormatted: '12,500.75',
    value: 12500.75,
    network: 'ethereum'
  },
  {
    token: 'USDT',
    balance: 5000.00,
    balanceFormatted: '5,000.00',
    value: 5000.00,
    network: 'ethereum'
  },
  {
    token: 'DAI',
    balance: 2250.50,
    balanceFormatted: '2,250.50',
    value: 2250.50,
    network: 'ethereum'
  },
  
  // Arbitrum balances
  {
    token: 'USDC',
    balance: 8750.25,
    balanceFormatted: '8,750.25',
    value: 8750.25,
    network: 'arbitrum'
  },
  {
    token: 'USDT',
    balance: 3200.00,
    balanceFormatted: '3,200.00',
    value: 3200.00,
    network: 'arbitrum'
  },
  
  // Polygon balances
  {
    token: 'USDC',
    balance: 4500.80,
    balanceFormatted: '4,500.80',
    value: 4500.80,
    network: 'polygon'
  },
  {
    token: 'DAI',
    balance: 1800.25,
    balanceFormatted: '1,800.25',
    value: 1800.25,
    network: 'polygon'
  },
  
  // BSC balances
  {
    token: 'USDT',
    balance: 2100.00,
    balanceFormatted: '2,100.00',
    value: 2100.00,
    network: 'bsc'
  },
  {
    token: 'USDC',
    balance: 1500.50,
    balanceFormatted: '1,500.50',
    value: 1500.50,
    network: 'bsc'
  }
];

// Calculate total portfolio value
const TOTAL_PORTFOLIO_VALUE = MOCK_BALANCES.reduce((sum, balance) => sum + balance.value, 0);

// Mock transaction history
export const MOCK_TRANSACTIONS: Transaction[] = [
  {
    id: 'tx-1',
    type: 'deposit',
    status: 'confirmed',
    amount: 5000,
    token: 'USDC',
    network: 'ethereum',
    strategy: 'conservative-stable',
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 2).toISOString(), // 2 hours ago
    txHash: '0x1234567890abcdef1234567890abcdef12345678',
    gasUsed: 21000,
    gasPrice: 20
  },
  {
    id: 'tx-2',
    type: 'earn',
    status: 'confirmed',
    amount: 127.50,
    token: 'USDC',
    network: 'ethereum',
    protocol: 'aave-v3',
    strategy: 'conservative-stable',
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 24).toISOString(), // 1 day ago
    txHash: '0xabcdef1234567890abcdef1234567890abcdef12'
  },
  {
    id: 'tx-3',
    type: 'rebalance',
    status: 'confirmed',
    amount: 2500,
    token: 'USDT',
    network: 'arbitrum',
    strategy: 'moderate-growth',
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 48).toISOString(), // 2 days ago
    txHash: '0x567890abcdef1234567890abcdef1234567890ab'
  },
  {
    id: 'tx-4',
    type: 'deposit',
    status: 'pending',
    amount: 1000,
    token: 'DAI',
    network: 'polygon',
    strategy: 'moderate-growth',
    timestamp: new Date().toISOString(),
    gasUsed: 65000,
    gasPrice: 30
  }
];

// Initial wallet state
export const INITIAL_WALLET_STATE: WalletState = {
  isConnected: false,
  address: '',
  evmAddress: '',
  principal: '',
  networkId: 1, // Default to Ethereum
  balances: [],
  totalPortfolioValue: 0,
  transactions: []
};

// Mock connected wallet state
export const MOCK_CONNECTED_WALLET: WalletState = {
  isConnected: true,
  address: MOCK_ADDRESSES.metamask,
  evmAddress: MOCK_ADDRESSES.evmGenerated,
  principal: MOCK_ADDRESSES.icPrincipal,
  networkId: 1,
  balances: MOCK_BALANCES,
  totalPortfolioValue: TOTAL_PORTFOLIO_VALUE,
  transactions: MOCK_TRANSACTIONS
};

// Helper functions for wallet operations
export const getBalancesByNetwork = (networkSlug: string): WalletBalance[] => {
  return MOCK_BALANCES.filter(balance => balance.network === networkSlug);
};

export const getBalanceByToken = (token: string, network?: string): WalletBalance | undefined => {
  return MOCK_BALANCES.find(balance => 
    balance.token === token && (!network || balance.network === network)
  );
};

export const getTotalBalanceByToken = (token: string): number => {
  return MOCK_BALANCES
    .filter(balance => balance.token === token)
    .reduce((sum, balance) => sum + balance.value, 0);
};

export const getNetworkTVL = (networkSlug: string): number => {
  return getBalancesByNetwork(networkSlug)
    .reduce((sum, balance) => sum + balance.value, 0);
};

export const formatBalance = (balance: number, decimals: number = 2): string => {
  return new Intl.NumberFormat('en-US', {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals
  }).format(balance);
};

export const formatCurrency = (amount: number): string => {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  }).format(amount);
};

// Mock API functions for wallet operations
export const mockConnectWallet = async (): Promise<WalletState> => {
  // Simulate connection delay
  await new Promise(resolve => setTimeout(resolve, 2000));
  
  // Simulate WalletConnect/MetaMask connection
  return MOCK_CONNECTED_WALLET;
};

export const mockGenerateEvmAddress = async (): Promise<string> => {
  // Simulate IC threshold ECDSA address generation
  await new Promise(resolve => setTimeout(resolve, 3000));
  return MOCK_ADDRESSES.evmGenerated;
};

export const mockDeposit = async (
  amount: number,
  token: string,
  network: string,
  strategy: string
): Promise<Transaction> => {
  // Simulate transaction processing
  await new Promise(resolve => setTimeout(resolve, 2000));
  
  const transaction: Transaction = {
    id: `tx-${Date.now()}`,
    type: 'deposit',
    status: 'confirmed',
    amount,
    token,
    network,
    strategy,
    timestamp: new Date().toISOString(),
    txHash: `0x${Math.random().toString(16).substr(2, 40)}`,
    gasUsed: Math.floor(Math.random() * 50000 + 21000),
    gasPrice: Math.floor(Math.random() * 20 + 10)
  };
  
  return transaction;
};

export const mockWithdraw = async (
  amount: number,
  token: string,
  network: string,
  strategy: string
): Promise<Transaction> => {
  // Simulate transaction processing
  await new Promise(resolve => setTimeout(resolve, 2000));
  
  const transaction: Transaction = {
    id: `tx-${Date.now()}`,
    type: 'withdraw',
    status: 'confirmed',
    amount,
    token,
    network,
    strategy,
    timestamp: new Date().toISOString(),
    txHash: `0x${Math.random().toString(16).substr(2, 40)}`,
    gasUsed: Math.floor(Math.random() * 50000 + 21000),
    gasPrice: Math.floor(Math.random() * 20 + 10)
  };
  
  return transaction;
};