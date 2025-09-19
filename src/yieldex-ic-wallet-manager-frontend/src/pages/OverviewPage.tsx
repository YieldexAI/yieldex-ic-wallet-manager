import React from 'react';
import { motion } from 'framer-motion';
import { ArrowRight, TrendingUp } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useWalletConnection, useWalletBalances } from '@/stores/walletStore';
import { formatCurrency } from '@/utils/formatters';
import { pageVariants, fadeVariants } from '@/utils/animations';
import { Section, Container, Grid } from '@/components/UI/Layout';
import Button from '@/components/UI/Button';
import { MetricCard } from '@/components/UI/Card';

const OverviewPage: React.FC = () => {
  const navigate = useNavigate();
  const { isConnected, evmAddress } = useWalletConnection();
  const { totalPortfolioValue, balances } = useWalletBalances();

  const handleExploreStrategies = () => {
    navigate('/strategies');
  };

  const handleViewDashboard = () => {
    navigate('/dashboard');
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

      {/* Wallet Balance Overview (when connected) */}
      {isConnected && (
        <Section title="Portfolio Overview">
          <Container>
            <motion.div
              variants={fadeVariants}
              initial="initial"
              animate="animate"
              className="space-y-6"
            >
              <Grid cols={2} gap="lg" className="md:grid-cols-4">
                <MetricCard
                  label="Total Portfolio Value"
                  value={formatCurrency(totalPortfolioValue)}
                  icon={<TrendingUp size={24} />}
                />
                {Object.entries(balances).map(([token, balance]) => (
                  <MetricCard
                    key={token}
                    label={`${token} Balance`}
                    value={formatCurrency(balance)}
                    icon={<span className="text-lg">💰</span>}
                  />
                ))}
              </Grid>

              <div className="flex flex-col sm:flex-row gap-4 justify-center">
                <Button
                  onClick={handleExploreStrategies}
                  rightIcon={<ArrowRight size={16} />}
                  size="lg"
                >
                  Explore Strategies
                </Button>
                <Button
                  variant="outline"
                  onClick={handleViewDashboard}
                  rightIcon={<ArrowRight size={16} />}
                  size="lg"
                >
                  View Dashboard
                </Button>
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