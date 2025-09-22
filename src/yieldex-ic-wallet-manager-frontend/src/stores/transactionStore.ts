import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';
import {
  Transaction,
  TransactionGroup,
  TransactionType,
  TransactionStatus,
  ActivityFilter,
  NetworkType,
  RebalanceDetails
} from '@/types/transactions';

interface TransactionStore {
  // State
  transactions: Transaction[];
  transactionGroups: TransactionGroup[];
  isLoading: boolean;
  error: string | null;

  // Filters
  activeFilter: ActivityFilter;

  // Actions
  addTransaction: (transaction: Omit<Transaction, 'id' | 'timestamp'>) => void;
  addTransactionGroup: (group: Omit<TransactionGroup, 'id' | 'timestamp'>) => void;
  updateTransactionStatus: (id: string, status: TransactionStatus) => void;

  // Filters
  setFilter: (filter: Partial<ActivityFilter>) => void;
  clearFilter: () => void;

  // Getters
  getTransactionsByPosition: (positionId: string) => Transaction[];
  getActivityByPosition: (positionId: string) => (Transaction | TransactionGroup)[];
  getRecentActivity: (limit?: number) => (Transaction | TransactionGroup)[];
  getFilteredTransactions: () => Transaction[];
  getTransactionsByType: (type: TransactionType) => Transaction[];

  // Utilities
  generateMockActivity: (positionId: string) => void;
  initializeWithDefaultActivity: (positionId?: string) => void;
  getExplorerUrl: (network: NetworkType, txHash: string) => string;
  clearError: () => void;
}

const DEFAULT_FILTER: ActivityFilter = {
  types: [],
  status: [],
  dateRange: {
    from: '',
    to: ''
  },
  positionIds: []
};

const EXPLORER_URLS = {
  ethereum: 'https://sepolia.etherscan.io/tx/',
  arbitrum: 'https://arbiscan.io/tx/',
  icp: 'https://dashboard.internetcomputer.org/transaction/'
};

export const useTransactionStore = create<TransactionStore>()(
  devtools(
    persist(
      (set, get) => ({
        // Initial state
        transactions: [],
        transactionGroups: [],
        isLoading: false,
        error: null,
        activeFilter: DEFAULT_FILTER,

        // Actions
        addTransaction: (transactionData) => {
          const transaction: Transaction = {
            ...transactionData,
            id: `tx-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
            timestamp: new Date().toISOString(),
          };

          set((state) => ({
            transactions: [transaction, ...state.transactions]
          }));
        },

        addTransactionGroup: (groupData) => {
          const group: TransactionGroup = {
            ...groupData,
            id: `group-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
            timestamp: new Date().toISOString(),
          };

          set((state) => ({
            transactionGroups: [group, ...state.transactionGroups]
          }));
        },

        updateTransactionStatus: (id, status) => {
          set((state) => ({
            transactions: state.transactions.map(tx =>
              tx.id === id ? { ...tx, status } : tx
            ),
            transactionGroups: state.transactionGroups.map(group => ({
              ...group,
              transactions: group.transactions.map(tx =>
                tx.id === id ? { ...tx, status } : tx
              ),
              status: group.transactions.every(tx => tx.id === id || tx.status === 'completed')
                ? 'completed'
                : group.status
            }))
          }));
        },

        setFilter: (filterUpdates) => {
          set((state) => ({
            activeFilter: { ...state.activeFilter, ...filterUpdates }
          }));
        },

        clearFilter: () => {
          set({ activeFilter: DEFAULT_FILTER });
        },

        // Getters
        getTransactionsByPosition: (positionId) => {
          const { transactions } = get();
          return transactions.filter(tx => tx.positionId === positionId);
        },

        getActivityByPosition: (positionId) => {
          const { transactions, transactionGroups } = get();

          // Get individual transactions for this position
          const positionTransactions = transactions.filter(tx => tx.positionId === positionId);

          // Get transaction groups that contain transactions for this position
          const positionGroups = transactionGroups.filter(group =>
            group.transactions.some(tx => tx.positionId === positionId)
          );

          // Combine and sort by timestamp
          const allActivity = [
            ...positionTransactions,
            ...positionGroups
          ].sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());

          return allActivity;
        },

        getRecentActivity: (limit = 10) => {
          const { transactions, transactionGroups } = get();

          // Combine and sort by timestamp
          const allActivity = [
            ...transactions,
            ...transactionGroups
          ].sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());

          return allActivity.slice(0, limit);
        },

        getFilteredTransactions: () => {
          const { transactions, activeFilter } = get();

          if (!activeFilter.types.length && !activeFilter.status.length &&
              !activeFilter.positionIds.length && !activeFilter.dateRange.from) {
            return transactions;
          }

          return transactions.filter(tx => {
            // Filter by type
            if (activeFilter.types.length && !activeFilter.types.includes(tx.type)) {
              return false;
            }

            // Filter by status
            if (activeFilter.status.length && !activeFilter.status.includes(tx.status)) {
              return false;
            }

            // Filter by position
            if (activeFilter.positionIds.length && tx.positionId &&
                !activeFilter.positionIds.includes(tx.positionId)) {
              return false;
            }

            // Filter by date range
            if (activeFilter.dateRange.from && activeFilter.dateRange.to) {
              const txDate = new Date(tx.timestamp);
              const fromDate = new Date(activeFilter.dateRange.from);
              const toDate = new Date(activeFilter.dateRange.to);

              if (txDate < fromDate || txDate > toDate) {
                return false;
              }
            }

            return true;
          });
        },

        getTransactionsByType: (type) => {
          const { transactions } = get();
          return transactions.filter(tx => tx.type === type);
        },

        // Utilities
        generateMockActivity: (positionId) => {
          const { addTransaction, addTransactionGroup } = get();

          // Only generate rebalance for specific positions to show variety
          const shouldGenerateRebalance = positionId && (
            positionId.includes('position-2') ||
            positionId.includes('position-3') ||
            positionId.includes('usdc') ||
            true // Force for all positions for demo
          );

          // Create some mock transactions for testing
          const now = Date.now();

          if (shouldGenerateRebalance) {
            // Enhanced rebalancing group with AI decision chain
            const rebalanceGroup: Omit<TransactionGroup, 'id' | 'timestamp'> = {
              type: 'rebalancing_sequence',
              description: 'AI-Powered Rebalance: Compound → AAVE',
              status: 'completed',
              totalGasCost: 0.0025,
              groupIcon: '🤖',
              rebalanceDetails: {
                fromProtocol: 'Compound III',
                toProtocol: 'AAVE V3',
                reason: 'Higher yield opportunity detected with better risk profile',
                oldApy: 4.2,
                newApy: 6.1,
                amount: 4.50,
                token: 'USDC',
                potentialSavings: 0.095, // (6.1 - 4.2) * 4.50 / 100 * 365/365
                actualSavings: 0.091,
                confidenceScore: 0.87
              },
              transactions: [
                {
                  id: `tx-${now}-1`,
                  positionId,
                  type: 'ai_decision',
                  status: 'completed',
                  timestamp: new Date(now - 18 * 60 * 1000).toISOString(),
                  description: 'AI analyzed market conditions',
                  icon: '🤖',
                  color: 'text-blue-400',
                  blockchainRefs: [],
                  rebalanceDetails: {
                    fromProtocol: 'Compound III',
                    toProtocol: 'AAVE V3',
                    reason: 'Market analysis showed 1.9% APY improvement with lower risk',
                    oldApy: 4.2,
                    newApy: 6.1,
                    amount: 4.50,
                    token: 'USDC',
                    confidenceScore: 0.87
                  }
                },
                {
                  id: `tx-${now}-2`,
                  positionId,
                  type: 'withdrawal',
                  status: 'completed',
                  timestamp: new Date(now - 12 * 60 * 1000).toISOString(),
                  amount: 4.50,
                  token: 'USDC',
                  description: 'Withdrew from Compound III',
                  icon: '📤',
                  color: 'text-orange-400',
                  blockchainRefs: [{
                    network: 'arbitrum',
                    txHash: '0x1234567890abcdef1234567890abcdef12345678',
                    explorerUrl: 'https://arbiscan.io/tx/0x1234567890abcdef1234567890abcdef12345678'
                  }]
                },
                {
                  id: `tx-${now}-3`,
                  positionId,
                  type: 'deposit',
                  status: 'completed',
                  timestamp: new Date(now - 8 * 60 * 1000).toISOString(),
                  amount: 4.50,
                  token: 'USDC',
                  description: 'Deposited to AAVE V3',
                  icon: '📥',
                  color: 'text-green-400',
                  blockchainRefs: [{
                    network: 'ethereum',
                    txHash: '0xabcdef1234567890abcdef1234567890abcdef12',
                    explorerUrl: 'https://sepolia.etherscan.io/tx/0xabcdef1234567890abcdef1234567890abcdef12'
                  }]
                }
              ]
            };

            console.log('Adding rebalance group for position:', positionId);
            addTransactionGroup(rebalanceGroup);
          }

          // Recent yield collection
          addTransaction({
            positionId,
            type: 'yield_collection',
            status: 'completed',
            amount: 0.02,
            token: 'DAI',
            description: 'Automatic yield collection from AAVE V3',
            icon: '💰',
            color: 'text-green-400',
            blockchainRefs: [{
              network: 'ethereum',
              txHash: '0xef1234567890abcdef1234567890abcdef123456',
              explorerUrl: 'https://sepolia.etherscan.io/tx/0xef1234567890abcdef1234567890abcdef123456'
            }]
          });

          // Initial deposit
          addTransaction({
            positionId,
            type: 'deposit',
            status: 'completed',
            amount: 4.74,
            token: 'DAI',
            description: 'Initial deposit to AAVE V3 Strategy',
            icon: '💰',
            color: 'text-green-400',
            blockchainRefs: [{
              network: 'ethereum',
              txHash: '0x567890abcdef1234567890abcdef1234567890ab',
              explorerUrl: 'https://sepolia.etherscan.io/tx/0x567890abcdef1234567890abcdef1234567890ab'
            }]
          });

          // Smart wallet creation
          addTransaction({
            positionId,
            type: 'smart_wallet_creation',
            status: 'completed',
            description: 'Smart wallet created via IC threshold ECDSA',
            icon: '🏦',
            color: 'text-purple-400',
            blockchainRefs: [{
              network: 'icp',
              explorerUrl: 'https://dashboard.internetcomputer.org/canister/rdmx6-jaaaa-aaaah-qdrqq-cai'
            }]
          });
        },

        initializeWithDefaultActivity: (positionId?: string) => {
          const { transactions, transactionGroups, addTransaction, addTransactionGroup } = get();

          // For position-specific initialization, check if position has transactions
          if (positionId) {
            const positionTransactions = transactions.filter(tx => tx.positionId === positionId);
            const positionGroups = transactionGroups.filter(group =>
              group.transactions.some(tx => tx.positionId === positionId)
            );

            if (positionTransactions.length === 0 && positionGroups.length === 0) {
              // Add default transactions for this position
              const now = Date.now();

              // 1. Add one rebalance transaction group
              const rebalanceGroup = {
                type: 'rebalancing_sequence' as const,
                description: 'AI-Powered Rebalance: Compound → AAVE',
                status: 'completed' as const,
                totalGasCost: 0.0025,
                groupIcon: '🤖',
                rebalanceDetails: {
                  fromProtocol: 'Compound III',
                  toProtocol: 'AAVE V3',
                  reason: 'Higher yield opportunity detected with better risk profile',
                  oldApy: 4.2,
                  newApy: 6.1,
                  amount: 4.50,
                  token: 'USDC',
                  potentialSavings: 0.095,
                  actualSavings: 0.091,
                  confidenceScore: 0.87
                },
                transactions: [
                  {
                    id: `tx-${now}-1`,
                    positionId,
                    type: 'ai_decision' as const,
                    status: 'completed' as const,
                    timestamp: new Date(now - 20 * 60 * 1000).toISOString(),
                    description: 'AI analyzed market conditions',
                    icon: '🤖',
                    color: 'text-blue-400',
                    blockchainRefs: []
                  },
                  {
                    id: `tx-${now}-2`,
                    positionId,
                    type: 'withdrawal' as const,
                    status: 'completed' as const,
                    timestamp: new Date(now - 18 * 60 * 1000).toISOString(),
                    amount: 4.50,
                    token: 'USDC',
                    description: 'Withdrew from Compound III',
                    icon: '📤',
                    color: 'text-orange-400',
                    blockchainRefs: [{
                      network: 'arbitrum' as const,
                      txHash: '0x1234567890abcdef1234567890abcdef12345678',
                      explorerUrl: 'https://arbiscan.io/tx/0x1234567890abcdef1234567890abcdef12345678'
                    }]
                  },
                  {
                    id: `tx-${now}-3`,
                    positionId,
                    type: 'deposit' as const,
                    status: 'completed' as const,
                    timestamp: new Date(now - 15 * 60 * 1000).toISOString(),
                    amount: 4.50,
                    token: 'USDC',
                    description: 'Deposited to AAVE V3',
                    icon: '📥',
                    color: 'text-green-400',
                    blockchainRefs: [{
                      network: 'ethereum' as const,
                      txHash: '0xabcdef1234567890abcdef1234567890abcdef12',
                      explorerUrl: 'https://sepolia.etherscan.io/tx/0xabcdef1234567890abcdef1234567890abcdef12'
                    }]
                  }
                ]
              };

              addTransactionGroup(rebalanceGroup);

              // 2. Withdrawal transaction
              addTransaction({
                positionId,
                type: 'withdrawal',
                status: 'completed',
                amount: 2.5,
                token: 'USDC',
                description: 'Withdrew funds from Compound V3',
                icon: '📤',
                color: 'text-orange-400',
                blockchainRefs: [{
                  network: 'arbitrum',
                  txHash: '0xeff881d08d16eafe4dad9a86b1ee3b0fec19eff1d59f5707dc821e44b15f702a',
                  explorerUrl: 'https://arbiscan.io/tx/0xeff881d08d16eafe4dad9a86b1ee3b0fec19eff1d59f5707dc821e44b15f702a'
                }]
              });

              // 3. Supply transaction (earlier than withdrawal)
              addTransaction({
                positionId,
                type: 'deposit',
                status: 'completed',
                amount: 5.0,
                token: 'USDC',
                description: 'Supplied funds to Compound V3',
                icon: '📥',
                color: 'text-green-400',
                blockchainRefs: [{
                  network: 'arbitrum',
                  txHash: '0x951a0b18ebc1a918f22c6a8defbf26f4fba9b01941852e9a9613e709c1385653',
                  explorerUrl: 'https://arbiscan.io/tx/0x951a0b18ebc1a918f22c6a8defbf26f4fba9b01941852e9a9613e709c1385653'
                }]
              });

              // 4. Smart-wallet creation (earliest transaction)
              addTransaction({
                positionId,
                type: 'smart_wallet_creation',
                status: 'completed',
                description: 'Smart-wallet generated via IC threshold ECDSA',
                icon: '🏦',
                color: 'text-purple-400',
                blockchainRefs: [{
                  network: 'arbitrum',
                  explorerUrl: 'https://arbiscan.io/address/0x01e9ec708d2ccf81f2f0d5cc9a4f3321cd287145'
                }]
              });
            }
          } else {
            // Global initialization - only add default transactions if store is empty
            if (transactions.length === 0) {
              const now = Date.now();

              // 1. Withdrawal transaction
              addTransaction({
                type: 'withdrawal',
                status: 'completed',
                amount: 2.5,
                token: 'USDC',
                description: 'Withdrew funds from AAVE V3',
                icon: '📤',
                color: 'text-orange-400',
                blockchainRefs: [{
                  network: 'arbitrum',
                  txHash: '0xeff881d08d16eafe4dad9a86b1ee3b0fec19eff1d59f5707dc821e44b15f702a',
                  explorerUrl: 'https://arbiscan.io/tx/0xeff881d08d16eafe4dad9a86b1ee3b0fec19eff1d59f5707dc821e44b15f702a'
                }]
              });

              // 2. Supply transaction (earlier than withdrawal)
              addTransaction({
                type: 'deposit',
                status: 'completed',
                amount: 5.0,
                token: 'USDC',
                description: 'Supplied funds to AAVE V3',
                icon: '📥',
                color: 'text-green-400',
                blockchainRefs: [{
                  network: 'arbitrum',
                  txHash: '0x951a0b18ebc1a918f22c6a8defbf26f4fba9b01941852e9a9613e709c1385653',
                  explorerUrl: 'https://arbiscan.io/tx/0x951a0b18ebc1a918f22c6a8defbf26f4fba9b01941852e9a9613e709c1385653'
                }]
              });

              // 3. Smart-wallet creation (earliest transaction)
              addTransaction({
                type: 'smart_wallet_creation',
                status: 'completed',
                description: 'Smart-wallet generated via IC threshold ECDSA',
                icon: '🏦',
                color: 'text-purple-400',
                blockchainRefs: [{
                  network: 'arbitrum',
                  explorerUrl: 'https://arbiscan.io/address/0x01e9ec708d2ccf81f2f0d5cc9a4f3321cd287145'
                }]
              });
            }
          }
        },

        getExplorerUrl: (network, txHash) => {
          return `${EXPLORER_URLS[network]}${txHash}`;
        },

        clearError: () => {
          set({ error: null });
        }
      }),
      {
        name: 'transaction-store',
        partialize: (state) => ({
          transactions: state.transactions,
          transactionGroups: state.transactionGroups
        })
      }
    ),
    { name: 'transaction-store' }
  )
);

// Convenience hooks
export const usePositionActivity = (positionId: string) => {
  const getTransactionsByPosition = useTransactionStore(state => state.getTransactionsByPosition);
  const generateMockActivity = useTransactionStore(state => state.generateMockActivity);

  return {
    transactions: getTransactionsByPosition(positionId),
    generateMockActivity: () => generateMockActivity(positionId)
  };
};

export const useRecentActivity = (limit?: number) => {
  const getRecentActivity = useTransactionStore(state => state.getRecentActivity);
  return getRecentActivity(limit);
};