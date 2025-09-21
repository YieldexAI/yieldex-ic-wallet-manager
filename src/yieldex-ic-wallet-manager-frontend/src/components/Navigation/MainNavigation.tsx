import React from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { motion } from 'framer-motion';
import { useWalletConnection, useStablecoinBalances } from '@/stores/walletStore';
import { useWalletIntegration } from '@/hooks/useWalletIntegration';
import RealWalletConnect from '@/components/Wallet/RealWalletConnect';
import Button from '@/components/UI/Button';
import { clsx } from 'clsx';

const MainNavigation: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const { isConnected } = useWalletConnection();
  const { realIsConnected } = useWalletIntegration();
  const { fetchRealBalances } = useStablecoinBalances();

  const handleWalletConnected = async (address: string, chainId: number) => {
    console.log('Wallet connected:', { address, chainId });
    // Fetch stablecoin balances immediately after connection
    await fetchRealBalances();
  };

  const handleWalletDisconnected = () => {
    console.log('Wallet disconnected');
    navigate('/');
  };

  const navigationItems = [
    { path: '/', label: 'Overview' },
    { path: '/strategies', label: 'Strategies', requiresWallet: false },
    { path: '/dashboard', label: 'Dashboard', requiresWallet: false }
  ];

  const currentPath = location.pathname;

  return (
    <motion.header
      initial={{ opacity: 0, y: -20 }}
      animate={{ opacity: 1, y: 0 }}
      className="border-b border-gray-800 bg-gray-900/80 backdrop-blur-xl sticky top-0 z-50"
    >
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16">
          {/* Logo */}
          <button
            onClick={() => navigate('/')}
            className="flex items-center space-x-2 hover:opacity-80 transition-opacity cursor-pointer"
          >
            <img
              src="/logo.png"
              alt="Yieldex Logo"
              className="w-8 h-8 object-contain"
            />
            <span className="text-white font-semibold text-lg">Yieldex</span>
          </button>

          {/* Navigation Links */}
          <nav className="hidden md:flex items-center space-x-1">
            {navigationItems.map((item) => {
              const isActive = currentPath === item.path;
              const isDisabled = item.requiresWallet && !realIsConnected;

              return (
                <Button
                  key={item.path}
                  variant={isActive ? 'primary' : 'ghost'}
                  size="sm"
                  onClick={() => navigate(item.path)}
                  disabled={isDisabled}
                  className={clsx(
                    'transition-all duration-200',
                    isActive && 'bg-primary-500/20 text-primary-300',
                    !isActive && !isDisabled && 'hover:bg-gray-800/50'
                  )}
                >
                  {item.label}
                </Button>
              );
            })}
          </nav>

          {/* Wallet Connection */}
          <div className="flex items-center space-x-4">
            <RealWalletConnect
              onConnected={handleWalletConnected}
              onDisconnected={handleWalletDisconnected}
              onConnectionSuccess={() => fetchRealBalances()}
            />
          </div>
        </div>

        {/* Mobile Navigation */}
        <div className="md:hidden pb-4">
          <nav className="flex items-center justify-center space-x-1">
            {navigationItems.map((item) => {
              const isActive = currentPath === item.path;
              const isDisabled = item.requiresWallet && !realIsConnected;

              return (
                <Button
                  key={item.path}
                  variant={isActive ? 'primary' : 'ghost'}
                  size="sm"
                  onClick={() => navigate(item.path)}
                  disabled={isDisabled}
                  className={clsx(
                    'transition-all duration-200 text-xs',
                    isActive && 'bg-primary-500/20 text-primary-300'
                  )}
                >
                  {item.label}
                </Button>
              );
            })}
          </nav>
        </div>
      </div>
    </motion.header>
  );
};

export default MainNavigation;