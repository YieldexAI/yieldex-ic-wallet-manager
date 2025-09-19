import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Filter, TrendingUp, Shield, Zap, Target, Wallet, Sparkles, Settings } from 'lucide-react';
import { useWalletIntegration } from '@/hooks/useWalletIntegration';
import { useStrategies, useStrategySelection } from '@/stores/strategyStore';
import { Strategy } from '@/mock/strategies';
import { staggerContainer, listItemVariants, fadeVariants } from '@/utils/animations';
import { Section, Grid } from '@/components/UI/Layout';
import Button from '@/components/UI/Button';
import StrategyCard from './StrategyCard';
import DepositModal from './DepositModal';
import RealWalletConnect from '@/components/Wallet/RealWalletConnect';
import CustomStrategyBuilder from './CustomStrategyBuilder';
import { clsx } from 'clsx';

type TabFilter = 'curated' | 'custom';

const StrategySelector: React.FC = () => {
  const [activeTab, setActiveTab] = useState<TabFilter>('curated');
  const [showDepositModal, setShowDepositModal] = useState(false);
  const [showWalletPrompt, setShowWalletPrompt] = useState(false);

  const { realIsConnected } = useWalletIntegration();
  const { strategies } = useStrategies();
  const { selectedStrategy, selectStrategy } = useStrategySelection();

  const filteredStrategies = activeTab === 'curated'
    ? strategies
    : [];

  const handleStrategySelect = (strategy: Strategy) => {
    selectStrategy(strategy.id);
  };

  const handleStartEarning = () => {
    if (!selectedStrategy) return;

    if (!realIsConnected) {
      setShowWalletPrompt(true);
    } else {
      setShowDepositModal(true);
    }
  };

  const handleWalletConnected = () => {
    // Close wallet prompt and open deposit modal
    setShowWalletPrompt(false);
    setShowDepositModal(true);
  };

  const handleWalletPromptClose = () => {
    setShowWalletPrompt(false);
  };

  const tabFilters = [
    { id: 'curated' as const, label: 'Curated', icon: Sparkles, description: 'DeFi strategies optimized for different risk profiles' },
    { id: 'custom' as const, label: 'Custom', icon: Settings, description: 'Build your own strategy with selected protocols' },
  ];

  return (
    <div className="space-y-8">
      {/* Header */}
      <Section
        title="Choose Your Yield Strategy"
        description="Select from our curated DeFi strategies optimized for different risk profiles and return expectations."
      >
        {/* Strategy Tabs */}
        <motion.div
          className="flex gap-6 mb-8 border-b border-gray-700"
          variants={staggerContainer}
          initial="hidden"
          animate="visible"
        >
          {tabFilters.map((tab, index) => {
            const Icon = tab.icon;
            const isActive = activeTab === tab.id;

            return (
              <motion.div key={tab.id} variants={listItemVariants} custom={index}>
                <button
                  onClick={() => setActiveTab(tab.id)}
                  className={clsx(
                    'flex items-center space-x-2 px-4 py-3 text-sm font-medium transition-all duration-200 border-b-2',
                    isActive
                      ? 'border-primary-500 text-primary-400'
                      : 'border-transparent text-gray-400 hover:text-gray-300 hover:border-gray-500'
                  )}
                >
                  <Icon size={16} />
                  <span>{tab.label}</span>
                </button>
              </motion.div>
            );
          })}
        </motion.div>

        {/* Tab Description */}
        <motion.p
          className="text-gray-400 text-sm mb-6"
          variants={fadeVariants}
          initial="initial"
          animate="animate"
        >
          {tabFilters.find(tab => tab.id === activeTab)?.description}
        </motion.p>

        {/* Tab Content */}
        <AnimatePresence mode="wait">
          <motion.div
            key={activeTab}
            variants={staggerContainer}
            initial="hidden"
            animate="visible"
            exit="hidden"
          >
            {activeTab === 'curated' && (
              <Grid cols={3} gap="lg">
                {strategies.map((strategy, index) => (
                  <motion.div
                    key={strategy.id}
                    variants={listItemVariants}
                    custom={index}
                  >
                    <StrategyCard
                      strategy={strategy}
                      onSelect={handleStrategySelect}
                      isSelected={selectedStrategy?.id === strategy.id}
                    />
                  </motion.div>
                ))}
              </Grid>
            )}

            {activeTab === 'custom' && (
              <CustomStrategyBuilder />
            )}
          </motion.div>
        </AnimatePresence>
      </Section>

      {/* Selected Strategy Actions */}
      <AnimatePresence>
        {selectedStrategy && (
          <motion.div
            variants={fadeVariants}
            initial="initial"
            animate="animate"
            exit="exit"
            className="sticky bottom-6 z-40"
          >
            <div className="bg-gray-800/90 backdrop-blur-xl border border-gray-700/50 rounded-xl p-6 shadow-2xl">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="font-semibold text-white mb-1">
                    {selectedStrategy.name} Selected
                  </h3>
                  <p className="text-sm text-gray-400">
                    Expected APY: <span className="text-primary-400 font-medium">
                      {selectedStrategy.expectedApy.toFixed(2)}%
                    </span>
                  </p>
                </div>
                
                <div className="flex items-center space-x-3">
                  <Button
                    variant="ghost"
                    onClick={() => selectStrategy('')}
                  >
                    Cancel
                  </Button>
                  <Button
                    onClick={handleStartEarning}
                    leftIcon={<TrendingUp size={16} />}
                    size="lg"
                  >
                    Start Earning
                  </Button>
                </div>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Strategy Comparison */}
      <Section title="Strategy Comparison">
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="text-left py-3 text-sm font-medium text-gray-300">Strategy</th>
                <th className="text-center py-3 text-sm font-medium text-gray-300">APY</th>
                <th className="text-center py-3 text-sm font-medium text-gray-300">Risk</th>
                <th className="text-center py-3 text-sm font-medium text-gray-300">Min Deposit</th>
                <th className="text-center py-3 text-sm font-medium text-gray-300">Protocols</th>
              </tr>
            </thead>
            <tbody>
              {strategies.map((strategy, index) => (
                <motion.tr
                  key={strategy.id}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: index * 0.1 }}
                  className="border-b border-gray-800/50 hover:bg-gray-800/30 transition-colors"
                >
                  <td className="py-4">
                    <div className="font-medium text-white">{strategy.name}</div>
                    <div className="text-xs text-gray-400 mt-1">
                      {strategy.description.substring(0, 60)}...
                    </div>
                  </td>
                  <td className="text-center py-4">
                    <span className="text-primary-400 font-semibold">
                      {strategy.expectedApy.toFixed(2)}%
                    </span>
                  </td>
                  <td className="text-center py-4">
                    <span className={clsx(
                      'px-2 py-1 rounded-full text-xs font-medium',
                      strategy.risk === 'conservative' && 'bg-green-500/20 text-green-400',
                      strategy.risk === 'moderate' && 'bg-yellow-500/20 text-yellow-400',
                      strategy.risk === 'aggressive' && 'bg-red-500/20 text-red-400'
                    )}>
                      {strategy.risk}
                    </span>
                  </td>
                  <td className="text-center py-4 text-gray-300">
                    ${strategy.minDeposit.toLocaleString()}
                  </td>
                  <td className="text-center py-4 text-gray-300">
                    {strategy.protocols.length}
                  </td>
                </motion.tr>
              ))}
            </tbody>
          </table>
        </div>
      </Section>

      {/* Wallet Connection Prompt Modal */}
      <AnimatePresence>
        {showWalletPrompt && selectedStrategy && (
          <motion.div
            variants={fadeVariants}
            initial="initial"
            animate="animate"
            exit="exit"
            className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4"
            onClick={() => setShowWalletPrompt(false)}
          >
            <motion.div
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.9 }}
              className="bg-gray-800/90 backdrop-blur-xl border border-gray-700/50 rounded-xl p-6 max-w-md w-full shadow-2xl"
              onClick={(e) => e.stopPropagation()}
            >
              <div className="text-center space-y-4">
                <div className="w-12 h-12 bg-primary-500/20 rounded-full flex items-center justify-center mx-auto">
                  <Wallet size={24} className="text-primary-400" />
                </div>

                <div>
                  <h3 className="text-lg font-semibold text-white mb-2">Connect Your Wallet</h3>
                  <p className="text-gray-400 text-sm">
                    Connect your wallet to start earning with {selectedStrategy.name} strategy.
                  </p>
                </div>

                <div className="bg-gray-700/50 rounded-lg p-4 border border-gray-600/50">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-300">Selected Strategy:</span>
                    <span className="text-white font-medium">{selectedStrategy.name}</span>
                  </div>
                  <div className="flex items-center justify-between text-sm mt-2">
                    <span className="text-gray-300">Expected APY:</span>
                    <span className="text-primary-400 font-medium">{selectedStrategy.expectedApy.toFixed(2)}%</span>
                  </div>
                </div>

                {/* Embedded Real Wallet Connection */}
                <div className="mt-6">
                  <RealWalletConnect
                    showModal={showWalletPrompt}
                    onModalClose={handleWalletPromptClose}
                    onConnectionSuccess={handleWalletConnected}
                  />
                </div>

                <div className="flex space-x-3 pt-4">
                  <Button
                    variant="ghost"
                    onClick={handleWalletPromptClose}
                    className="flex-1"
                  >
                    Cancel
                  </Button>
                </div>

                <p className="text-xs text-gray-500 mt-4">
                  Choose a wallet option above to connect and start earning.
                </p>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Deposit Modal */}
      {selectedStrategy && (
        <DepositModal
          isOpen={showDepositModal}
          onClose={() => setShowDepositModal(false)}
          strategy={selectedStrategy}
        />
      )}
    </div>
  );
};

export default StrategySelector;