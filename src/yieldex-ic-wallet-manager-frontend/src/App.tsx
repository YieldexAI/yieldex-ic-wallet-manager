import { useState, useEffect, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ArrowRight, ChevronLeft } from 'lucide-react';
import { useWalletConnection } from '@/stores/walletStore';
import { useRealTimeSimulation } from '@/stores/strategyStore';
import { useWalletIntegration } from '@/hooks/useWalletIntegration';
import { pageVariants, fadeVariants } from '@/utils/animations';
import Layout, { Header, Main, Section, Container } from '@/components/UI/Layout';
import Button from '@/components/UI/Button';
import RealWalletConnect from '@/components/Wallet/RealWalletConnect';
import EVMAddressGeneration from '@/components/Wallet/EVMAddressGeneration';
import StrategySelector from '@/components/Strategies/StrategySelector';
import Portfolio from '@/components/Dashboard/Portfolio';
import { ToastProvider } from '@/contexts/ToastContext';

type DemoStep = 'landing' | 'address' | 'strategies' | 'dashboard';

function App() {
  const [currentStep, setCurrentStep] = useState<DemoStep>('landing');
  const { evmAddress } = useWalletConnection();
  const { startRealTimeSimulation } = useRealTimeSimulation();
  
  // Bridge real wallet with our demo store
  const { realIsConnected } = useWalletIntegration();

  // Removed auto-advance demo steps for manual user control

  // Start simulation when app loads
  useEffect(() => {
    startRealTimeSimulation();
  }, []);

  const steps = [
    { id: 'landing', title: 'Connect Wallet', description: 'Connect your wallet to get started' },
    { id: 'address', title: 'Generate Address', description: 'Create your IC threshold ECDSA address' },
    { id: 'strategies', title: 'Choose Strategy', description: 'Select a yield strategy' },
    { id: 'dashboard', title: 'Dashboard', description: 'Monitor your positions' }
  ];

  const currentStepIndex = steps.findIndex(step => step.id === currentStep);

  const nextStep = () => {
    const nextIndex = Math.min(currentStepIndex + 1, steps.length - 1);
    setCurrentStep(steps[nextIndex].id as DemoStep);
  };

  const prevStep = () => {
    const prevIndex = Math.max(currentStepIndex - 1, 0);
    setCurrentStep(steps[prevIndex].id as DemoStep);
  };

  const goToStep = (step: DemoStep) => {
    setCurrentStep(step);
  };

  const handleWalletConnected = useCallback((address: string, chainId: number) => {
    console.log('Wallet connected:', { address, chainId });
    // Manual control - user clicks Continue button to proceed
  }, []);

  const handleWalletDisconnected = useCallback(() => {
    console.log('Wallet disconnected');
    setCurrentStep('landing');
  }, []);

  return (
    <ToastProvider>
      <Layout maxWidth="xl">
        <motion.div
          variants={pageVariants}
          initial="initial"
          animate="enter"
          className="space-y-8"
        >
        {/* Header */}
        <Header>
          <Container>
            {/* Top Navigation Bar */}
            <div className="flex items-center justify-between mb-8 -mx-6">
              <button 
                onClick={() => goToStep('landing')}
                className="flex items-center space-x-2 hover:opacity-80 transition-opacity cursor-pointer pl-6"
              >
                <img 
                  src="/logo.png" 
                  alt="Yieldex Logo" 
                  className="w-8 h-8 object-contain"
                />
                <span className="text-white font-semibold text-lg">Yieldex</span>
              </button>
              
              {/* Connect Wallet Button - Industry Standard Position */}
              <div className="flex items-center space-x-4 pr-6">
                <RealWalletConnect 
                  onConnected={handleWalletConnected}
                  onDisconnected={handleWalletDisconnected}
                />
              </div>
            </div>

            {/* Main Title and Description */}
            <div className="text-center space-y-4">
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
        </Header>

        {/* Progress Steps */}
        <Section>
          <Container size="lg">
            <div className="mb-8">
              {/* Mobile: Vertical Steps */}
              <div className="block md:hidden">
                <div className="flex items-center justify-center mb-4">
                  <div className="text-sm text-gray-400">
                    Step {currentStepIndex + 1} of {steps.length}
                  </div>
                </div>
                <div className="bg-gray-800/50 rounded-lg p-4 text-center">
                  <div className="flex items-center justify-center space-x-3 mb-2">
                    <div className="w-8 h-8 rounded-full bg-primary-500 text-white flex items-center justify-center text-sm font-semibold">
                      {currentStepIndex + 1}
                    </div>
                    <div>
                      <div className="font-medium text-white">{steps[currentStepIndex].title}</div>
                      <div className="text-xs text-gray-400">{steps[currentStepIndex].description}</div>
                    </div>
                  </div>
                </div>
              </div>

              {/* Desktop: Horizontal Steps */}
              <div className="hidden md:block">
                <div className="flex items-center justify-between overflow-x-auto scrollbar-hide pb-4">
                  {steps.map((step, index) => (
                    <div key={step.id} className="flex items-center flex-shrink-0">
                      <button
                        onClick={() => goToStep(step.id as DemoStep)}
                        className={`flex items-center space-x-3 px-4 py-3 rounded-lg transition-all whitespace-nowrap ${
                          index <= currentStepIndex
                            ? 'bg-primary-500/20 text-primary-300 border border-primary-500/30'
                            : 'bg-gray-800/50 text-gray-500 border border-gray-700/50 hover:bg-gray-800/70'
                        }`}
                        disabled={!realIsConnected && step.id !== 'landing'}
                      >
                        <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-semibold ${
                          index <= currentStepIndex ? 'bg-primary-500 text-white' : 'bg-gray-600 text-gray-400'
                        }`}>
                          {index + 1}
                        </div>
                        <div className="text-left">
                          <div className="font-medium">{step.title}</div>
                          <div className="text-xs opacity-70">{step.description}</div>
                        </div>
                      </button>
                      {index < steps.length - 1 && (
                        <ArrowRight size={16} className="text-gray-600 mx-3 flex-shrink-0" />
                      )}
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </Container>
        </Section>

        {/* Main Content */}
        <Main>
          <Container>
            <AnimatePresence mode="wait">
              {currentStep === 'landing' && (
                <motion.div
                  key="landing"
                  variants={fadeVariants}
                  initial="initial"
                  animate="animate"
                  exit="exit"
                >
                  <Section 
                    title="Welcome to Yieldex" 
                    description="The next-generation DeFi wallet powered by Internet Computer's revolutionary technology"
                  >
                    <div className="max-w-2xl mx-auto space-y-8">
                      <div className="grid md:grid-cols-3 gap-6">
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
                      </div>
                      
                      <div className="text-center">
                        <p className="text-gray-400 mb-4">
                          Connect your wallet to get started with institutional-grade DeFi operations
                        </p>
                        <p className="text-sm text-gray-500">
                          Demo mode - No real transactions will be executed
                        </p>
                      </div>
                    </div>
                  </Section>
                </motion.div>
              )}

              {currentStep === 'address' && (
                <motion.div
                  key="address"
                  variants={fadeVariants}
                  initial="initial"
                  animate="animate"
                  exit="exit"
                >
                  <Section 
                    title="IC Threshold ECDSA" 
                    description="Generate your unique EVM address using Internet Computer's secure threshold cryptography"
                  >
                    <div className="max-w-2xl mx-auto space-y-6">
                      <EVMAddressGeneration 
                        autoGenerate={true} 
                        onGenerated={() => {}}
                      />
                      
                      <div className="flex justify-between">
                        <Button variant="ghost" onClick={prevStep} leftIcon={<ChevronLeft size={16} />}>
                          Back
                        </Button>
                        <Button 
                          onClick={nextStep} 
                          rightIcon={<ArrowRight size={16} />}
                          disabled={!evmAddress}
                        >
                          Continue
                        </Button>
                      </div>
                    </div>
                  </Section>
                </motion.div>
              )}

              {currentStep === 'strategies' && (
                <motion.div
                  key="strategies"
                  variants={fadeVariants}
                  initial="initial"
                  animate="animate"
                  exit="exit"
                >
                  <div className="space-y-6">
                    <StrategySelector />
                    
                    <div className="flex justify-between max-w-4xl mx-auto">
                      <Button variant="ghost" onClick={prevStep} leftIcon={<ChevronLeft size={16} />}>
                        Back
                      </Button>
                      <Button onClick={nextStep} rightIcon={<ArrowRight size={16} />}>
                        View Dashboard
                      </Button>
                    </div>
                  </div>
                </motion.div>
              )}

              {currentStep === 'dashboard' && (
                <motion.div
                  key="dashboard"
                  variants={fadeVariants}
                  initial="initial"
                  animate="animate"
                  exit="exit"
                  className="space-y-8"
                >
                  <div className="flex justify-between items-center">
                    <h2 className="text-2xl font-bold text-white">Dashboard</h2>
                    <Button variant="ghost" onClick={prevStep} leftIcon={<ChevronLeft size={16} />}>
                      Back
                    </Button>
                  </div>

                  <Portfolio />
                </motion.div>
              )}
            </AnimatePresence>
          </Container>
        </Main>

        {/* Footer */}
        <footer className="text-center py-8 border-t border-gray-800">
          <Container>
            <p className="text-gray-500 text-sm">
              Built with ❤️ on Internet Computer • Demo Interface
            </p>
            <div className="flex justify-center space-x-4 mt-4 text-gray-600">
              <a href="https://internetcomputer.org" className="hover:text-primary-400 transition-colors">
                IC Docs
              </a>
              <span>•</span>
              <a href="https://github.com" className="hover:text-primary-400 transition-colors">
                GitHub
              </a>
              <span>•</span>
              <a href="https://twitter.com" className="hover:text-primary-400 transition-colors">
                Twitter
              </a>
            </div>
          </Container>
        </footer>
        </motion.div>
      </Layout>
    </ToastProvider>
  );
}

export default App;