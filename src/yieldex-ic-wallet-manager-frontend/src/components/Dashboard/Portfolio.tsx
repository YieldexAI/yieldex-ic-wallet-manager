import React, { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { TrendingUp, TrendingDown, DollarSign, PieChart, Eye, EyeOff } from 'lucide-react';
import { useWalletBalances } from '@/stores/walletStore';
import { useUserPositions, useRealTimeSimulation } from '@/stores/strategyStore';
import { formatCurrency, formatPercentage } from '@/utils/formatters';
import { cardVariants, counterVariants, staggerContainer, listItemVariants } from '@/utils/animations';
import { MetricCard } from '@/components/UI/Card';
import { Grid, Section } from '@/components/UI/Layout';
import Button from '@/components/UI/Button';
import PositionManagementModal from './PositionManagementModal';

const Portfolio: React.FC = () => {
  const [showBalances, setShowBalances] = useState(true);
  const [selectedPosition, setSelectedPosition] = useState<any>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);
  
  const { totalPortfolioValue } = useWalletBalances();
  const { 
    totalInvested, 
    totalEarnings, 
    totalValue, 
    positions,
    withdrawPosition,
    closePosition
  } = useUserPositions();
  
  const { 
    simulationActive, 
    startRealTimeSimulation, 
    stopRealTimeSimulation 
  } = useRealTimeSimulation();

  // Start real-time simulation when component mounts
  useEffect(() => {
    if (positions.length > 0 && !simulationActive) {
      startRealTimeSimulation();
    }
    
    return () => {
      if (simulationActive) {
        stopRealTimeSimulation();
      }
    };
  }, [positions.length, simulationActive, startRealTimeSimulation, stopRealTimeSimulation]);

  const totalPortfolio = totalPortfolioValue + totalValue;
  const totalGainLoss = totalEarnings;
  const gainLossPercentage = totalInvested > 0 ? (totalGainLoss / totalInvested) * 100 : 0;
  const isPositive = totalGainLoss >= 0;

  // Mock data for additional metrics
  const dailyChange = 156.78;
  const dailyChangePercentage = 0.68;
  const isDailyPositive = dailyChange >= 0;

  // Position management handlers
  const handlePositionClick = (position: any) => {
    setSelectedPosition(position);
    setIsModalOpen(true);
  };

  const handleCloseModal = () => {
    setIsModalOpen(false);
    setSelectedPosition(null);
  };

  const handleWithdraw = async (positionId: string, amount: number) => {
    await withdrawPosition(positionId, amount);
  };

  const handleClosePosition = async (positionId: string) => {
    await closePosition(positionId);
  };

  const metrics = [
    {
      label: 'Total Portfolio Value',
      value: showBalances ? formatCurrency(totalPortfolio) : '••••••',
      change: showBalances ? `${isPositive ? '+' : ''}${formatCurrency(totalGainLoss)}` : '••••',
      changeType: isPositive ? 'positive' as const : 'negative' as const,
      icon: <DollarSign size={24} />
    },
    {
      label: 'DeFi Positions',
      value: showBalances ? formatCurrency(totalValue) : '••••••',
      change: showBalances ? `${formatPercentage(gainLossPercentage)}` : '••••',
      changeType: isPositive ? 'positive' as const : 'negative' as const,
      icon: <PieChart size={24} />
    },
    {
      label: 'Total Earnings',
      value: showBalances ? formatCurrency(totalEarnings) : '••••••',
      change: showBalances ? (totalEarnings > 0 ? 'All time' : 'No earnings yet') : '••••',
      changeType: totalEarnings > 0 ? 'positive' as const : 'neutral' as const,
      icon: <TrendingUp size={24} />
    },
    {
      label: '24h Change',
      value: showBalances ? `${isDailyPositive ? '+' : ''}${formatCurrency(dailyChange)}` : '••••••',
      change: showBalances ? `${isDailyPositive ? '+' : ''}${formatPercentage(dailyChangePercentage)}` : '••••',
      changeType: isDailyPositive ? 'positive' as const : 'negative' as const,
      icon: isDailyPositive ? <TrendingUp size={24} /> : <TrendingDown size={24} />
    }
  ];

  return (
    <Section title="Portfolio Overview">
      <div className="space-y-8">
        {/* Privacy Toggle */}
        <div className="flex justify-end">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setShowBalances(!showBalances)}
            leftIcon={showBalances ? <EyeOff size={16} /> : <Eye size={16} />}
          >
            {showBalances ? 'Hide' : 'Show'} Balances
          </Button>
        </div>

        {/* Metrics Grid */}
        <motion.div
          variants={staggerContainer}
          initial="hidden"
          animate="visible"
        >
          <Grid cols={2} gap="lg" className="md:grid-cols-4">
            {metrics.map((metric, index) => (
              <motion.div
                key={metric.label}
                variants={listItemVariants}
                custom={index}
              >
                <MetricCard
                  label={metric.label}
                  value={metric.value}
                  change={metric.change}
                  changeType={metric.changeType}
                  icon={metric.icon}
                />
              </motion.div>
            ))}
          </Grid>
        </motion.div>

        {/* Portfolio Breakdown */}
        {positions.length > 0 && (
          <motion.div
            variants={cardVariants}
            initial="initial"
            animate="animate"
            className="bg-gray-800/50 rounded-xl p-6"
          >
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-lg font-semibold text-white">Active Positions</h3>
              <div className="flex items-center space-x-2">
                {simulationActive ? (
                  <div className="flex items-center space-x-2 text-green-400 text-sm">
                    <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse" />
                    <span>Live</span>
                  </div>
                ) : (
                  <div className="flex items-center space-x-2 text-gray-400 text-sm">
                    <div className="w-2 h-2 bg-gray-400 rounded-full" />
                    <span>Static</span>
                  </div>
                )}
              </div>
            </div>

            <div className="space-y-4">
              {positions.map((position, index) => {
                const earnings = position.realTimeEarnings;
                const earningsPercentage = (earnings / position.amount) * 100;
                const isPositiveEarnings = earnings >= 0;

                return (
                  <motion.div
                    key={position.id}
                    variants={listItemVariants}
                    initial="hidden"
                    animate="visible"
                    custom={index}
                    className="flex items-center justify-between p-4 bg-gray-700/30 rounded-lg cursor-pointer hover:bg-gray-700/50 transition-all duration-200"
                    onClick={() => handlePositionClick(position)}
                  >
                    <div className="flex-1">
                      <div className="flex items-center space-x-3 mb-2">
                        <h4 className="font-medium text-white">
                          Strategy Position
                        </h4>
                        <span className="px-2 py-1 bg-primary-500/20 text-primary-400 rounded text-xs">
                          {position.token}
                        </span>
                      </div>
                      <div className="flex items-center space-x-4 text-sm text-gray-400">
                        <span>
                          Initial: {showBalances ? formatCurrency(position.amount) : '••••••'}
                        </span>
                        <span>•</span>
                        <span>
                          Current: {showBalances ? formatCurrency(position.realTimeValue) : '••••••'}
                        </span>
                        <span>•</span>
                        <span>APY: {position.apy.toFixed(2)}%</span>
                      </div>
                    </div>

                    <div className="text-right">
                      <motion.div
                        key={position.realTimeEarnings}
                        variants={counterVariants}
                        initial="initial"
                        animate="animate"
                        className={`text-lg font-semibold ${
                          isPositiveEarnings ? 'text-green-400' : 'text-red-400'
                        }`}
                      >
                        {showBalances ? (
                          `${isPositiveEarnings ? '+' : ''}${formatCurrency(earnings)}`
                        ) : '••••••'}
                      </motion.div>
                      <div className="text-xs text-gray-400">
                        {showBalances ? (
                          `${isPositiveEarnings ? '+' : ''}${earningsPercentage.toFixed(2)}%`
                        ) : '••••'}
                      </div>
                    </div>
                  </motion.div>
                );
              })}
            </div>
          </motion.div>
        )}

        {/* Empty State */}
        {positions.length === 0 && (
          <motion.div
            variants={cardVariants}
            initial="initial"
            animate="animate"
            className="text-center py-12 bg-gray-800/30 rounded-xl"
          >
            <PieChart size={48} className="text-gray-500 mx-auto mb-4" />
            <h3 className="text-lg font-semibold text-white mb-2">No Active Positions</h3>
            <p className="text-gray-400 mb-6 max-w-md mx-auto">
              You don't have any active DeFi positions yet. Start by selecting a yield strategy 
              and making your first deposit to begin earning.
            </p>
            <Button
              leftIcon={<TrendingUp size={16} />}
              onClick={() => {
                // In a real app, this would navigate to strategies
                console.log('Navigate to strategies');
              }}
            >
              Explore Strategies
            </Button>
          </motion.div>
        )}

        {/* Position Management Modal */}
        <PositionManagementModal
          isOpen={isModalOpen}
          onClose={handleCloseModal}
          position={selectedPosition}
          onWithdraw={handleWithdraw}
          onClosePosition={handleClosePosition}
        />

      </div>
    </Section>
  );
};

export default Portfolio;