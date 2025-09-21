import React, { useEffect } from 'react';
import { motion } from 'framer-motion';
import { ArrowRight, TrendingUp, RefreshCw, Wallet, PlusCircle } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useWalletConnection } from '@/stores/walletStore';
import { useUserPositions } from '@/stores/strategyStore';
import { formatCurrency } from '@/utils/formatters';
import { pageVariants, fadeVariants } from '@/utils/animations';
import { Section, Container, Grid } from '@/components/UI/Layout';
import Button from '@/components/UI/Button';
import { MetricCard } from '@/components/UI/Card';

const OverviewPage: React.FC = () => {
  const navigate = useNavigate();
  const { isConnected, address } = useWalletConnection();
  const {
    positions,
    totalInvested,
    totalEarnings,
    totalValue,
    isDepositing,
    isWithdrawing
  } = useUserPositions();

  const handleExploreStrategies = () => {
    navigate('/strategies');
  };

  const handleViewDashboard = () => {
    navigate('/dashboard');
  };

  // Get active positions count
  const activePositionsCount = positions.length;
  const hasPositions = activePositionsCount > 0;

  return (
    <motion.div
      variants={pageVariants}
      initial="initial"
      animate="enter"
      className="space-y-8"
    >
      {/* Hero Section */}
      <Section>
        <Container>
          <div className="text-center space-y-6">
            <motion.div
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ duration: 0.5 }}
            >
              <h1 className="text-4xl md:text-5xl font-bold bg-gradient-to-r from-primary-400 to-primary-600 bg-clip-text text-transparent">
                Your stable way to earn
              </h1>
              <p className="text-xl text-gray-400 mt-4 max-w-2xl mx-auto">
                The Next-Generation Cross-Chain DeFi Wallet Powered by Internet Computer
              </p>
            </motion.div>

            <div className="flex flex-wrap justify-center gap-2">
              {['Threshold ECDSA', 'Cross-Chain', 'AI-Powered', 'Zero Bridges'].map((tag, index) => (
                <motion.span
                  key={tag}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: 0.5 + index * 0.1 }}
                  className="px-3 py-1 bg-primary-500/20 text-primary-300 rounded-full text-sm border border-primary-500/30"
                >
                  {tag}
                </motion.span>
              ))}
            </div>
          </div>
        </Container>
      </Section>

      {/* Features Grid */}
      <Section>
        <Container>
          <motion.div
            variants={fadeVariants}
            initial="initial"
            animate="animate"
            className="max-w-4xl mx-auto"
          >
            <Grid cols={3} gap="lg">
              <div className="text-center p-6 bg-gray-800/30 rounded-lg">
                <div className="w-12 h-12 bg-primary-500/20 rounded-lg flex items-center justify-center mx-auto mb-4">
                  🔐
                </div>
                <h3 className="font-semibold text-white mb-2">Threshold ECDSA</h3>
                <p className="text-sm text-gray-400">No private keys, no single point of failure</p>
              </div>
              <div className="text-center p-6 bg-gray-800/30 rounded-lg">
                <div className="w-12 h-12 bg-primary-500/20 rounded-lg flex items-center justify-center mx-auto mb-4">
                  🌐
                </div>
                <h3 className="font-semibold text-white mb-2">Cross-Chain</h3>
                <p className="text-sm text-gray-400">Native multi-chain without bridges</p>
              </div>
              <div className="text-center p-6 bg-gray-800/30 rounded-lg">
                <div className="w-12 h-12 bg-primary-500/20 rounded-lg flex items-center justify-center mx-auto mb-4">
                  🤖
                </div>
                <h3 className="font-semibold text-white mb-2">AI-Powered</h3>
                <p className="text-sm text-gray-400">Automated yield optimization</p>
              </div>
            </Grid>
          </motion.div>
        </Container>
      </Section>

      {/* Strategy Portfolio Overview (when connected) */}
      {isConnected && (
        <Section>
          <Container>
            {/* Portfolio Summary */}
            <motion.div
              variants={fadeVariants}
              initial="initial"
              animate="animate"
              className="mb-8"
            >
              <div className="flex items-center justify-between mb-6">
                <div className="flex items-center space-x-4">
                  <Wallet className="text-primary-400" size={28} />
                  <div>
                    <h2 className="text-2xl font-bold text-white">Your Strategy Portfolio</h2>
                    <p className="text-gray-400">
                      Connected wallet: {address ? `${address.slice(0, 6)}...${address.slice(-4)}` : 'N/A'}
                    </p>
                  </div>
                </div>
              </div>

              {hasPositions ? (
                <Grid cols={2} gap="lg" className="md:grid-cols-3">
                  <MetricCard
                    label="Total Portfolio Value"
                    value={formatCurrency(totalValue)}
                    icon={<TrendingUp size={24} />}
                    className="md:col-span-1"
                  />
                  <MetricCard
                    label="Total Invested"
                    value={formatCurrency(totalInvested)}
                    icon={<span className="text-lg">💰</span>}
                  />
                  <MetricCard
                    label="Total Earnings"
                    value={formatCurrency(totalEarnings)}
                    icon={<span className="text-lg">📈</span>}
                    valueClassName={totalEarnings >= 0 ? 'text-green-400' : 'text-red-400'}
                  />
                </Grid>
              ) : (
                <div className="text-center py-12 bg-gray-800/30 rounded-lg border-2 border-dashed border-gray-600">
                  <div className="w-16 h-16 bg-primary-500/20 rounded-full flex items-center justify-center mx-auto mb-4">
                    <PlusCircle size={32} className="text-primary-400" />
                  </div>
                  <h3 className="text-xl font-semibold text-white mb-2">No Active Strategies</h3>
                  <p className="text-gray-400 mb-6 max-w-md mx-auto">
                    You haven't created any strategy positions yet. Start earning by deploying your first strategy.
                  </p>
                  <Button
                    onClick={handleExploreStrategies}
                    rightIcon={<ArrowRight size={16} />}
                    size="lg"
                    className="bg-gradient-to-r from-primary-500 to-primary-600 hover:from-primary-600 hover:to-primary-700"
                  >
                    Create Your First Strategy
                  </Button>
                </div>
              )}
            </motion.div>

            {/* Strategy Positions List (only show if has positions) */}
            {hasPositions && (
              <motion.div
                variants={fadeVariants}
                initial="initial"
                animate="animate"
                className="mb-8"
              >
                <h3 className="text-xl font-semibold text-white mb-6">Your Active Positions</h3>
                <div className="space-y-4">
                  {positions.map((position) => (
                    <motion.div
                      key={position.id}
                      initial={{ opacity: 0, y: 20 }}
                      animate={{ opacity: 1, y: 0 }}
                      className="bg-gray-800/50 rounded-lg p-6 border border-gray-700"
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex-1">
                          <div className="flex items-center space-x-3 mb-2">
                            <h4 className="font-medium text-white">Strategy #{position.strategyId}</h4>
                            <span className="px-2 py-1 bg-green-500/20 text-green-400 rounded text-xs">
                              Active
                            </span>
                          </div>
                          <p className="text-sm text-gray-400">
                            {position.token} • Invested: {formatCurrency(position.amount)}
                          </p>
                        </div>
                        <div className="text-right">
                          <p className="text-lg font-semibold text-white">
                            {formatCurrency(position.realTimeValue)}
                          </p>
                          <p className={`text-sm ${position.realTimeEarnings >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                            {position.realTimeEarnings >= 0 ? '+' : ''}{formatCurrency(position.realTimeEarnings)}
                          </p>
                        </div>
                      </div>
                    </motion.div>
                  ))}
                </div>
              </motion.div>
            )}

            {/* Action Buttons */}
            {hasPositions && (
              <motion.div
                variants={fadeVariants}
                initial="initial"
                animate="animate"
                className="flex flex-col sm:flex-row gap-4 justify-center"
              >
                <Button
                  onClick={handleExploreStrategies}
                  rightIcon={<ArrowRight size={16} />}
                  size="lg"
                  className="bg-gradient-to-r from-primary-500 to-primary-600 hover:from-primary-600 hover:to-primary-700"
                >
                  Manage Strategies
                </Button>
                <Button
                  variant="outline"
                  onClick={handleViewDashboard}
                  rightIcon={<ArrowRight size={16} />}
                  size="lg"
                >
                  View Dashboard
                </Button>
              </motion.div>
            )}
          </Container>
        </Section>
      )}

      {/* Call to Action (when not connected) */}
      {!isConnected && (
        <Section>
          <Container>
            <motion.div
              variants={fadeVariants}
              initial="initial"
              animate="animate"
              className="text-center space-y-6 max-w-2xl mx-auto"
            >
              <div>
                <p className="text-gray-400 mb-6">
                  Connect your wallet to get started with institutional-grade DeFi operations
                </p>
                <p className="text-sm text-gray-500 mb-8">
                  Demo mode - No real transactions will be executed
                </p>
              </div>

              <Button
                onClick={handleExploreStrategies}
                rightIcon={<ArrowRight size={16} />}
                size="lg"
              >
                Explore Strategies
              </Button>
            </motion.div>
          </Container>
        </Section>
      )}
    </motion.div>
  );
};

export default OverviewPage;