import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Wallet, ExternalLink, Copy, Check, AlertTriangle } from 'lucide-react';
import { useAccount, useConnect, useDisconnect, useChainId } from 'wagmi';
import { formatAddress } from '@/utils/formatters';
import { cardVariants, fadeVariants } from '@/utils/animations';
import Button from '@/components/UI/Button';
import Card from '@/components/UI/Card';
import Modal from '@/components/UI/Modal';
import { chains } from '@/wagmi';

interface RealWalletConnectProps {
  onConnected?: (address: string, chainId: number) => void;
  onDisconnected?: () => void;
  onConnectionSuccess?: () => void; // Called after successful connection
  showModal?: boolean; // External control of modal
  onModalClose?: () => void; // External modal close handler
}

const RealWalletConnect: React.FC<RealWalletConnectProps> = ({
  onConnected,
  onDisconnected,
  onConnectionSuccess,
  showModal: externalShowModal,
  onModalClose: externalOnModalClose
}) => {
  const [showModal, setShowModal] = useState(false);
  const [copied, setCopied] = useState(false);
  
  const { address, isConnected } = useAccount();
  const { connect, connectors, isPending, error } = useConnect();
  const { disconnect } = useDisconnect();
  const chainId = useChainId();
  
  // Get current chain info
  const chain = chains.find(c => c.id === chainId);

  // Notify parent component when connection changes
  useEffect(() => {
    if (isConnected && address && chain) {
      onConnected?.(address, chain.id);
      onConnectionSuccess?.(); // Call success callback
    } else if (!isConnected) {
      onDisconnected?.();
    }
  }, [isConnected, address, chain?.id]);

  const handleConnect = async (connector: typeof connectors[0]) => {
    console.log('Attempting to connect with:', {
      id: connector.id,
      name: connector.name,
      ready: connector.ready,
      type: connector.type
    });
    
    try {
      console.log('Calling connect...');
      const result = await connect({ connector });
      console.log('Connection successful:', result);

      // Close modal (either internal or external)
      if (externalOnModalClose) {
        externalOnModalClose();
      } else {
        setShowModal(false);
      }
    } catch (error) {
      console.error('Connection failed:', error);
    }
  };

  const handleDisconnect = async () => {
    try {
      await disconnect();
    } catch (error) {
      console.error('Disconnect failed:', error);
    }
  };

  const handleCopyAddress = async () => {
    if (!address) return;
    
    try {
      await navigator.clipboard.writeText(address);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error('Failed to copy address:', error);
    }
  };


  const getConnectorIcon = (connector: typeof connectors[0]) => {
    const name = connector.name.toLowerCase();
    
    // Check for specific wallets in injected
    if (name.includes('injected') && typeof window !== 'undefined' && window.ethereum?.isMetaMask) {
      return '🦊';
    }
    
    if (name.includes('injected') && typeof window !== 'undefined' && window.ethereum?.isTrust) {
      return '💙';
    }
    
    if (name.includes('walletconnect')) return '💙'; // Trust Wallet via WalletConnect
    if (name.includes('coinbase')) return '🔵';
    if (name.includes('injected')) return '👛';
    return '👛';
  };

  const getConnectorDisplayName = (connector: typeof connectors[0]) => {
    const name = connector.name.toLowerCase();
    
    // Check if MetaMask is available in window
    if (name.includes('injected') && typeof window !== 'undefined' && window.ethereum?.isMetaMask) {
      return 'MetaMask';
    }
    
    // Check if Trust Wallet is available
    if (name.includes('injected') && typeof window !== 'undefined' && window.ethereum?.isTrust) {
      return 'Trust Wallet';
    }
    
    if (name.includes('walletconnect')) return 'Trust Wallet'; // Mobile Trust Wallet
    if (name.includes('coinbase')) return 'Coinbase Wallet';
    if (name.includes('injected')) return 'Injected Wallet';
    return connector.name;
  };

  const getConnectorDescription = (connector: typeof connectors[0]) => {
    const name = connector.name.toLowerCase();
    
    // Check for specific wallets
    if (name.includes('injected') && typeof window !== 'undefined' && window.ethereum?.isMetaMask) {
      return 'Connect using MetaMask browser extension';
    }
    
    if (name.includes('injected') && typeof window !== 'undefined' && window.ethereum?.isTrust) {
      return 'Connect with Trust Wallet';
    }
    
    if (name.includes('walletconnect')) return 'Scan QR code with Trust Wallet mobile app';
    if (name.includes('coinbase')) return 'Connect with Coinbase Wallet';
    if (name.includes('injected')) return 'Connect with injected wallet';
    return 'Connect with wallet';
  };

  // Show all 3 connectors: Injected (MetaMask/Trust), WalletConnect (Mobile Trust), Coinbase
  const allowedConnectors = connectors; // Show all available connectors

  // Debug: Log connector states
  React.useEffect(() => {
    console.log('Available connectors:', connectors.map(c => ({
      id: c.id,
      name: c.name,
      ready: c.ready,
      type: c.type
    })));
  }, [connectors]);


  // Connected State
  if (isConnected && address) {
    return (
      <div className="flex items-center space-x-3">
        {/* Wallet Info */}
        <Card variant="glass" className="p-3">
          <div className="flex items-center space-x-3">
            <div className="w-8 h-8 bg-gradient-to-r from-primary-500 to-primary-600 rounded-full flex items-center justify-center">
              <Wallet size={16} className="text-white" />
            </div>
            <div className="flex items-center space-x-2">
              <span className="text-sm font-medium text-white">
                {formatAddress(address)}
              </span>
              <button
                onClick={handleCopyAddress}
                className="text-gray-400 hover:text-white transition-colors"
              >
                {copied ? <Check size={14} /> : <Copy size={14} />}
              </button>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={handleDisconnect}
              className="text-red-400 hover:text-red-300 hover:bg-red-400/10"
            >
              Disconnect
            </Button>
          </div>
        </Card>
      </div>
    );
  }

  // Disconnected State
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
          leftIcon={<Wallet size={20} />}
          loading={isPending}
        >
          {isPending ? 'Connecting...' : 'Connect Wallet'}
        </Button>
      </motion.div>

      {/* Connection Modal */}
      <Modal
        isOpen={externalShowModal !== undefined ? externalShowModal : showModal}
        onClose={externalOnModalClose || (() => setShowModal(false))}
        title="Connect Wallet"
        size="md"
      >
        <div className="space-y-4">
          <p className="text-gray-400 text-sm">
            Connect your wallet to access Your stable way to earn and start earning yield on your crypto.
          </p>

          {/* Error Message */}
          <AnimatePresence>
            {error && (
              <motion.div
                variants={fadeVariants}
                initial="initial"
                animate="animate"
                exit="exit"
                className="flex items-center space-x-2 p-3 bg-red-500/10 border border-red-500/20 rounded-lg"
              >
                <AlertTriangle size={16} className="text-red-400" />
                <span className="text-red-400 text-sm">{error.message}</span>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Wallet Options */}
          <div className="space-y-3">
            {allowedConnectors.map((connector, index) => (
              <motion.button
                key={connector.id}
                variants={cardVariants}
                initial="initial"
                animate="animate"
                whileHover="hover"
                whileTap="tap"
                custom={index}
                onClick={() => handleConnect(connector)}
                disabled={isPending}
                className={`w-full p-4 rounded-lg border text-left transition-all duration-200 ${
                  isPending
                    ? 'bg-gray-800/30 border-gray-700/30 cursor-not-allowed opacity-50'
                    : 'bg-gray-800/50 border-gray-700/50 hover:bg-gray-800/70 hover:border-gray-600/50 cursor-pointer'
                }`}
              >
                <div className="flex items-center space-x-3">
                  <div className="text-2xl">{getConnectorIcon(connector)}</div>
                  <div className="flex-1">
                    <h3 className="font-medium text-white">{getConnectorDisplayName(connector)}</h3>
                    <p className="text-sm text-gray-400">
                      {getConnectorDescription(connector)}
                    </p>
                  </div>
                  {!isPending && (
                    <ExternalLink size={16} className="text-gray-400" />
                  )}
                  {isPending && (
                    <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                  )}
                </div>
              </motion.button>
            ))}
          </div>

          {/* Info Notice */}
          <div className="mt-6 p-4 bg-blue-500/10 border border-blue-500/20 rounded-lg">
            <div className="flex items-start space-x-2">
              <div className="text-blue-400 mt-0.5">ℹ️</div>
              <div>
                <h4 className="text-blue-400 font-medium text-sm mb-1">Real Wallet Connection</h4>
                <p className="text-blue-300/80 text-xs">
                  This connects to your actual wallet. Make sure you're on the correct network 
                  and have sufficient funds for any transactions you plan to make.
                </p>
              </div>
            </div>
          </div>

          <div className="flex space-x-3 pt-4">
            <Button
              variant="ghost"
              onClick={externalOnModalClose || (() => setShowModal(false))}
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

export default RealWalletConnect;