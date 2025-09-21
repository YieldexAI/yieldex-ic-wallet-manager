import React from 'react';
import { motion } from 'framer-motion';
import StablecoinCard from './StablecoinCard';
import { ProcessedTokenBalance } from '@/services/types';
import { STABLECOINS } from '@/services/constants';

interface StablecoinGridProps {
  balances: ProcessedTokenBalance[];
  isLoading?: boolean;
  error?: string | null;
}

const StablecoinGrid: React.FC<StablecoinGridProps> = ({
  balances,
  isLoading = false,
  error = null
}) => {
  // Group balances by token symbol
  const groupedBalances = balances.reduce((acc, balance) => {
    const symbol = balance.symbol;
    if (!acc[symbol]) {
      acc[symbol] = {
        totalBalance: 0,
        networks: [],
        details: []
      };
    }

    acc[symbol].totalBalance += parseFloat(balance.balance);
    acc[symbol].networks.push({
      network: balance.network,
      balance: balance.balance,
      usdValue: parseFloat(balance.balance) // 1:1 for stablecoins
    });
    acc[symbol].details.push(balance);

    return acc;
  }, {} as Record<string, {
    totalBalance: number;
    networks: Array<{ network: string; balance: string; usdValue: number }>;
    details: ProcessedTokenBalance[];
  }>);

  // Create cards for all supported stablecoins, showing 0 if no balance
  const stablecoinCards = Object.entries(STABLECOINS).map(([symbol, config]) => {
    const balanceData = groupedBalances[symbol];
    const totalBalance = balanceData?.totalBalance || 0;
    const networkBalances = balanceData?.networks || [];

    return {
      symbol,
      name: config.name,
      totalBalance: totalBalance.toFixed(6),
      usdValue: totalBalance,
      networkBalances,
      logo: config.logo
    };
  }).sort((a, b) => b.usdValue - a.usdValue); // Sort by USD value

  if (isLoading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
        {[...Array(4)].map((_, index) => (
          <motion.div
            key={index}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: index * 0.1 }}
            className="bg-gray-800/30 border border-gray-700/50 rounded-xl p-6 animate-pulse"
          >
            <div className="flex items-center space-x-3 mb-4">
              <div className="w-8 h-8 bg-gray-700 rounded-full"></div>
              <div>
                <div className="h-4 bg-gray-700 rounded w-16 mb-2"></div>
                <div className="h-3 bg-gray-700 rounded w-24"></div>
              </div>
            </div>
            <div className="h-12 bg-gray-700/30 rounded-lg mb-4"></div>
            <div className="h-8 bg-gray-700 rounded"></div>
          </motion.div>
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center py-8">
        <div className="text-red-400 mb-2">Failed to load stablecoin balances</div>
        <div className="text-gray-400 text-sm">{error}</div>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
      {stablecoinCards.map((card, index) => (
        <motion.div
          key={card.symbol}
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: index * 0.1 }}
        >
          <StablecoinCard
            symbol={card.symbol}
            name={card.name}
            totalBalance={card.totalBalance}
            usdValue={card.usdValue}
            networkBalances={card.networkBalances}
            logo={card.logo}
          />
        </motion.div>
      ))}
    </div>
  );
};

export default StablecoinGrid;