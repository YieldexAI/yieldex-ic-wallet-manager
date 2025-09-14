import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  TrendingUp, 
  Download, 
  Target,
  DollarSign,
  Activity,
  Plus,
  Minus
} from 'lucide-react';
import { useUserPositions, useRealTimeSimulation } from '@/stores/strategyStore';
import { getStrategyById } from '@/mock/strategies';
import { formatCurrency, formatAPY, formatTimeAgo } from '@/utils/formatters';
import { cardVariants, staggerContainer, listItemVariants, counterVariants } from '@/utils/animations';
import Card, { CardHeader, CardTitle, CardContent, CardFooter } from '@/components/UI/Card';
import { Section, Grid } from '@/components/UI/Layout';
import Button from '@/components/UI/Button';
import Modal from '@/components/UI/Modal';
import { TokenAmountInput } from '@/components/UI/Input';
import DepositModal from '@/components/Strategies/DepositModal';
import { clsx } from 'clsx';

const ActivePositions: React.FC = () => {
  const [selectedPosition, setSelectedPosition] = useState<string | null>(null);
  const [showWithdrawModal, setShowWithdrawModal] = useState(false);
  const [showDepositModal, setShowDepositModal] = useState(false);
  const [withdrawAmount, setWithdrawAmount] = useState('');
  
  const { 
    positions, 
    totalValue, 
    totalEarnings, 
    withdrawPosition, 
    closePosition,
    isWithdrawing 
  } = useUserPositions();
  
  const { simulationActive } = useRealTimeSimulation();

  const selectedPositionData = positions.find(p => p.id === selectedPosition);
  const selectedStrategy = selectedPositionData 
    ? getStrategyById(selectedPositionData.strategyId) 
    : null;

  const handleWithdraw = async () => {
    if (!selectedPosition || !withdrawAmount) return;
    
    try {
      await withdrawPosition(selectedPosition, parseFloat(withdrawAmount));
      setShowWithdrawModal(false);
      setWithdrawAmount('');
      setSelectedPosition(null);
    } catch (error) {
      console.error('Withdrawal failed:', error);
    }
  };

  const handleClosePosition = async (positionId: string) => {
    try {
      await closePosition(positionId);
    } catch (error) {
      console.error('Failed to close position:', error);
    }
  };

  const openWithdrawModal = (positionId: string) => {
    setSelectedPosition(positionId);
    setShowWithdrawModal(true);
    setWithdrawAmount('');
  };

  const openDepositModal = (positionId: string) => {
    setSelectedPosition(positionId);
    setShowDepositModal(true);
  };

  const handleWithdrawAll = async (positionId: string) => {
    try {
      await closePosition(positionId);
    } catch (error) {
      console.error('Failed to withdraw all:', error);
    }
  };

  if (positions.length === 0) {
    return (
      <Section title="Active Positions">
        <motion.div
          variants={cardVariants}
          initial="initial"
          animate="animate"
          className="text-center py-12 bg-gray-800/30 rounded-xl"
        >
          <Target size={48} className="text-gray-500 mx-auto mb-4" />
          <h3 className="text-lg font-semibold text-white mb-2">No Active Positions</h3>
          <p className="text-gray-400 mb-6">
            Start earning yield by selecting a strategy and making your first deposit.
          </p>
          <Button
            leftIcon={<TrendingUp size={16} />}
            onClick={() => console.log('Navigate to strategies')}
          >
            Explore Strategies
          </Button>
        </motion.div>
      </Section>
    );
  }

  return (
    <>
      <Section title="Active Positions">
        <div className="space-y-6">
          {/* Summary Cards */}
          <motion.div
            variants={staggerContainer}
            initial="hidden"
            animate="visible"
          >
            <Grid cols={3} gap="md">
              <motion.div variants={listItemVariants} custom={0}>
                <Card variant="glass" className="text-center p-4">
                  <DollarSign className="w-8 h-8 text-primary-400 mx-auto mb-2" />
                  <motion.div
                    key={totalValue}
                    variants={counterVariants}
                    initial="initial"
                    animate="animate"
                    className="text-2xl font-bold text-white"
                  >
                    {formatCurrency(totalValue)}
                  </motion.div>
                  <p className="text-sm text-gray-400">Total Invested</p>
                </Card>
              </motion.div>

              <motion.div variants={listItemVariants} custom={1}>
                <Card variant="glass" className="text-center p-4">
                  <TrendingUp className="w-8 h-8 text-green-400 mx-auto mb-2" />
                  <motion.div
                    key={totalEarnings}
                    variants={counterVariants}
                    initial="initial"
                    animate="animate"
                    className="text-2xl font-bold text-green-400"
                  >
                    {formatCurrency(totalEarnings)}
                  </motion.div>
                  <p className="text-sm text-gray-400">Total Earnings</p>
                </Card>
              </motion.div>

              <motion.div variants={listItemVariants} custom={2}>
                <Card variant="glass" className="text-center p-4">
                  <Activity className="w-8 h-8 text-blue-400 mx-auto mb-2" />
                  <div className="text-2xl font-bold text-white">
                    {positions.length}
                  </div>
                  <div className="flex items-center justify-center space-x-2 text-sm text-gray-400">
                    <span>Active</span>
                    {simulationActive && (
                      <div className="flex items-center space-x-1 text-green-400">
                        <div className="w-1.5 h-1.5 bg-green-400 rounded-full animate-pulse" />
                        <span className="text-xs">Live</span>
                      </div>
                    )}
                  </div>
                </Card>
              </motion.div>
            </Grid>
          </motion.div>

          {/* Position Cards */}
          <motion.div
            variants={staggerContainer}
            initial="hidden"
            animate="visible"
          >
            <div className="space-y-4">
              {positions.map((position, index) => {
                const strategy = getStrategyById(position.strategyId);
                const earnings = position.realTimeEarnings;
                const earningsPercentage = position.amount > 0 ? (earnings / position.amount) * 100 : 0;
                const isPositive = earnings >= 0;
                
                return (
                  <motion.div
                    key={position.id}
                    variants={listItemVariants}
                    custom={index}
                  >
                    <PositionCard
                      position={position}
                      strategy={strategy}
                      earnings={earnings}
                      earningsPercentage={earningsPercentage}
                      isPositive={isPositive}
                      onWithdraw={() => openWithdrawModal(position.id)}
                      onWithdrawAll={() => handleWithdrawAll(position.id)}
                      onAddMore={() => openDepositModal(position.id)}
                      onClose={() => handleClosePosition(position.id)}
                      isWithdrawing={isWithdrawing}
                    />
                  </motion.div>
                );
              })}
            </div>
          </motion.div>
        </div>
      </Section>

      {/* Withdraw Modal */}
      {selectedPositionData && selectedStrategy && (
        <Modal
          isOpen={showWithdrawModal}
          onClose={() => {
            setShowWithdrawModal(false);
            setSelectedPosition(null);
            setWithdrawAmount('');
          }}
          title="Withdraw from Position"
          size="md"
        >
          <div className="space-y-6">
            {/* Position Info */}
            <div className="p-4 bg-gray-800/50 rounded-lg">
              <h3 className="font-semibold text-white mb-2">{selectedStrategy.name}</h3>
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <p className="text-gray-400">Current Value</p>
                  <p className="text-white font-medium">
                    {formatCurrency(selectedPositionData.realTimeValue)}
                  </p>
                </div>
                <div>
                  <p className="text-gray-400">Available to Withdraw</p>
                  <p className="text-white font-medium">
                    {formatCurrency(selectedPositionData.realTimeValue)}
                  </p>
                </div>
              </div>
            </div>

            {/* Withdrawal Amount */}
            <TokenAmountInput
              label="Withdrawal Amount"
              value={withdrawAmount}
              onChange={setWithdrawAmount}
              selectedToken={selectedPositionData.token}
              balance={selectedPositionData.realTimeValue}
              placeholder="0.0"
            />

            {/* Actions */}
            <div className="flex space-x-3">
              <Button
                variant="ghost"
                fullWidth
                onClick={() => {
                  setShowWithdrawModal(false);
                  setSelectedPosition(null);
                  setWithdrawAmount('');
                }}
              >
                Cancel
              </Button>
              <Button
                fullWidth
                onClick={handleWithdraw}
                loading={isWithdrawing}
                disabled={!withdrawAmount || parseFloat(withdrawAmount) <= 0}
                leftIcon={<Download size={16} />}
              >
                Withdraw
              </Button>
            </div>
          </div>
        </Modal>
      )}

      {/* Add More Modal */}
      {selectedPositionData && selectedStrategy && (
        <DepositModal
          isOpen={showDepositModal}
          onClose={() => {
            setShowDepositModal(false);
            setSelectedPosition(null);
          }}
          strategy={selectedStrategy}
          existingPosition={{
            id: selectedPositionData.id,
            amount: selectedPositionData.amount,
            token: selectedPositionData.token,
            realTimeValue: selectedPositionData.realTimeValue
          }}
        />
      )}
    </>
  );
};

// Individual Position Card Component
const PositionCard: React.FC<{
  position: any;
  strategy: any;
  earnings: number;
  earningsPercentage: number;
  isPositive: boolean;
  onWithdraw: () => void;
  onWithdrawAll: () => void;
  onAddMore: () => void;
  onClose: () => void;
  isWithdrawing: boolean;
}> = ({ 
  position, 
  strategy, 
  earnings, 
  earningsPercentage, 
  isPositive, 
  onWithdraw,
  onWithdrawAll, 
  onAddMore,
  onClose,
  isWithdrawing 
}) => {
  const [showActions, setShowActions] = useState(false);

  return (
    <Card 
      variant="glass" 
      className="cursor-pointer hover:bg-gray-800/50 transition-colors"
      clickable={true}
      onClick={() => setShowActions(!showActions)}
    >
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle size="md" className="flex items-center space-x-2">
            <Target className="w-5 h-5 text-primary-400" />
            <span>{strategy?.name || 'Unknown Strategy'}</span>
          </CardTitle>
          
          <div className="flex items-center space-x-2">
            <div className="text-right">
              <motion.div
                key={position.realTimeValue}
                variants={counterVariants}
                initial="initial"
                animate="animate"
                className="text-lg font-bold text-white"
              >
                {formatCurrency(position.realTimeValue)}
              </motion.div>
              <div className={clsx(
                'text-sm font-medium',
                isPositive ? 'text-green-400' : 'text-red-400'
              )}>
                {isPositive ? '+' : ''}{formatCurrency(earnings)} ({earningsPercentage.toFixed(2)}%)
              </div>
            </div>
            
            <div className="text-gray-400 text-xs">Click to manage</div>
          </div>
        </div>
      </CardHeader>

      <CardContent>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
          <div>
            <p className="text-gray-400 mb-1">Initial Deposit</p>
            <p className="text-white font-medium">
              {formatCurrency(position.amount)} {position.token}
            </p>
          </div>
          <div>
            <p className="text-gray-400 mb-1">APY</p>
            <p className="text-primary-400 font-medium">
              {formatAPY(position.apy)}
            </p>
          </div>
          <div>
            <p className="text-gray-400 mb-1">Duration</p>
            <p className="text-white font-medium">
              {formatTimeAgo(position.entryDate)}
            </p>
          </div>
          <div>
            <p className="text-gray-400 mb-1">Status</p>
            <div className="flex items-center space-x-2">
              <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse" />
              <span className="text-green-400 font-medium">Active</span>
            </div>
          </div>
        </div>
      </CardContent>

      <AnimatePresence>
        {showActions && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            transition={{ duration: 0.2 }}
          >
            <CardFooter>
              <div className="grid grid-cols-2 gap-2 w-full">
                {/* Top row */}
                <Button
                  variant="outline"
                  onClick={onAddMore}
                  disabled={isWithdrawing}
                  leftIcon={<Plus size={16} />}
                  className="text-green-400 border-green-400/30 hover:bg-green-400/10"
                >
                  Add More
                </Button>
                <Button
                  variant="outline"
                  onClick={onWithdraw}
                  disabled={isWithdrawing}
                  leftIcon={<Download size={16} />}
                >
                  Withdraw
                </Button>
                
                {/* Bottom row */}
                <Button
                  variant="outline"
                  onClick={onWithdrawAll}
                  disabled={isWithdrawing}
                  leftIcon={<Minus size={16} />}
                  className="text-orange-400 border-orange-400/30 hover:bg-orange-400/10"
                >
                  Withdraw All
                </Button>
                <Button
                  variant="ghost"
                  onClick={onClose}
                  disabled={isWithdrawing}
                  className="text-red-400 hover:text-red-300 hover:bg-red-400/10"
                >
                  Close Position
                </Button>
              </div>
            </CardFooter>
          </motion.div>
        )}
      </AnimatePresence>
    </Card>
  );
};

export default ActivePositions;