import React, { useState, useEffect, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { DollarSign, TrendingUp, AlertTriangle, CheckCircle } from 'lucide-react';
import { Strategy } from '@/mock/strategies';
import { useWalletBalances } from '@/stores/walletStore';
import { useUserPositions } from '@/stores/strategyStore';
import { formatCurrency, formatAPY, isValidAmount } from '@/utils/formatters';
import { fadeVariants, statusVariants } from '@/utils/animations';
import Modal, { ModalFooter } from '@/components/UI/Modal';
import Button from '@/components/UI/Button';
import { TokenAmountInput } from '@/components/UI/Input';

interface DepositModalProps {
  isOpen: boolean;
  onClose: () => void;
  strategy: Strategy;
  existingPosition?: {
    id: string;
    amount: number;
    token: string;
    realTimeValue: number;
  } | null;
}

const DepositModal: React.FC<DepositModalProps> = ({
  isOpen,
  onClose,
  strategy,
  existingPosition = null
}) => {
  const [amount, setAmount] = useState('');
  const [selectedToken, setSelectedToken] = useState<string>('USDC');
  const [step, setStep] = useState<'input' | 'confirm' | 'processing' | 'success'>('input');
  const [error, setError] = useState<string | null>(null);

  const {
    getTokenBalance,
    isLoadingBalances,
    balancesError,
    fetchRealBalances,
    stablecoinBalances
  } = useWalletBalances();
  const { createPosition, addToPosition, isDepositing } = useUserPositions();

  // Get available balance for selected token
  const availableBalance = getTokenBalance(selectedToken);

  // Filter tokens to only show those with positive balance (using stablecoinBalances directly)
  const availableTokens = useMemo((): string[] => {
    if (existingPosition) {
      // For existing position, only allow the same token
      return [existingPosition.token];
    }

    // If we have real balance data, use it
    if (stablecoinBalances.length > 0) {
      const tokensWithBalance = strategy.supportedTokens.filter(token => {
        return stablecoinBalances.some(balance =>
          balance.symbol === token && parseFloat(balance.balance) > 0
        );
      });
      return tokensWithBalance.length > 0 ? tokensWithBalance : strategy.supportedTokens;
    }

    // Fallback: show all supported tokens if no real data yet
    return strategy.supportedTokens;
  }, [existingPosition, strategy.supportedTokens, stablecoinBalances]);

  // Reset state when modal opens/closes
  useEffect(() => {
    if (isOpen) {
      setStep('input');
      setAmount('');
      setError(null);
      // Set token to existing position's token if adding to existing position
      if (existingPosition) {
        setSelectedToken(existingPosition.token);
      } else {
        // Select the first available token with positive balance
        const firstAvailableToken = availableTokens[0] || 'USDC';
        setSelectedToken(firstAvailableToken);
      }
      // Refresh real balances when modal opens to ensure latest data
      fetchRealBalances();
    }
  }, [isOpen, existingPosition, fetchRealBalances]);

  // Separate effect to handle token selection when available tokens change
  useEffect(() => {
    if (isOpen && !existingPosition && availableTokens.length > 0) {
      // Only update if current token is not in available tokens
      if (!availableTokens.includes(selectedToken)) {
        setSelectedToken(availableTokens[0]);
      }
    }
  }, [availableTokens, isOpen, existingPosition, selectedToken]);

  // Clear error when token changes
  useEffect(() => {
    if (error) {
      setError(null);
    }
  }, [selectedToken]);

  const validateAmount = (): string | null => {
    if (!amount || !isValidAmount(amount)) {
      return 'Please enter a valid amount';
    }

    const numAmount = parseFloat(amount);

    if (numAmount > availableBalance) {
      return `Insufficient ${selectedToken} balance`;
    }

    return null;
  };

  const handleContinue = () => {
    const validationError = validateAmount();
    if (validationError) {
      setError(validationError);
      return;
    }

    setError(null);
    setStep('confirm');
  };

  const handleDeposit = async () => {
    try {
      setStep('processing');
      
      if (existingPosition) {
        await addToPosition(existingPosition.id, parseFloat(amount), selectedToken);
      } else {
        await createPosition(strategy.id, parseFloat(amount), selectedToken);
      }
      
      setStep('success');
      
      // Auto close after success
      setTimeout(() => {
        onClose();
      }, 2000);
      
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Deposit failed');
      setStep('input');
    }
  };

  const handleBack = () => {
    setStep('input');
    setError(null);
  };

  const handleClose = () => {
    if (step === 'processing') return; // Prevent closing during processing
    onClose();
  };

  const estimatedEarnings = amount && isValidAmount(amount) 
    ? (parseFloat(amount) * strategy.expectedApy / 100) 
    : 0;

  const estimatedMonthlyEarnings = estimatedEarnings / 12;

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleClose}
      title={existingPosition ? "Add to Position" : "Deposit to Strategy"}
      size="md"
      closeOnBackdrop={step !== 'processing'}
      showCloseButton={step !== 'processing'}
    >
      <AnimatePresence mode="wait">
        {/* Input Step */}
        {step === 'input' && (
          <motion.div
            key="input"
            variants={fadeVariants}
            initial="initial"
            animate="animate"
            exit="exit"
            className="space-y-6"
          >
            {/* Strategy Info */}
            <div className="p-4 bg-primary-500/10 border border-primary-500/20 rounded-lg">
              <div className="flex items-center justify-between mb-2">
                <h3 className="font-semibold text-white">{strategy.name}</h3>
                <span className="text-primary-400 font-bold text-lg">
                  {formatAPY(strategy.expectedApy)}
                </span>
              </div>
              <p className="text-sm text-gray-400">{strategy.description}</p>
              
              {/* Show existing position info if adding to existing */}
              {existingPosition && (
                <div className="mt-3 pt-3 border-t border-primary-500/20">
                  <div className="text-xs text-gray-400 mb-1">Current Position</div>
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-300">Amount:</span>
                    <span className="text-white">{formatCurrency(existingPosition.amount)} {existingPosition.token}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-300">Value:</span>
                    <span className="text-white">{formatCurrency(existingPosition.realTimeValue)}</span>
                  </div>
                </div>
              )}
            </div>

            {/* Amount Input */}
            <div className="space-y-4">
              <div>
                <TokenAmountInput
                  label={existingPosition ? "Additional Amount" : "Deposit Amount"}
                  value={amount}
                  onChange={setAmount}
                  selectedToken={selectedToken}
                  onTokenChange={existingPosition ? () => {} : setSelectedToken}
                  availableTokens={availableTokens}
                  balance={availableBalance}
                  error={error || undefined}
                  placeholder="0.0"
                />

                {/* Balance Loading State */}
                {isLoadingBalances && (
                  <div className="flex items-center text-xs text-gray-400 mt-1">
                    <div className="w-3 h-3 border border-primary-500/30 border-t-primary-500 rounded-full animate-spin mr-2" />
                    Loading real-time balance...
                  </div>
                )}

                {/* Balance Error State */}
                {balancesError && (
                  <div className="flex items-center justify-between text-xs text-red-400 mt-1">
                    <span>Error loading balance: {balancesError}</span>
                    <button
                      onClick={fetchRealBalances}
                      className="text-primary-400 hover:text-primary-300 underline"
                    >
                      Retry
                    </button>
                  </div>
                )}

                {/* No Tokens Available Warning */}
                {!isLoadingBalances && !balancesError && availableBalance === 0 && (
                  <div className="flex items-center text-xs text-yellow-400 mt-1">
                    <span>⚠️ No {selectedToken} balance available. Please add funds to your wallet first.</span>
                  </div>
                )}
              </div>

            </div>

            {/* Earnings Estimate */}
            {amount && isValidAmount(amount) && !error && (
              <motion.div
                variants={fadeVariants}
                initial="initial"
                animate="animate"
                className="p-4 bg-green-500/10 border border-green-500/20 rounded-lg"
              >
                <h4 className="font-medium text-green-400 mb-2">Estimated Earnings</h4>
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <p className="text-gray-400">Monthly</p>
                    <p className="text-white font-semibold">
                      {formatCurrency(estimatedMonthlyEarnings)}
                    </p>
                  </div>
                  <div>
                    <p className="text-gray-400">Yearly</p>
                    <p className="text-white font-semibold">
                      {formatCurrency(estimatedEarnings)}
                    </p>
                  </div>
                </div>
              </motion.div>
            )}
          </motion.div>
        )}

        {/* Confirmation Step */}
        {step === 'confirm' && (
          <motion.div
            key="confirm"
            variants={fadeVariants}
            initial="initial"
            animate="animate"
            exit="exit"
            className="space-y-6"
          >
            <div className="text-center">
              <h3 className="text-lg font-semibold text-white mb-2">
                {existingPosition ? "Confirm Addition" : "Confirm Deposit"}
              </h3>
              <p className="text-gray-400">
                {existingPosition 
                  ? "Review the additional amount before proceeding" 
                  : "Review your deposit details before proceeding"
                }
              </p>
            </div>

            {/* Deposit Summary */}
            <div className="space-y-4 p-4 bg-gray-800/50 rounded-lg">
              <div className="flex justify-between">
                <span className="text-gray-400">Strategy</span>
                <span className="text-white font-medium">{strategy.name}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400">Amount</span>
                <span className="text-white font-medium">{amount} {selectedToken}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400">USD Value</span>
                <span className="text-white font-medium">
                  {formatCurrency(parseFloat(amount))}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400">Expected APY</span>
                <span className="text-primary-400 font-semibold">
                  {formatAPY(strategy.expectedApy)}
                </span>
              </div>
              <div className="pt-2 border-t border-gray-700">
                <div className="flex justify-between">
                  <span className="text-gray-400">Est. Yearly Earnings</span>
                  <span className="text-green-400 font-semibold">
                    {formatCurrency(estimatedEarnings)}
                  </span>
                </div>
              </div>
            </div>

            {/* Risks Warning */}
            <div className="p-3 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
              <div className="flex items-start space-x-2">
                <AlertTriangle size={16} className="text-yellow-400 mt-0.5 flex-shrink-0" />
                <div className="text-xs text-yellow-300">
                  <p className="font-medium mb-1">Risk Disclosure</p>
                  <p>
                    DeFi investments carry risks including smart contract vulnerabilities, 
                    market volatility, and potential loss of principal. Past performance 
                    does not guarantee future results.
                  </p>
                </div>
              </div>
            </div>
          </motion.div>
        )}

        {/* Processing Step */}
        {step === 'processing' && (
          <motion.div
            key="processing"
            variants={fadeVariants}
            initial="initial"
            animate="animate"
            exit="exit"
            className="text-center py-8"
          >
            <motion.div
              animate={{ rotate: 360 }}
              transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
              className="w-16 h-16 border-4 border-primary-500/30 border-t-primary-500 rounded-full mx-auto mb-4"
            />
            <h3 className="text-lg font-semibold text-white mb-2">
              {existingPosition ? "Processing Addition" : "Processing Deposit"}
            </h3>
            <p className="text-gray-400 mb-4">
              Please wait while we {existingPosition ? "add funds to your position in" : "process your deposit to"} {strategy.name}
            </p>
            <div className="text-sm text-gray-500">
              This usually takes 1-2 minutes...
            </div>
          </motion.div>
        )}

        {/* Success Step */}
        {step === 'success' && (
          <motion.div
            key="success"
            variants={statusVariants}
            animate="success"
            className="text-center py-8"
          >
            <CheckCircle size={64} className="text-green-400 mx-auto mb-4" />
            <h3 className="text-lg font-semibold text-white mb-2">
              {existingPosition ? "Addition Successful!" : "Deposit Successful!"}
            </h3>
            <p className="text-gray-400 mb-4">
              Your funds have been successfully {existingPosition ? "added to your position in" : "deposited to"} {strategy.name}
            </p>
            <div className="p-3 bg-green-500/10 border border-green-500/20 rounded-lg text-sm">
              <p className="text-green-400 font-medium">
                You've {existingPosition ? "added" : "deposited"} {amount} {selectedToken}
              </p>
              <p className="text-gray-400 mt-1">
                {existingPosition ? "Your position has been increased!" : "You'll start earning yield immediately!"}
              </p>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Modal Footer */}
      <ModalFooter>
        <AnimatePresence mode="wait">
          {step === 'input' && (
            <motion.div
              key="input-footer"
              variants={fadeVariants}
              initial="initial"
              animate="animate"
              exit="exit"
              className="flex space-x-3 w-full"
            >
              <Button variant="ghost" onClick={handleClose} fullWidth>
                Cancel
              </Button>
              <Button
                onClick={handleContinue}
                disabled={!amount || !!error}
                fullWidth
                rightIcon={<TrendingUp size={16} />}
              >
                Continue
              </Button>
            </motion.div>
          )}

          {step === 'confirm' && (
            <motion.div
              key="confirm-footer"
              variants={fadeVariants}
              initial="initial"
              animate="animate"
              exit="exit"
              className="flex space-x-3 w-full"
            >
              <Button variant="ghost" onClick={handleBack} fullWidth>
                Back
              </Button>
              <Button
                onClick={handleDeposit}
                loading={isDepositing}
                fullWidth
                leftIcon={<DollarSign size={16} />}
              >
                Deposit Now
              </Button>
            </motion.div>
          )}

          {step === 'success' && (
            <motion.div
              key="success-footer"
              variants={fadeVariants}
              initial="initial"
              animate="animate"
              exit="exit"
              className="w-full"
            >
              <Button onClick={handleClose} fullWidth variant="primary">
                Done
              </Button>
            </motion.div>
          )}
        </AnimatePresence>
      </ModalFooter>
    </Modal>
  );
};

export default DepositModal;