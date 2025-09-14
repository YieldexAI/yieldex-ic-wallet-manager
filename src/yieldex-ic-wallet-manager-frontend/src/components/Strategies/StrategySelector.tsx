import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Filter, TrendingUp, Shield, Zap, Target } from 'lucide-react';
import { useStrategies, useStrategySelection } from '@/stores/strategyStore';
import { Strategy } from '@/mock/strategies';
import { staggerContainer, listItemVariants, fadeVariants } from '@/utils/animations';
import { Section, Grid } from '@/components/UI/Layout';
import Button from '@/components/UI/Button';
import StrategyCard from './StrategyCard';
import DepositModal from './DepositModal';
import { clsx } from 'clsx';

type RiskFilter = 'all' | 'conservative' | 'moderate' | 'aggressive';

const StrategySelector: React.FC = () => {
  const [riskFilter, setRiskFilter] = useState<RiskFilter>('all');
  const [showDepositModal, setShowDepositModal] = useState(false);
  
  const { strategies, getStrategiesByRisk } = useStrategies();
  const { selectedStrategy, selectStrategy } = useStrategySelection();

  const filteredStrategies = riskFilter === 'all' 
    ? strategies 
    : getStrategiesByRisk(riskFilter);

  const handleStrategySelect = (strategy: Strategy) => {
    selectStrategy(strategy.id);
  };

  const handleStartEarning = () => {
    if (selectedStrategy) {
      setShowDepositModal(true);
    }
  };

  const riskFilters = [
    { id: 'all' as const, label: 'All Strategies', icon: TrendingUp, count: strategies.length },
    { id: 'conservative' as const, label: 'Conservative', icon: Shield, count: getStrategiesByRisk('conservative').length },
    { id: 'moderate' as const, label: 'Moderate', icon: Target, count: getStrategiesByRisk('moderate').length },
    { id: 'aggressive' as const, label: 'Aggressive', icon: Zap, count: getStrategiesByRisk('aggressive').length },
  ];

  return (
    <div className="space-y-8">
      {/* Header */}
      <Section 
        title="Choose Your Yield Strategy" 
        description="Select from our curated DeFi strategies optimized for different risk profiles and return expectations."
      >
        {/* Risk Filter Tabs */}
        <motion.div 
          className="flex flex-wrap gap-3 mb-8"
          variants={staggerContainer}
          initial="hidden"
          animate="visible"
        >
          {riskFilters.map((filter, index) => {
            const Icon = filter.icon;
            const isActive = riskFilter === filter.id;
            
            return (
              <motion.div key={filter.id} variants={listItemVariants} custom={index}>
                <Button
                  variant={isActive ? 'primary' : 'outline'}
                  onClick={() => setRiskFilter(filter.id)}
                  leftIcon={<Icon size={16} />}
                  className={clsx(
                    'transition-all duration-200',
                    !isActive && 'hover:border-primary-500/50 hover:text-primary-300'
                  )}
                >
                  {filter.label}
                  <span className="ml-2 px-1.5 py-0.5 bg-white/10 rounded text-xs">
                    {filter.count}
                  </span>
                </Button>
              </motion.div>
            );
          })}
        </motion.div>

        {/* Strategy Cards */}
        <AnimatePresence mode="wait">
          <motion.div
            key={riskFilter}
            variants={staggerContainer}
            initial="hidden"
            animate="visible"
            exit="hidden"
          >
            <Grid cols={3} gap="lg">
              {filteredStrategies.map((strategy, index) => (
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
          </motion.div>
        </AnimatePresence>

        {/* Empty State */}
        {filteredStrategies.length === 0 && (
          <motion.div
            variants={fadeVariants}
            initial="initial"
            animate="animate"
            className="text-center py-12"
          >
            <Filter className="w-16 h-16 text-gray-500 mx-auto mb-4" />
            <h3 className="text-lg font-semibold text-white mb-2">No Strategies Found</h3>
            <p className="text-gray-400">
              No strategies match your current filter criteria.
            </p>
          </motion.div>
        )}
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