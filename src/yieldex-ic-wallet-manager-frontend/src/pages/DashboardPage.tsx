import React from 'react';
import { motion } from 'framer-motion';
import { ChevronLeft, TrendingUp, Wallet } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useWalletConnection } from '@/stores/walletStore';
import { useWalletIntegration } from '@/hooks/useWalletIntegration';
import { useUserPositions } from '@/stores/strategyStore';
import { pageVariants, fadeVariants } from '@/utils/animations';
import { Container } from '@/components/UI/Layout';
import Button from '@/components/UI/Button';
import Portfolio from '@/components/Dashboard/Portfolio';

const DashboardPage: React.FC = () => {
  const navigate = useNavigate();
  const { isConnected } = useWalletConnection();
  const { realIsConnected } = useWalletIntegration();
  const { positions } = useUserPositions();

  const handleBackToStrategies = () => {
    navigate('/strategies');
  };

  const handleBackToOverview = () => {
    navigate('/');
  };

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
      {/* Page Header */}
      <Container>
        <div className="flex items-center justify-between mb-8">
          <div className="flex items-center space-x-4">
            <Button
              variant="ghost"
              onClick={handleBackToOverview}
              leftIcon={<ChevronLeft size={16} />}
            >
              Back to Overview
            </Button>

            <div className="hidden sm:block">
              <Button
                variant="ghost"
                onClick={handleBackToStrategies}
              >
                Strategies
              </Button>
            </div>
          </div>

          <div className="flex items-center space-x-2">
            <h1 className="text-2xl font-bold text-white">Dashboard</h1>
          </div>
        </div>
      </Container>

      {/* Portfolio Component - only show if wallet is connected */}
      {realIsConnected ? (
        <Portfolio />
      ) : (
        /* Wallet Not Connected State */
        <Container>
          <motion.div
            variants={fadeVariants}
            initial="initial"
            animate="animate"
            className="text-center py-12 bg-gray-800/30 rounded-xl"
          >
            <Wallet size={48} className="text-gray-500 mx-auto mb-4" />
            <h3 className="text-lg font-semibold text-white mb-2">Connect Your Wallet</h3>
            <p className="text-gray-400 mb-6 max-w-md mx-auto">
              Connect your wallet to view your portfolio and manage your DeFi positions.
            </p>
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <Button
                onClick={handleBackToOverview}
                leftIcon={<ChevronLeft size={16} />}
                variant="outline"
              >
                Back to Overview
              </Button>
              <Button
                onClick={handleExploreStrategies}
                rightIcon={<TrendingUp size={16} />}
              >
                Explore Strategies
              </Button>
            </div>
          </motion.div>
        </Container>
      )}

      {/* Empty State Action (if wallet connected but no positions) */}
      {realIsConnected && positions.length === 0 && (
        <Container>
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3 }}
            className="text-center py-8"
          >
            <div className="max-w-md mx-auto space-y-4">
              <p className="text-gray-400">
                Ready to start earning? Explore our DeFi strategies and make your first deposit.
              </p>

              <Button
                onClick={handleExploreStrategies}
                leftIcon={<TrendingUp size={16} />}
                size="lg"
              >
                Explore Strategies
              </Button>
            </div>
          </motion.div>
        </Container>
      )}

      {/* Navigation Footer */}
      <Container>
        <div className="flex justify-between items-center pt-8 border-t border-gray-800">
          <Button
            variant="ghost"
            onClick={handleBackToOverview}
            leftIcon={<ChevronLeft size={16} />}
          >
            Back to Overview
          </Button>

          <div className="text-center">
            <p className="text-sm text-gray-400">
              {!realIsConnected
                ? 'Connect wallet to view positions'
                : positions.length > 0
                ? `Managing ${positions.length} active position${positions.length > 1 ? 's' : ''}`
                : 'No active positions yet'
              }
            </p>
          </div>

          <Button
            onClick={handleExploreStrategies}
            rightIcon={<TrendingUp size={16} />}
            variant="outline"
          >
            Add Strategy
          </Button>
        </div>
      </Container>
    </motion.div>
  );
};

export default DashboardPage;