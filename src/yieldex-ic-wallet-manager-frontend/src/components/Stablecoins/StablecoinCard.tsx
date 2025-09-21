import React from 'react';
import { motion } from 'framer-motion';
import { ArrowRight, TrendingUp } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import Button from '@/components/UI/Button';
import { ProcessedTokenBalance } from '@/services/types';

interface StablecoinCardProps {
  symbol: string;
  name: string;
  totalBalance: string;
  usdValue: number;
  networkBalances: Array<{
    network: string;
    balance: string;
    usdValue: number;
  }>;
  logo?: string;
  className?: string;
}

// Token icons component (you can replace with actual SVG icons later)
const TokenIcon: React.FC<{ symbol: string; className?: string }> = ({ symbol, className = "w-8 h-8" }) => {
  const icons: Record<string, string> = {
    USDT: '💚', // Green circle for USDT
    USDC: '🔵', // Blue circle for USDC
    USDe: '⚫', // Black circle for USDe
    DAI: '🟡'  // Yellow circle for DAI
  };

  return (
    <div className={`${className} flex items-center justify-center text-2xl bg-gray-700/50 rounded-full`}>
      {icons[symbol] || '💰'}
    </div>
  );
};

const StablecoinCard: React.FC<StablecoinCardProps> = ({
  symbol,
  name,
  totalBalance,
  usdValue,
  networkBalances,
  logo,
  className = ""
}) => {
  const navigate = useNavigate();

  const handleStartEarn = () => {
    navigate('/strategies', {
      state: {
        selectedToken: symbol,
        balance: totalBalance
      }
    });
  };

  const formatBalance = (balance: string): string => {
    const num = parseFloat(balance);
    if (num >= 1000000) {
      return `${(num / 1000000).toFixed(2)}M`;
    } else if (num >= 1000) {
      return `${(num / 1000).toFixed(2)}K`;
    }
    return num.toFixed(2);
  };

  const formatUSD = (amount: number): string => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: 2
    }).format(amount);
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      whileHover={{ y: -2 }}
      className={`
        bg-gradient-to-br from-gray-800/50 to-gray-900/50
        border border-gray-700/50 rounded-xl p-6
        hover:border-primary-500/30 transition-all duration-300
        backdrop-blur-sm
        ${className}
      `}
    >
      {/* Header */}
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center space-x-3">
          <TokenIcon symbol={symbol} />
          <div>
            <h3 className="font-semibold text-white text-lg">{symbol}</h3>
            <p className="text-sm text-gray-400">{name}</p>
          </div>
        </div>

        <div className="text-right">
          <div className="text-sm text-gray-400">Total Balance</div>
          <div className="font-mono text-white text-lg">
            {formatBalance(totalBalance)}
          </div>
        </div>
      </div>

      {/* USD Value */}
      <div className="mb-4 p-3 bg-gray-700/30 rounded-lg">
        <div className="flex items-center justify-between">
          <span className="text-gray-400 text-sm">USD Value</span>
          <div className="flex items-center space-x-2">
            <TrendingUp size={16} className="text-green-400" />
            <span className="font-semibold text-green-400 text-lg">
              {formatUSD(usdValue)}
            </span>
          </div>
        </div>
      </div>

      {/* Network Breakdown */}
      {networkBalances.length > 0 && (
        <div className="mb-4">
          <div className="text-xs text-gray-400 mb-2">Networks</div>
          <div className="space-y-1">
            {networkBalances.map((network, index) => (
              <div key={index} className="flex justify-between text-sm">
                <span className="text-gray-300 capitalize">{network.network}</span>
                <span className="text-white font-mono">
                  {formatBalance(network.balance)}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Action Button */}
      <Button
        onClick={handleStartEarn}
        variant="primary"
        size="sm"
        rightIcon={<ArrowRight size={14} />}
        className="w-full bg-gradient-to-r from-primary-500 to-primary-600 hover:from-primary-600 hover:to-primary-700"
        disabled={parseFloat(totalBalance) === 0}
      >
        {parseFloat(totalBalance) > 0 ? 'Start Earn' : 'No Balance'}
      </Button>
    </motion.div>
  );
};

export default StablecoinCard;