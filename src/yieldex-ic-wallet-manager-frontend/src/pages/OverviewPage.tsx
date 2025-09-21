import React from 'react';
import { motion } from 'framer-motion';
import { ArrowRight, TrendingUp, RefreshCw, WalletMinimal } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useWalletConnection, useStablecoinBalances } from '@/stores/walletStore';
import { pageVariants, fadeVariants } from '@/utils/animations';
import { Section, Container, Grid } from '@/components/UI/Layout';
import Button from '@/components/UI/Button';
import { MetricCard } from '@/components/UI/Card';
import { StablecoinGrid } from '@/components/Stablecoins';

const OverviewPage: React.FC = () => {
  const navigate = useNavigate();
  const { isConnected, address } = useWalletConnection();
  const {
    stablecoinBalances,
    portfolioSummary,
    isLoadingBalances,
    balancesError,
    lastBalanceUpdate,
    refreshStablecoinBalances
  } = useStablecoinBalances();

  const handleExploreStrategies = () => {
    navigate('/strategies');
  };


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

      {/* Stablecoin Portfolio Overview (when connected) */}
      {isConnected && (
        <Section>
          <Container>
            <motion.div
              variants={fadeVariants}
              initial="initial"
              animate="animate"
              className="mb-8"
            >
              <div className="flex items-center justify-between mb-6">
                <div className="flex items-center space-x-4">
                  <div className="w-8 h-8 bg-primary-500/20 rounded-lg flex items-center justify-center">
                    <WalletMinimal size={18} className="text-primary-400" />
                  </div>
                  <div>
                    <h2 className="text-2xl font-bold text-white">Your Stablecoin Portfolio</h2>
                    <p className="text-gray-400">
                      Connected wallet: {address ? `${address.slice(0, 6)}...${address.slice(-4)}` : 'N/A'}
                    </p>
                  </div>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={refreshStablecoinBalances}
                  disabled={isLoadingBalances}
                  leftIcon={<RefreshCw size={16} className={isLoadingBalances ? 'animate-spin' : ''} />}
                >
                  Refresh
                </Button>
              </div>

              {/* Portfolio Summary */}
              {portfolioSummary && (
                <Grid cols={2} gap="lg" className="md:grid-cols-3 mb-8">
                  <MetricCard
                    label="Total Portfolio Value"
                    value={`$${(portfolioSummary.totalUsdValue || 0).toFixed(2)}`}
                    icon={<TrendingUp size={24} />}
                    className="md:col-span-1"
                  />
                  <MetricCard
                    label="Stablecoins Held"
                    value={(portfolioSummary.totalTokens || 0).toString()}
                    icon={<span className="text-lg">🪙</span>}
                  />
                  <MetricCard
                    label="Networks"
                    value={(portfolioSummary.networks?.length || 0).toString()}
                    icon={<span className="text-lg">🌐</span>}
                  />
                </Grid>
              )}

              {/* Last Updated */}
              {lastBalanceUpdate && (
                <p className="text-sm text-gray-500 mb-6">
                  Last updated: {lastBalanceUpdate.toLocaleString()}
                </p>
              )}

              {/* Stablecoin Grid */}
              <div>
                <h3 className="text-xl font-semibold text-white mb-6">Your Stablecoins</h3>
                <StablecoinGrid
                  balances={stablecoinBalances}
                  isLoading={isLoadingBalances}
                  error={balancesError}
                />
              </div>
            </motion.div>
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