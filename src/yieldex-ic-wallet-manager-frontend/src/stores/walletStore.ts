import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';
import { 
  WalletState, 
  INITIAL_WALLET_STATE,
  mockConnectWallet,
  mockGenerateEvmAddress,
  mockDeposit,
  mockWithdraw,
  getNetworkTVL,
  getTotalBalanceByToken
} from '@/mock/walletData';
import { NETWORKS } from '@/mock/protocols';

interface WalletStore extends WalletState {
  // Connection states
  isConnecting: boolean;
  isGeneratingAddress: boolean;
  connectionError: string | null;
  
  // Actions
  connectWallet: () => Promise<void>;
  disconnectWallet: () => void;
  generateEvmAddress: () => Promise<void>;
  switchNetwork: (networkId: number) => void;
  
  // Transaction actions
  deposit: (amount: number, token: string, strategy: string) => Promise<void>;
  withdraw: (amount: number, token: string, strategy: string) => Promise<void>;
  
  // Utility actions
  refreshBalances: () => Promise<void>;
  clearError: () => void;
  
  // Getters
  getCurrentNetwork: () => typeof NETWORKS[0] | undefined;
  getTokenBalance: (token: string) => number;
  getNetworkValue: (networkSlug: string) => number;
}

export const useWalletStore = create<WalletStore>()(
  devtools(
    persist(
      (set, get) => ({
        // Initial state
        ...INITIAL_WALLET_STATE,
        isConnecting: false,
        isGeneratingAddress: false,
        connectionError: null,

        // Actions
        connectWallet: async () => {
          set({ isConnecting: true, connectionError: null });
          
          try {
            const walletData = await mockConnectWallet();
            set({ 
              ...walletData,
              isConnecting: false 
            });
          } catch (error) {
            set({ 
              isConnecting: false,
              connectionError: error instanceof Error ? error.message : 'Failed to connect wallet'
            });
          }
        },

        disconnectWallet: () => {
          set({
            ...INITIAL_WALLET_STATE,
            connectionError: null
          });
        },

        generateEvmAddress: async () => {
          set({ isGeneratingAddress: true, connectionError: null });
          
          try {
            const evmAddress = await mockGenerateEvmAddress();
            set({ 
              evmAddress,
              isGeneratingAddress: false 
            });
          } catch (error) {
            set({ 
              isGeneratingAddress: false,
              connectionError: error instanceof Error ? error.message : 'Failed to generate EVM address'
            });
          }
        },

        switchNetwork: (networkId: number) => {
          const network = NETWORKS.find(n => n.id === networkId);
          if (network) {
            set({ networkId });
          }
        },

        deposit: async (amount: number, token: string, strategy: string) => {
          const state = get();
          const currentNetwork = state.getCurrentNetwork();
          
          if (!currentNetwork) {
            throw new Error('No network selected');
          }

          try {
            const transaction = await mockDeposit(
              amount,
              token,
              currentNetwork.slug,
              strategy
            );

            set({
              transactions: [transaction, ...state.transactions]
            });

            // Refresh balances after successful deposit
            await state.refreshBalances();
          } catch (error) {
            set({
              connectionError: error instanceof Error ? error.message : 'Deposit failed'
            });
            throw error;
          }
        },

        withdraw: async (amount: number, token: string, strategy: string) => {
          const state = get();
          const currentNetwork = state.getCurrentNetwork();
          
          if (!currentNetwork) {
            throw new Error('No network selected');
          }

          try {
            const transaction = await mockWithdraw(
              amount,
              token,
              currentNetwork.slug,
              strategy
            );

            set({
              transactions: [transaction, ...state.transactions]
            });

            // Refresh balances after successful withdrawal
            await state.refreshBalances();
          } catch (error) {
            set({
              connectionError: error instanceof Error ? error.message : 'Withdrawal failed'
            });
            throw error;
          }
        },

        refreshBalances: async () => {
          // In a real app, this would fetch fresh data from the blockchain
          // For demo purposes, we simulate balance updates
          const state = get();
          const updatedBalances = state.balances.map(balance => ({
            ...balance,
            // Simulate small random changes in balances
            balance: balance.balance + (Math.random() - 0.5) * 10,
            value: balance.value + (Math.random() - 0.5) * 10
          }));

          const newTotalValue = updatedBalances.reduce((sum, balance) => sum + balance.value, 0);

          set({
            balances: updatedBalances,
            totalPortfolioValue: newTotalValue
          });
        },

        clearError: () => {
          set({ connectionError: null });
        },

        // Getters
        getCurrentNetwork: () => {
          const state = get();
          return NETWORKS.find(n => n.id === state.networkId);
        },

        getTokenBalance: (token: string) => {
          return getTotalBalanceByToken(token);
        },

        getNetworkValue: (networkSlug: string) => {
          return getNetworkTVL(networkSlug);
        }
      }),
      {
        name: 'yieldex-wallet-store',
        // Only persist essential data
        partialize: (state) => ({
          isConnected: state.isConnected,
          address: state.address,
          evmAddress: state.evmAddress,
          principal: state.principal,
          networkId: state.networkId
        })
      }
    ),
    { name: 'WalletStore' }
  )
);

// Selector hooks for optimized re-renders
export const useWalletConnection = () => useWalletStore(state => ({
  isConnected: state.isConnected,
  isConnecting: state.isConnecting,
  address: state.address,
  evmAddress: state.evmAddress,
  connectionError: state.connectionError,
  connectWallet: state.connectWallet,
  disconnectWallet: state.disconnectWallet,
  clearError: state.clearError
}));

export const useWalletBalances = () => useWalletStore(state => ({
  balances: state.balances,
  totalPortfolioValue: state.totalPortfolioValue,
  refreshBalances: state.refreshBalances,
  getTokenBalance: state.getTokenBalance,
  getNetworkValue: state.getNetworkValue
}));

export const useWalletNetwork = () => useWalletStore(state => ({
  networkId: state.networkId,
  switchNetwork: state.switchNetwork,
  getCurrentNetwork: state.getCurrentNetwork
}));

export const useWalletTransactions = () => useWalletStore(state => ({
  transactions: state.transactions,
  deposit: state.deposit,
  withdraw: state.withdraw
}));