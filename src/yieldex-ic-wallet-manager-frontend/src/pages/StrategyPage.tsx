import React from 'react';
import { motion } from 'framer-motion';
import { ChevronLeft, ArrowRight } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useStrategySelection } from '@/stores/strategyStore';
import { pageVariants } from '@/utils/animations';
import { Container } from '@/components/UI/Layout';
import Button from '@/components/UI/Button';
import StrategySelector from '@/components/Strategies/StrategySelector';

const StrategyPage: React.FC = () => {
  const navigate = useNavigate();
  const { selectedStrategy } = useStrategySelection();

  const handleBackToOverview = () => {
    navigate('/');
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
      {/* Page Header */}
      <Container>
        <div className="flex items-center justify-between mb-8">
          <Button
            variant="ghost"
            onClick={handleBackToOverview}
            leftIcon={<ChevronLeft size={16} />}
          >
            Back to Overview
          </Button>

          {selectedStrategy && (
            <Button
              onClick={handleViewDashboard}
              rightIcon={<ArrowRight size={16} />}
              variant="outline"
            >
              View Dashboard
            </Button>
          )}
        </div>
      </Container>

      {/* Strategy Selector Component */}
      <StrategySelector />

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
              Select a strategy to start earning with DeFi protocols
            </p>
          </div>

          <Button
            onClick={handleViewDashboard}
            rightIcon={<ArrowRight size={16} />}
            disabled={!selectedStrategy}
          >
            Continue to Dashboard
          </Button>
        </div>
      </Container>
    </motion.div>
  );
};

export default StrategyPage;