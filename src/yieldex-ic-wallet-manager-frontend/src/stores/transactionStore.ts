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
  getRecentActivity: (limit?: number) => (Transaction | TransactionGroup)[];
  getFilteredTransactions: () => Transaction[];
  getTransactionsByType: (type: TransactionType) => Transaction[];

  // Utilities
  generateMockActivity: (positionId: string) => void;
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

          // Create some mock transactions for testing
          const now = Date.now();

          // Recent rebalancing group
          const rebalanceGroup: Omit<TransactionGroup, 'id' | 'timestamp'> = {
            type: 'rebalancing_sequence',
            description: 'Rebalanced to higher yield protocol',
            status: 'completed',
            totalGasCost: 0.0025,
            transactions: [
              {
                id: `tx-${now}-1`,
                positionId,
                type: 'opportunity_detection',
                status: 'completed',
                timestamp: new Date(now - 15 * 60 * 1000).toISOString(),
                description: 'Found better yield opportunity: Compound V3 (6.1% APY)',
                icon: '🎯',
                color: 'text-blue-400',
                blockchainRefs: []
              },
              {
                id: `tx-${now}-2`,
                positionId,
                type: 'withdrawal',
                status: 'completed',
                timestamp: new Date(now - 12 * 60 * 1000).toISOString(),
                amount: 4.50,
                token: 'DAI',
                description: 'Withdrew from AAVE V3',
                icon: '📤',
                color: 'text-orange-400',
                blockchainRefs: [{
                  network: 'ethereum',
                  txHash: '0x1234567890abcdef1234567890abcdef12345678',
                  explorerUrl: 'https://sepolia.etherscan.io/tx/0x1234567890abcdef1234567890abcdef12345678'
                }]
              },
              {
                id: `tx-${now}-3`,
                positionId,
                type: 'deposit',
                status: 'completed',
                timestamp: new Date(now - 10 * 60 * 1000).toISOString(),
                amount: 4.50,
                token: 'DAI',
                description: 'Deposited to Compound V3',
                icon: '📥',
                color: 'text-green-400',
                blockchainRefs: [{
                  network: 'arbitrum',
                  txHash: '0xabcdef1234567890abcdef1234567890abcdef12',
                  explorerUrl: 'https://arbiscan.io/tx/0xabcdef1234567890abcdef1234567890abcdef12'
                }]
              }
            ]
          };

          addTransactionGroup(rebalanceGroup);

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