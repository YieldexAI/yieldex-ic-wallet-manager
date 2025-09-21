import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';
import {
  Strategy,
  UserPosition,
  STRATEGIES,
  MOCK_USER_POSITIONS,
  getStrategyById,
  simulateBalanceGrowth
} from '@/mock/strategies';

interface ActivePosition extends UserPosition {
  realTimeValue: number;
  realTimeEarnings: number;
  updatedAt: Date;
}

interface StrategyStore {
  // State
  strategies: Strategy[];
  userPositions: ActivePosition[];
  selectedStrategy: Strategy | null;
  isDepositing: boolean;
  isWithdrawing: boolean;
  error: string | null;
  
  // Real-time simulation
  simulationActive: boolean;
  simulationInterval: NodeJS.Timeout | null;
  
  // Actions
  selectStrategy: (strategyId: string) => void;
  clearSelection: () => void;
  
  // Position management
  createPosition: (strategyId: string, amount: number, token: string) => Promise<void>;
  addToPosition: (positionId: string, amount: number, token: string) => Promise<void>;
  withdrawPosition: (positionId: string, amount: number) => Promise<void>;
  closePosition: (positionId: string) => Promise<void>;
  
  // Real-time updates
  startRealTimeSimulation: () => void;
  stopRealTimeSimulation: () => void;
  updatePositionValues: () => void;
  
  // Getters
  getStrategy: (id: string) => Strategy | undefined;
  getStrategiesByRisk: (risk: 'conservative' | 'moderate' | 'aggressive') => Strategy[];
  getUserPositions: () => ActivePosition[];
  getTotalInvested: () => number;
  getTotalEarnings: () => number;
  getTotalValue: () => number;
  
  // Utility
  clearError: () => void;
}

export const useStrategyStore = create<StrategyStore>()(
  devtools(
    persist(
      (set, get) => ({
        // Initial state
        strategies: STRATEGIES,
        userPositions: [], // Start with empty positions - real positions created by user only
        selectedStrategy: null,
        isDepositing: false,
        isWithdrawing: false,
        error: null,
        simulationActive: false,
        simulationInterval: null,

        // Actions
        selectStrategy: (strategyId: string) => {
          const strategy = getStrategyById(strategyId);
          set({ selectedStrategy: strategy || null });
        },

        clearSelection: () => {
          set({ selectedStrategy: null });
        },

        createPosition: async (strategyId: string, amount: number, token: string) => {
          set({ isDepositing: true, error: null });
          
          try {
            // Simulate deposit processing
            await new Promise(resolve => setTimeout(resolve, 2000));
            
            const newPosition: ActivePosition = {
              id: `pos-${Date.now()}`,
              strategyId,
              amount,
              token,
              entryDate: new Date().toISOString(),
              currentValue: amount,
              totalEarnings: 0,
              apy: getStrategyById(strategyId)?.expectedApy || 0,
              isActive: true,
              realTimeValue: amount,
              realTimeEarnings: 0,
              updatedAt: new Date()
            };

            set(state => ({
              userPositions: [...state.userPositions, newPosition],
              isDepositing: false
            }));

            // Start real-time simulation if not already active
            const state = get();
            if (!state.simulationActive) {
              state.startRealTimeSimulation();
            }
          } catch (error) {
            set({
              isDepositing: false,
              error: error instanceof Error ? error.message : 'Failed to create position'
            });
            throw error;
          }
        },

        addToPosition: async (positionId: string, amount: number, _token: string) => {
          set({ isDepositing: true, error: null });
          
          try {
            // Simulate deposit processing
            await new Promise(resolve => setTimeout(resolve, 2000));
            
            set(state => ({
              userPositions: state.userPositions.map(pos => {
                if (pos.id === positionId) {
                  const newAmount = pos.amount + amount;
                  const newRealTimeValue = pos.realTimeValue + amount;
                  const newEarnings = newRealTimeValue - newAmount;
                  
                  return {
                    ...pos,
                    amount: newAmount,
                    currentValue: newRealTimeValue,
                    realTimeValue: newRealTimeValue,
                    totalEarnings: Math.max(0, newEarnings),
                    realTimeEarnings: Math.max(0, newEarnings),
                    updatedAt: new Date()
                  };
                }
                return pos;
              }),
              isDepositing: false
            }));
          } catch (error) {
            set({
              isDepositing: false,
              error: error instanceof Error ? error.message : 'Failed to add to position'
            });
            throw error;
          }
        },

        withdrawPosition: async (positionId: string, amount: number) => {
          set({ isWithdrawing: true, error: null });
          
          try {
            // Simulate withdrawal processing
            await new Promise(resolve => setTimeout(resolve, 2000));
            
            set(state => ({
              userPositions: state.userPositions.map(pos => {
                if (pos.id === positionId) {
                  const newAmount = Math.max(0, pos.amount - amount);
                  const newCurrentValue = Math.max(0, pos.realTimeValue - amount);
                  const newEarnings = Math.max(0, newCurrentValue - newAmount);
                  
                  return {
                    ...pos,
                    amount: newAmount,
                    currentValue: newCurrentValue,
                    realTimeValue: newCurrentValue,
                    totalEarnings: newEarnings,
                    realTimeEarnings: newEarnings,
                    updatedAt: new Date(),
                    isActive: newAmount > 0
                  };
                }
                return pos;
              }),
              isWithdrawing: false
            }));
          } catch (error) {
            set({
              isWithdrawing: false,
              error: error instanceof Error ? error.message : 'Failed to withdraw'
            });
            throw error;
          }
        },

        closePosition: async (positionId: string) => {
          const position = get().userPositions.find(p => p.id === positionId);
          if (position) {
            await get().withdrawPosition(positionId, position.realTimeValue);
          }
        },

        startRealTimeSimulation: () => {
          const state = get();
          if (state.simulationActive) return;
          
          const interval = setInterval(() => {
            get().updatePositionValues();
          }, 5000); // Update every 5 seconds
          
          set({
            simulationActive: true,
            simulationInterval: interval
          });
        },

        stopRealTimeSimulation: () => {
          const state = get();
          if (state.simulationInterval) {
            clearInterval(state.simulationInterval);
          }
          
          set({
            simulationActive: false,
            simulationInterval: null
          });
        },

        updatePositionValues: () => {
          set(state => ({
            userPositions: state.userPositions.map(position => {
              if (!position.isActive) return position;
              
              const entryDate = new Date(position.entryDate);
              const newValue = simulateBalanceGrowth(
                position.amount,
                position.apy,
                entryDate
              );
              
              const newEarnings = newValue - position.amount;
              
              return {
                ...position,
                realTimeValue: newValue,
                realTimeEarnings: newEarnings,
                currentValue: newValue,
                totalEarnings: newEarnings,
                updatedAt: new Date()
              };
            })
          }));
        },

        // Getters
        getStrategy: (id: string) => {
          return get().strategies.find(s => s.id === id);
        },

        getStrategiesByRisk: (risk: 'conservative' | 'moderate' | 'aggressive') => {
          return get().strategies.filter(s => s.risk === risk);
        },

        getUserPositions: () => {
          return get().userPositions.filter(pos => pos.isActive);
        },

        getTotalInvested: () => {
          return get().userPositions
            .filter(pos => pos.isActive)
            .reduce((sum, pos) => sum + pos.amount, 0);
        },

        getTotalEarnings: () => {
          return get().userPositions
            .filter(pos => pos.isActive)
            .reduce((sum, pos) => sum + pos.realTimeEarnings, 0);
        },

        getTotalValue: () => {
          return get().userPositions
            .filter(pos => pos.isActive)
            .reduce((sum, pos) => sum + pos.realTimeValue, 0);
        },

        clearError: () => {
          set({ error: null });
        }
      }),
      {
        name: 'yieldex-strategy-store',
        // Only persist user positions and selected strategy
        partialize: (state) => ({
          userPositions: state.userPositions,
          selectedStrategy: state.selectedStrategy
        })
      }
    ),
    { name: 'StrategyStore' }
  )
);

// Cleanup simulation on page unload
if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', () => {
    useStrategyStore.getState().stopRealTimeSimulation();
  });
}

// Selector hooks for optimized re-renders
export const useStrategies = () => useStrategyStore(state => ({
  strategies: state.strategies,
  getStrategy: state.getStrategy,
  getStrategiesByRisk: state.getStrategiesByRisk
}));

export const useStrategySelection = () => useStrategyStore(state => ({
  selectedStrategy: state.selectedStrategy,
  selectStrategy: state.selectStrategy,
  clearSelection: state.clearSelection
}));

export const useUserPositions = () => useStrategyStore(state => ({
  positions: state.getUserPositions(),
  totalInvested: state.getTotalInvested(),
  totalEarnings: state.getTotalEarnings(),
  totalValue: state.getTotalValue(),
  isDepositing: state.isDepositing,
  isWithdrawing: state.isWithdrawing,
  createPosition: state.createPosition,
  addToPosition: state.addToPosition,
  withdrawPosition: state.withdrawPosition,
  closePosition: state.closePosition
}));

export const useRealTimeSimulation = () => useStrategyStore(state => ({
  simulationActive: state.simulationActive,
  startRealTimeSimulation: state.startRealTimeSimulation,
  stopRealTimeSimulation: state.stopRealTimeSimulation,
  updatePositionValues: state.updatePositionValues
}));