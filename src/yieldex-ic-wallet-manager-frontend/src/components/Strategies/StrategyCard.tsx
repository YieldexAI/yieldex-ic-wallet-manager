import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { TrendingUp, Shield, Zap, Target, ChevronRight } from 'lucide-react';
import { Strategy } from '@/mock/strategies';
import { PROTOCOLS } from '@/mock/protocols';
import { formatCurrency, formatAPY, getRiskColor } from '@/utils/formatters';
import { cardVariants, fadeVariants } from '@/utils/animations';
import Button from '@/components/UI/Button';
import Card, { CardHeader, CardTitle, CardContent, CardFooter } from '@/components/UI/Card';
import { clsx } from 'clsx';

interface StrategyCardProps {
  strategy: Strategy;
  onSelect: (strategy: Strategy) => void;
  isSelected?: boolean;
  className?: string;
}

const StrategyCard: React.FC<StrategyCardProps> = ({
  strategy,
  onSelect,
  isSelected = false,
  className
}) => {
  const [showDetails, setShowDetails] = useState(false);

  const getRiskIcon = (risk: Strategy['risk']) => {
    switch (risk) {
      case 'conservative':
        return <Shield className="w-5 h-5" />;
      case 'moderate':
        return <Target className="w-5 h-5" />;
      case 'aggressive':
        return <Zap className="w-5 h-5" />;
      default:
        return <TrendingUp className="w-5 h-5" />;
    }
  };

  const getRiskLabel = (risk: Strategy['risk']) => {
    switch (risk) {
      case 'conservative':
        return 'Conservative';
      case 'moderate':
        return 'Moderate';
      case 'aggressive':
        return 'Aggressive';
      default:
        return 'Unknown';
    }
  };

  const getStrategyProtocols = () => {
    return strategy.protocols.map(protocolId => {
      const protocol = PROTOCOLS.find(p => p.id === protocolId);
      return { protocol };
    }).filter(item => item.protocol);
  };

  return (
    <motion.div
      variants={cardVariants}
      initial="initial"
      animate="animate"
      whileHover="hover"
      className={clsx('h-full', className)}
    >
      <Card
        variant="glass"
        className={clsx(
          'h-full transition-all duration-300 cursor-pointer',
          isSelected && 'ring-2 ring-primary-500/50 bg-primary-500/5',
          'hover:shadow-xl hover:shadow-primary-500/10'
        )}
        onClick={() => onSelect(strategy)}
      >
        <CardHeader>
          <div className="flex items-center justify-between gap-6">
            <CardTitle size="lg" className="flex items-center space-x-2">
              <div className={clsx('p-2 rounded-lg', getRiskColor(strategy.risk))}>
                {getRiskIcon(strategy.risk)}
              </div>
              <span>{strategy.name}</span>
            </CardTitle>
            
            <div className="text-right">
              <div className="text-2xl font-bold text-primary-400">
                {formatAPY(strategy.expectedApy)}
              </div>
              <div className="text-xs text-gray-400">Expected APY</div>
            </div>
          </div>
        </CardHeader>

        <CardContent>
          <div className="space-y-4">
            {/* Description */}
            <p className="text-sm text-gray-400 leading-relaxed">
              {strategy.description}
            </p>

            {/* Risk Badge */}
            <div className="flex items-center space-x-2">
              <span className={clsx(
                'px-2 py-1 rounded-full text-xs font-medium border',
                getRiskColor(strategy.risk)
              )}>
                {getRiskLabel(strategy.risk)}
              </span>
              <span className="text-xs text-gray-500">
                Min: {formatCurrency(strategy.minDeposit)}
              </span>
            </div>

            {/* Protocols */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium text-gray-300">Protocols</span>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    setShowDetails(!showDetails);
                  }}
                  className="text-xs text-primary-400 hover:text-primary-300 transition-colors"
                >
                  {showDetails ? 'Hide' : 'Show'} Details
                </button>
              </div>

              {/* Protocol Pills */}
              <div className="flex flex-wrap gap-2">
                {getStrategyProtocols().slice(0, showDetails ? undefined : 3).map(({ protocol }) => (
                  <div
                    key={protocol!.id}
                    className="px-2 py-1 bg-gray-700/50 rounded-full text-xs"
                  >
                    <span className="text-gray-300">{protocol!.name}</span>
                  </div>
                ))}
                {!showDetails && getStrategyProtocols().length > 3 && (
                  <div className="px-2 py-1 bg-gray-700/50 rounded-full text-xs text-gray-400">
                    +{getStrategyProtocols().length - 3} more
                  </div>
                )}
              </div>
            </div>

            {/* Strategy Stats */}
            <div className="grid grid-cols-2 gap-4 pt-4 border-t border-gray-700/50">
              <div className="text-center">
                <div className="text-lg font-semibold text-white">
                  {formatCurrency(strategy.totalDeposited / 1000000)}M
                </div>
                <div className="text-xs text-gray-400">Total Deposited</div>
              </div>
              <div className="text-center">
                <div className="text-lg font-semibold text-green-400">
                  {strategy.protocols.length}
                </div>
                <div className="text-xs text-gray-400">Protocols</div>
              </div>
            </div>

            {/* Features */}
            {showDetails && (
              <motion.div
                variants={fadeVariants}
                initial="initial"
                animate="animate"
                className="space-y-2 pt-4 border-t border-gray-700/50"
              >
                <h4 className="text-sm font-medium text-white">Key Features</h4>
                <div className="space-y-1">
                  {strategy.features.map((feature, index) => (
                    <div key={index} className="flex items-center space-x-2 text-xs">
                      <div className="w-1 h-1 bg-primary-400 rounded-full flex-shrink-0" />
                      <span className="text-gray-400">{feature}</span>
                    </div>
                  ))}
                </div>
              </motion.div>
            )}
          </div>
        </CardContent>

        <CardFooter>
          <Button
            fullWidth
            variant={isSelected ? 'primary' : 'outline'}
            rightIcon={<ChevronRight size={16} />}
            onClick={(e) => {
              e.stopPropagation();
              onSelect(strategy);
            }}
          >
            {isSelected ? 'Selected Strategy' : 'Select Strategy'}
          </Button>
        </CardFooter>
      </Card>
    </motion.div>
  );
};

export default StrategyCard;