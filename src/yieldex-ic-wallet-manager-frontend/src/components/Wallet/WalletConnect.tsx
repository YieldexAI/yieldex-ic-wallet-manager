import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Wallet, ExternalLink, Copy, Check, AlertTriangle } from 'lucide-react';
import { useWalletConnection } from '@/stores/walletStore';
import { formatAddress } from '@/utils/formatters';
import { cardVariants, fadeVariants } from '@/utils/animations';
import Button from '@/components/UI/Button';
import Card from '@/components/UI/Card';
import Modal from '@/components/UI/Modal';

interface WalletConnectProps {
  onConnected?: () => void;
}

const WalletConnect: React.FC<WalletConnectProps> = ({ onConnected }) => {
  const [showModal, setShowModal] = useState(false);
  const [copied, setCopied] = useState(false);
  const {
    isConnected,
    isConnecting,
    address,
    evmAddress,
    connectionError,
    connectWallet,
    disconnectWallet,
    clearError
  } = useWalletConnection();

  const handleConnect = async () => {
    try {
      await connectWallet();
      setShowModal(false);
      onConnected?.();
    } catch (error) {
      console.error('Connection failed:', error);
    }
  };

  const handleCopyAddress = async (addr: string) => {
    try {
      await navigator.clipboard.writeText(addr);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error('Failed to copy address:', error);
    }
  };

  const walletOptions = [
    {
      name: 'MetaMask',
      icon: '🦊',
      description: 'Connect using MetaMask browser extension',
      disabled: false
    },
    {
      name: 'WalletConnect',
      icon: '🔗',
      description: 'Connect with WalletConnect protocol',
      disabled: false
    },
    {
      name: 'Coinbase Wallet',
      icon: '🔵',
      description: 'Connect using Coinbase Wallet',
      disabled: true
    }
  ];

  if (isConnected) {
    return (
      <Card variant="glass" className="p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className="w-10 h-10 bg-gradient-to-r from-primary-500 to-primary-600 rounded-full flex items-center justify-center">
              <Wallet size={20} className="text-white" />
            </div>
            <div>
              <div className="flex items-center space-x-2">
                <span className="text-sm font-medium text-white">
                  {formatAddress(address)}
                </span>
                <button
                  onClick={() => handleCopyAddress(address)}
                  className="text-gray-400 hover:text-white transition-colors"
                >
                  {copied ? <Check size={14} /> : <Copy size={14} />}
                </button>
              </div>
              {evmAddress && (
                <div className="flex items-center space-x-2 mt-1">
                  <span className="text-xs text-gray-400">
                    IC EVM: {formatAddress(evmAddress)}
                  </span>
                  <button
                    onClick={() => handleCopyAddress(evmAddress)}
                    className="text-gray-400 hover:text-white transition-colors"
                  >
                    {copied ? <Check size={12} /> : <Copy size={12} />}
                  </button>
                </div>
              )}
            </div>
          </div>
          <Button
            variant="ghost"
            size="sm"
            onClick={disconnectWallet}
            className="text-red-400 hover:text-red-300 hover:bg-red-400/10"
          >
            Disconnect
          </Button>
        </div>
      </Card>
    );
  }

  return (
    <>
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.3 }}
      >
        <Button
          onClick={() => setShowModal(true)}
          size="lg"
          fullWidth
          leftIcon={<Wallet size={20} />}
          loading={isConnecting}
        >
          {isConnecting ? 'Connecting...' : 'Connect Wallet'}
        </Button>
      </motion.div>

      {/* Connection Modal */}
      <Modal
        isOpen={showModal}
        onClose={() => {
          setShowModal(false);
          clearError();
        }}
        title="Connect Wallet"
        size="md"
      >
        <div className="space-y-4">
          <p className="text-gray-400 text-sm">
            Connect your wallet to access Your stable way to earn and start earning yield on your stablecoins.
          </p>

          {/* Error Message */}
          <AnimatePresence>
            {connectionError && (
              <motion.div
                variants={fadeVariants}
                initial="initial"
                animate="animate"
                exit="exit"
                className="flex items-center space-x-2 p-3 bg-red-500/10 border border-red-500/20 rounded-lg"
              >
                <AlertTriangle size={16} className="text-red-400" />
                <span className="text-red-400 text-sm">{connectionError}</span>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Wallet Options */}
          <div className="space-y-3">
            {walletOptions.map((wallet, index) => (
              <motion.button
                key={wallet.name}
                variants={cardVariants}
                initial="initial"
                animate="animate"
                whileHover={!wallet.disabled ? "hover" : undefined}
                whileTap={!wallet.disabled ? "tap" : undefined}
                custom={index}
                onClick={!wallet.disabled ? handleConnect : undefined}
                disabled={wallet.disabled || isConnecting}
                className={`w-full p-4 rounded-lg border text-left transition-all duration-200 ${
                  wallet.disabled
                    ? 'bg-gray-800/30 border-gray-700/30 cursor-not-allowed opacity-50'
                    : 'bg-gray-800/50 border-gray-700/50 hover:bg-gray-800/70 hover:border-gray-600/50 cursor-pointer'
                }`}
              >
                <div className="flex items-center space-x-3">
                  <div className="text-2xl">{wallet.icon}</div>
                  <div className="flex-1">
                    <h3 className="font-medium text-white">{wallet.name}</h3>
                    <p className="text-sm text-gray-400">{wallet.description}</p>
                  </div>
                  {!wallet.disabled && (
                    <ExternalLink size={16} className="text-gray-400" />
                  )}
                </div>
              </motion.button>
            ))}
          </div>

          {/* Demo Notice */}
          <div className="mt-6 p-4 bg-blue-500/10 border border-blue-500/20 rounded-lg">
            <div className="flex items-start space-x-2">
              <div className="text-blue-400 mt-0.5">ℹ️</div>
              <div>
                <h4 className="text-blue-400 font-medium text-sm mb-1">Demo Mode</h4>
                <p className="text-blue-300/80 text-xs">
                  This is a demo interface. The wallet connection is simulated and will generate 
                  mock data for demonstration purposes. No real transactions will be executed.
                </p>
              </div>
            </div>
          </div>

          <div className="flex space-x-3 pt-4">
            <Button
              variant="ghost"
              onClick={() => {
                setShowModal(false);
                clearError();
              }}
              fullWidth
            >
              Cancel
            </Button>
          </div>
        </div>
      </Modal>
    </>
  );
};

export default WalletConnect;