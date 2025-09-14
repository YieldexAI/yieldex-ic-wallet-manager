import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { X, TrendingDown, AlertTriangle } from 'lucide-react';
import { formatCurrency, formatPercentage } from '@/utils/formatters';
import { modalVariants, fadeVariants } from '@/utils/animations';
import { useToast } from '@/contexts/ToastContext';
import Modal from '@/components/UI/Modal';
import Button from '@/components/UI/Button';
import Input from '@/components/UI/Input';
import Card, { CardContent, CardHeader, CardTitle } from '@/components/UI/Card';

interface Position {
  id: string;
  strategyId: string;
  token: string;
  amount: number;
  realTimeValue: number;
  realTimeEarnings: number;
  apy: number;
  entryDate: string;
  isActive: boolean;
}

interface PositionManagementModalProps {
  isOpen: boolean;
  onClose: () => void;
  position: Position | null;
  onWithdraw: (positionId: string, amount: number) => Promise<void>;
  onClosePosition: (positionId: string) => Promise<void>;
}

const PositionManagementModal: React.FC<PositionManagementModalProps> = ({
  isOpen,
  onClose,
  position,
  onWithdraw,
  onClosePosition
}) => {
  const [withdrawAmount, setWithdrawAmount] = useState('');
  const [isWithdrawing, setIsWithdrawing] = useState(false);
  const [isClosing, setIsClosing] = useState(false);
  const [withdrawMode, setWithdrawMode] = useState<'partial' | 'full'>('partial');
  const { success, error, info } = useToast();

  if (!position) return null;

  const maxWithdrawAmount = position.realTimeValue;
  const earnings = position.realTimeEarnings;
  const earningsPercentage = (earnings / position.amount) * 100;
  const isProfit = earnings >= 0;

  const handleWithdraw = async () => {
    if (!position) return;
    
    setIsWithdrawing(true);
    
    // Show info notification that processing has started
    const amount = withdrawMode === 'full' ? maxWithdrawAmount : parseFloat(withdrawAmount);
    info(
      'Processing Withdrawal...',
      `Withdrawing ${formatCurrency(amount)} from your ${position.token} position.`
    );
    
    try {
      await onWithdraw(position.id, amount);
      
      // Success notification
      if (withdrawMode === 'full') {
        success(
          'Position Withdrawn Successfully!',
          `Successfully withdrew ${formatCurrency(amount)} from your ${position.token} position.`
        );
      } else {
        success(
          'Partial Withdrawal Complete!',
          `Successfully withdrew ${formatCurrency(amount)} from your ${position.token} position. Remaining balance: ${formatCurrency(maxWithdrawAmount - amount)}.`
        );
      }
      
      // Small delay to show the success state before closing
      setTimeout(() => onClose(), 500);
    } catch (err) {
      console.error('Withdraw failed:', err);
      error(
        'Withdrawal Failed',
        'There was an error processing your withdrawal. Please try again.'
      );
    } finally {
      setIsWithdrawing(false);
    }
  };

  const handleClosePosition = async () => {
    if (!position) return;
    
    setIsClosing(true);
    
    // Show info notification that processing has started
    info(
      'Closing Position...',
      `Closing your ${position.token} position and withdrawing all funds.`
    );
    
    try {
      await onClosePosition(position.id);
      
      // Success notification
      success(
        'Position Closed Successfully!',
        `Your ${position.token} position has been completely closed and all funds have been withdrawn to your wallet.`
      );
      
      // Small delay to show the success state before closing
      setTimeout(() => onClose(), 500);
    } catch (err) {
      console.error('Close position failed:', err);
      error(
        'Failed to Close Position',
        'There was an error closing your position. Please try again.'
      );
    } finally {
      setIsClosing(false);
    }
  };

  const isValidWithdrawAmount = () => {
    if (withdrawMode === 'full') return true;
    const amount = parseFloat(withdrawAmount);
    return !isNaN(amount) && amount > 0 && amount <= maxWithdrawAmount;
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} size="lg">
      <motion.div
        variants={modalVariants}
        initial="hidden"
        animate="visible"
        exit="exit"
        className="bg-gray-800 rounded-xl p-6 w-full max-w-2xl mx-auto"
      >
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <h2 className="text-xl font-semibold text-white">Manage Position</h2>
            <p className="text-sm text-gray-400">
              Withdraw funds or close your position
            </p>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-gray-700 rounded-lg transition-colors"
          >
            <X size={20} className="text-gray-400" />
          </button>
        </div>

        {/* Position Info */}
        <Card variant="glass" className="mb-6">
          <CardHeader>
            <CardTitle size="md" className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <span>Strategy Position</span>
                <span className="px-2 py-1 bg-primary-500/20 text-primary-400 rounded text-xs">
                  {position.token}
                </span>
              </div>
              <div className={`text-lg font-semibold ${isProfit ? 'text-green-400' : 'text-red-400'}`}>
                {isProfit ? '+' : ''}{formatCurrency(earnings)}
              </div>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
              <div>
                <p className="text-gray-400">Initial Amount</p>
                <p className="text-white font-medium">{formatCurrency(position.amount)}</p>
              </div>
              <div>
                <p className="text-gray-400">Current Value</p>
                <p className="text-white font-medium">{formatCurrency(position.realTimeValue)}</p>
              </div>
              <div>
                <p className="text-gray-400">APY</p>
                <p className="text-white font-medium">{position.apy.toFixed(2)}%</p>
              </div>
              <div>
                <p className="text-gray-400">P&L</p>
                <p className={`font-medium ${isProfit ? 'text-green-400' : 'text-red-400'}`}>
                  {isProfit ? '+' : ''}{formatPercentage(earningsPercentage)}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Withdraw Section */}
        <div className="space-y-4 mb-6">
          <h3 className="text-lg font-semibold text-white flex items-center">
            <TrendingDown size={20} className="mr-2" />
            Withdraw Funds
          </h3>
          
          {/* Withdraw Mode Toggle */}
          <div className="flex space-x-2">
            <Button
              variant={withdrawMode === 'partial' ? 'primary' : 'ghost'}
              size="sm"
              onClick={() => setWithdrawMode('partial')}
            >
              Partial Withdraw
            </Button>
            <Button
              variant={withdrawMode === 'full' ? 'primary' : 'ghost'}
              size="sm"
              onClick={() => setWithdrawMode('full')}
            >
              Full Withdraw
            </Button>
          </div>

          {withdrawMode === 'partial' && (
            <motion.div
              variants={fadeVariants}
              initial="initial"
              animate="animate"
              className="space-y-3"
            >
              <Input
                label="Withdraw Amount"
                placeholder="0.00"
                value={withdrawAmount}
                onChange={(e) => setWithdrawAmount(e.target.value)}
                rightElement={
                  <div className="flex items-center space-x-2">
                    <span className="text-sm text-gray-400">{position.token}</span>
                    <button
                      onClick={() => setWithdrawAmount(maxWithdrawAmount.toString())}
                      className="text-xs text-primary-400 hover:text-primary-300"
                    >
                      Max
                    </button>
                  </div>
                }
              />
              <div className="flex justify-between text-sm text-gray-400">
                <span>Available to withdraw:</span>
                <span>{formatCurrency(maxWithdrawAmount)}</span>
              </div>
            </motion.div>
          )}

          {withdrawMode === 'full' && (
            <motion.div
              variants={fadeVariants}
              initial="initial"
              animate="animate"
              className="bg-yellow-500/10 border border-yellow-500/20 rounded-lg p-4"
            >
              <div className="flex items-start space-x-3">
                <AlertTriangle size={20} className="text-yellow-400 mt-0.5 flex-shrink-0" />
                <div>
                  <p className="text-yellow-400 font-medium">Full Withdrawal</p>
                  <p className="text-sm text-gray-300 mt-1">
                    You will withdraw the entire position value of {formatCurrency(maxWithdrawAmount)}.
                    This will close your position completely.
                  </p>
                </div>
              </div>
            </motion.div>
          )}

          <Button
            onClick={handleWithdraw}
            disabled={!isValidWithdrawAmount() || isWithdrawing}
            loading={isWithdrawing}
            className="w-full"
            leftIcon={<TrendingDown size={16} />}
          >
            {withdrawMode === 'full' 
              ? `Withdraw All (${formatCurrency(maxWithdrawAmount)})`
              : `Withdraw ${withdrawAmount ? formatCurrency(parseFloat(withdrawAmount)) : '0.00'}`
            }
          </Button>
        </div>

        {/* Close Position Section */}
        {withdrawMode !== 'full' && (
          <div className="border-t border-gray-700 pt-6">
            <h3 className="text-lg font-semibold text-white flex items-center mb-3">
              <X size={20} className="mr-2" />
              Close Position
            </h3>
            
            <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-4 mb-4">
              <div className="flex items-start space-x-3">
                <AlertTriangle size={20} className="text-red-400 mt-0.5 flex-shrink-0" />
                <div>
                  <p className="text-red-400 font-medium">Permanent Action</p>
                  <p className="text-sm text-gray-300 mt-1">
                    Closing this position will withdraw all funds and end the strategy. 
                    This action cannot be undone.
                  </p>
                </div>
              </div>
            </div>

            <Button
              onClick={handleClosePosition}
              disabled={isClosing}
              loading={isClosing}
              variant="danger"
              className="w-full"
              leftIcon={<X size={16} />}
            >
              Close Position Completely
            </Button>
          </div>
        )}
      </motion.div>
    </Modal>
  );
};

export default PositionManagementModal;