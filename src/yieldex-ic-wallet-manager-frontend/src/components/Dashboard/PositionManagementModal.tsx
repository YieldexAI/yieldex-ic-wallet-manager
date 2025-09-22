import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { X, TrendingDown, AlertTriangle, Activity, BarChart3, Settings } from 'lucide-react';
import { formatCurrency, formatPercentage } from '@/utils/formatters';
import { modalVariants, fadeVariants } from '@/utils/animations';
import { useToast } from '@/contexts/ToastContext';
import Modal from '@/components/UI/Modal';
import Button from '@/components/UI/Button';
import Input from '@/components/UI/Input';
import Card, { CardContent, CardHeader, CardTitle } from '@/components/UI/Card';
import ActivityTimeline from './ActivityTimeline';
import { clsx } from 'clsx';

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

type TabType = 'manage' | 'activity' | 'analytics';

const PositionManagementModal: React.FC<PositionManagementModalProps> = ({
  isOpen,
  onClose,
  position,
  onWithdraw,
  onClosePosition
}) => {
  const [activeTab, setActiveTab] = useState<TabType>('manage');
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

  const tabs = [
    { id: 'manage' as TabType, label: 'Manage', icon: TrendingDown },
    { id: 'activity' as TabType, label: 'Activity', icon: Activity },
    { id: 'analytics' as TabType, label: 'Analytics', icon: BarChart3 },
  ];

  return (
    <Modal isOpen={isOpen} onClose={onClose} size="lg">
      <motion.div
        variants={modalVariants}
        initial="hidden"
        animate="visible"
        exit="exit"
        className="bg-gray-800 rounded-xl p-6 w-full max-w-3xl mx-auto"
      >
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <h2 className="text-xl font-semibold text-white">Position Details</h2>
            <p className="text-sm text-gray-400">
              Manage your position, view activity, and analyze performance
            </p>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-gray-700 rounded-lg transition-colors"
          >
            <X size={20} className="text-gray-400" />
          </button>
        </div>

        {/* Tabs */}
        <div className="flex space-x-1 mb-6 bg-gray-900/50 rounded-lg p-1">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={clsx(
                  'flex items-center space-x-2 px-4 py-2 rounded-md text-sm font-medium transition-colors flex-1 justify-center',
                  activeTab === tab.id
                    ? 'bg-primary-500 text-white'
                    : 'text-gray-400 hover:text-white hover:bg-gray-700'
                )}
              >
                <Icon size={16} />
                <span>{tab.label}</span>
              </button>
            );
          })}
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

        {/* Tab Content */}
        <div className="min-h-[400px]">
          {activeTab === 'manage' && (
            <motion.div
              variants={fadeVariants}
              initial="initial"
              animate="animate"
              className="space-y-6"
            >
              {/* Withdraw Section */}
              <div className="space-y-4">
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
          )}

          {activeTab === 'activity' && (
            <motion.div
              variants={fadeVariants}
              initial="initial"
              animate="animate"
            >
              <ActivityTimeline
                positionId={position.id}
                limit={20}
                showFilters={true}
                className="bg-transparent border-0"
              />
            </motion.div>
          )}

          {activeTab === 'analytics' && (
            <motion.div
              variants={fadeVariants}
              initial="initial"
              animate="animate"
              className="space-y-6"
            >
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <Card variant="glass">
                  <CardContent className="p-4">
                    <h4 className="text-sm font-medium text-gray-400 mb-1">Total Return</h4>
                    <p className={clsx(
                      'text-2xl font-bold',
                      isProfit ? 'text-green-400' : 'text-red-400'
                    )}>
                      {isProfit ? '+' : ''}{formatPercentage(earningsPercentage)}
                    </p>
                  </CardContent>
                </Card>

                <Card variant="glass">
                  <CardContent className="p-4">
                    <h4 className="text-sm font-medium text-gray-400 mb-1">Days Active</h4>
                    <p className="text-2xl font-bold text-white">
                      {Math.floor((Date.now() - new Date(position.entryDate).getTime()) / (1000 * 60 * 60 * 24))}
                    </p>
                  </CardContent>
                </Card>

                <Card variant="glass">
                  <CardContent className="p-4">
                    <h4 className="text-sm font-medium text-gray-400 mb-1">Current APY</h4>
                    <p className="text-2xl font-bold text-primary-400">
                      {position.apy.toFixed(2)}%
                    </p>
                  </CardContent>
                </Card>

                <Card variant="glass">
                  <CardContent className="p-4">
                    <h4 className="text-sm font-medium text-gray-400 mb-1">Daily Earnings</h4>
                    <p className="text-2xl font-bold text-green-400">
                      +{formatCurrency(earnings / Math.max(1, Math.floor((Date.now() - new Date(position.entryDate).getTime()) / (1000 * 60 * 60 * 24))))}
                    </p>
                  </CardContent>
                </Card>
              </div>

              <Card variant="glass">
                <CardHeader>
                  <CardTitle size="sm">Performance Summary</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="space-y-3 text-sm">
                    <div className="flex justify-between">
                      <span className="text-gray-400">Initial Deposit:</span>
                      <span className="text-white">{formatCurrency(position.amount)} {position.token}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-400">Current Value:</span>
                      <span className="text-white">{formatCurrency(position.realTimeValue)} {position.token}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-400">Total Earnings:</span>
                      <span className={isProfit ? 'text-green-400' : 'text-red-400'}>
                        {isProfit ? '+' : ''}{formatCurrency(earnings)} {position.token}
                      </span>
                    </div>
                    <div className="flex justify-between border-t border-gray-700 pt-3">
                      <span className="text-gray-400">Effective APY:</span>
                      <span className="text-primary-400 font-medium">{position.apy.toFixed(2)}%</span>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </motion.div>
          )}
        </div>
      </motion.div>
    </Modal>
  );
};

export default PositionManagementModal;