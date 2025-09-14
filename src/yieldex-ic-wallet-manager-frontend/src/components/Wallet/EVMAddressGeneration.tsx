import React, { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Key, Copy, Check, RefreshCw, ExternalLink } from 'lucide-react';
import { useWalletStore } from '@/stores/walletStore';
import { formatAddress } from '@/utils/formatters';
import { fadeVariants, pulseVariants } from '@/utils/animations';
import Button from '@/components/UI/Button';
import Card, { CardHeader, CardTitle, CardContent } from '@/components/UI/Card';

interface EVMAddressGenerationProps {
  autoGenerate?: boolean;
  onGenerated?: (address: string) => void;
}

const EVMAddressGeneration: React.FC<EVMAddressGenerationProps> = ({
  autoGenerate = false,
  onGenerated
}) => {
  const [copied, setCopied] = useState(false);
  const [showDetails, setShowDetails] = useState(false);
  
  const {
    evmAddress,
    isGeneratingAddress,
    connectionError,
    generateEvmAddress,
    clearError
  } = useWalletStore();

  useEffect(() => {
    if (autoGenerate && !evmAddress && !isGeneratingAddress) {
      handleGenerate();
    }
  }, [autoGenerate]);

  useEffect(() => {
    if (evmAddress && onGenerated) {
      onGenerated(evmAddress);
    }
  }, [evmAddress, onGenerated]);

  const handleGenerate = async () => {
    try {
      clearError();
      await generateEvmAddress();
    } catch (error) {
      console.error('Failed to generate EVM address:', error);
    }
  };

  const handleCopyAddress = async () => {
    if (!evmAddress) return;
    
    try {
      await navigator.clipboard.writeText(evmAddress);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error('Failed to copy address:', error);
    }
  };

  const openEtherscan = () => {
    if (evmAddress) {
      window.open(`https://etherscan.io/address/${evmAddress}`, '_blank');
    }
  };

  const generationSteps = [
    'Connecting to IC Threshold ECDSA...',
    'Generating cryptographic keys...',
    'Deriving EVM-compatible address...',
    'Validating address format...',
    'Address generation complete!'
  ];

  const [currentStep, setCurrentStep] = useState(0);

  useEffect(() => {
    if (isGeneratingAddress) {
      const interval = setInterval(() => {
        setCurrentStep((prev) => {
          if (prev < generationSteps.length - 1) {
            return prev + 1;
          }
          clearInterval(interval);
          return prev;
        });
      }, 600);

      return () => {
        clearInterval(interval);
        setCurrentStep(0);
      };
    }
  }, [isGeneratingAddress]);

  if (evmAddress) {
    return (
      <Card variant="glass">
        <CardHeader>
          <CardTitle size="md" className="flex items-center space-x-2">
            <Key size={20} className="text-primary-400" />
            <span>IC Threshold ECDSA Address</span>
          </CardTitle>
        </CardHeader>
        
        <CardContent>
          <div className="space-y-4">
            {/* Address Display */}
            <div className="p-4 bg-gray-900/50 rounded-lg border border-gray-700/50">
              <div className="flex items-center justify-between">
                <div className="flex-1">
                  <p className="text-xs text-gray-400 mb-1">EVM Address</p>
                  <p className="font-mono text-sm text-white break-all">
                    {evmAddress}
                  </p>
                </div>
                <div className="flex items-center space-x-2 ml-4">
                  <button
                    onClick={handleCopyAddress}
                    className="p-2 text-gray-400 hover:text-white transition-colors rounded-lg hover:bg-gray-700/50"
                    title="Copy address"
                  >
                    {copied ? <Check size={16} /> : <Copy size={16} />}
                  </button>
                  <button
                    onClick={openEtherscan}
                    className="p-2 text-gray-400 hover:text-white transition-colors rounded-lg hover:bg-gray-700/50"
                    title="View on Etherscan"
                  >
                    <ExternalLink size={16} />
                  </button>
                </div>
              </div>
            </div>

            {/* Address Info */}
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <p className="text-gray-400 mb-1">Short Format</p>
                <p className="text-white font-mono">{formatAddress(evmAddress)}</p>
              </div>
              <div>
                <p className="text-gray-400 mb-1">Network</p>
                <p className="text-white">Ethereum Compatible</p>
              </div>
            </div>

            {/* Additional Details */}
            <div className="pt-4 border-t border-gray-700/50">
              <button
                onClick={() => setShowDetails(!showDetails)}
                className="flex items-center space-x-2 text-sm text-primary-400 hover:text-primary-300 transition-colors"
              >
                <span>{showDetails ? 'Hide' : 'Show'} Details</span>
                <motion.div
                  animate={{ rotate: showDetails ? 180 : 0 }}
                  transition={{ duration: 0.2 }}
                >
                  ▼
                </motion.div>
              </button>

              <AnimatePresence>
                {showDetails && (
                  <motion.div
                    variants={fadeVariants}
                    initial="initial"
                    animate="animate"
                    exit="exit"
                    className="mt-3 p-3 bg-gray-800/30 rounded-lg"
                  >
                    <div className="text-xs text-gray-400 space-y-2">
                      <p>
                        <span className="font-medium">Generation Method:</span> IC Threshold ECDSA
                      </p>
                      <p>
                        <span className="font-medium">Security:</span> Distributed key generation with no single point of failure
                      </p>
                      <p>
                        <span className="font-medium">Compatibility:</span> EVM-compatible across all Ethereum-based networks
                      </p>
                      <p>
                        <span className="font-medium">Control:</span> Managed by Internet Computer smart contract
                      </p>
                    </div>
                  </motion.div>
                )}
              </AnimatePresence>
            </div>

            {/* Regenerate Option */}
            <div className="flex justify-center pt-2">
              <Button
                variant="ghost"
                size="sm"
                onClick={handleGenerate}
                leftIcon={<RefreshCw size={16} />}
                disabled={isGeneratingAddress}
              >
                Generate New Address
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card variant="glass">
      <CardHeader>
        <CardTitle size="md" className="flex items-center space-x-2">
          <Key size={20} className="text-primary-400" />
          <span>Generate EVM Address</span>
        </CardTitle>
      </CardHeader>
      
      <CardContent>
        <div className="space-y-6">
          {/* Description */}
          <div className="text-sm text-gray-400 space-y-2">
            <p>
              Generate a unique EVM-compatible address using Internet Computer's 
              Threshold ECDSA technology.
            </p>
            <div className="p-3 bg-blue-500/10 border border-blue-500/20 rounded-lg">
              <p className="text-blue-300 text-xs">
                🔐 <strong>Secure:</strong> Your address is generated using distributed 
                cryptography with no single point of failure.
              </p>
            </div>
          </div>

          {/* Generation Process */}
          {isGeneratingAddress && (
            <motion.div
              variants={fadeVariants}
              initial="initial"
              animate="animate"
              className="space-y-4"
            >
              <div className="text-center">
                <motion.div
                  variants={pulseVariants}
                  animate="animate"
                  className="w-16 h-16 bg-primary-500/20 rounded-full flex items-center justify-center mx-auto mb-4"
                >
                  <Key size={24} className="text-primary-400" />
                </motion.div>
                <p className="text-white font-medium">Generating Your EVM Address</p>
              </div>

              <div className="space-y-3">
                {generationSteps.map((step, index) => (
                  <motion.div
                    key={index}
                    className="flex items-center space-x-3"
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ 
                      opacity: index <= currentStep ? 1 : 0.3,
                      x: 0
                    }}
                    transition={{ delay: index * 0.1 }}
                  >
                    <div className={`w-2 h-2 rounded-full ${
                      index <= currentStep ? 'bg-primary-400' : 'bg-gray-600'
                    }`} />
                    <p className={`text-sm ${
                      index <= currentStep ? 'text-white' : 'text-gray-500'
                    }`}>
                      {step}
                    </p>
                  </motion.div>
                ))}
              </div>
            </motion.div>
          )}

          {/* Error Message */}
          <AnimatePresence>
            {connectionError && (
              <motion.div
                variants={fadeVariants}
                initial="initial"
                animate="animate"
                exit="exit"
                className="p-3 bg-red-500/10 border border-red-500/20 rounded-lg"
              >
                <p className="text-red-400 text-sm">{connectionError}</p>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Generate Button */}
          {!isGeneratingAddress && (
            <div className="flex justify-center">
              <Button
                onClick={handleGenerate}
                leftIcon={<Key size={20} />}
                loading={isGeneratingAddress}
                size="lg"
              >
                Generate EVM Address
              </Button>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
};

export default EVMAddressGeneration;