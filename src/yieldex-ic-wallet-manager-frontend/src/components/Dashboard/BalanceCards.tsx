import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { Wallet, Send, Download, RefreshCw } from 'lucide-react';
import { useWalletBalances, useWalletNetwork } from '@/stores/walletStore';
import { NETWORKS } from '@/mock/protocols';
import { formatCurrency, getNetworkColor } from '@/utils/formatters';
import { cardVariants, staggerContainer, listItemVariants } from '@/utils/animations';
import Card, { CardHeader, CardTitle, CardContent, CardFooter } from '@/components/UI/Card';
import { Grid, Section } from '@/components/UI/Layout';
import Button from '@/components/UI/Button';
import { clsx } from 'clsx';

interface BalanceCardsProps {
  showActions?: boolean;
}

const BalanceCards: React.FC<BalanceCardsProps> = ({ showActions = true }) => {
  const [refreshing, setRefreshing] = useState(false);
  const { balances, totalPortfolioValue, refreshBalances } = useWalletBalances();
  const { networkId } = useWalletNetwork();
  

  // Group balances by network
  const balancesByNetwork = balances.reduce((acc, balance) => {
    if (!acc[balance.network]) {
      acc[balance.network] = [];
    }
    acc[balance.network].push(balance);
    return acc;
  }, {} as Record<string, typeof balances>);

  // Calculate network totals
  const networkTotals = Object.entries(balancesByNetwork).map(([networkSlug, networkBalances]) => {
    const network = NETWORKS.find(n => n.slug === networkSlug);
    const total = networkBalances.reduce((sum, balance) => sum + balance.value, 0);
    return {
      network,
      balances: networkBalances,
      total,
      slug: networkSlug
    };
  });

  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      await refreshBalances();
    } finally {
      setTimeout(() => setRefreshing(false), 1000); // Ensure at least 1s feedback
    }
  };

  const getNetworkIcon = (network?: typeof NETWORKS[0]) => {
    if (!network) return '🔗';
    const icons: Record<string, string> = {
      ethereum: '⟠',
      arbitrum: '🔵',
      polygon: '🟣',
      bsc: '🟡'
    };
    return icons[network.slug] || '🔗';
  };

  return (
    <Section title="Token Balances">
      <div className="space-y-6">
        {/* Header with Total and Refresh */}
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold text-white">
              {formatCurrency(totalPortfolioValue)}
            </h2>
            <p className="text-gray-400">Total Portfolio Value</p>
          </div>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleRefresh}
            loading={refreshing}
            leftIcon={<RefreshCw size={16} />}
          >
            Refresh
          </Button>
        </div>

        {/* Network Tabs or Current Network Display */}
        {networkTotals.length > 1 && (
          <div className="flex flex-wrap gap-2">
            {networkTotals.map(({ network, total }) => {
              if (!network) return null;
              
              const isActive = network.id === networkId;
              
              return (
                <div
                  key={network.id}
                  className={clsx(
                    'flex items-center space-x-2 px-3 py-2 rounded-lg border transition-all',
                    isActive 
                      ? 'bg-primary-500/20 border-primary-500/50 text-primary-300'
                      : 'bg-gray-800/50 border-gray-700/50 text-gray-400'
                  )}
                >
                  <span className="text-lg">{getNetworkIcon(network)}</span>
                  <div>
                    <div className="font-medium text-white">{network.name}</div>
                    <div className="text-xs">{formatCurrency(total)}</div>
                  </div>
                </div>
              );
            })}
          </div>
        )}

        {/* Balance Cards */}
        <motion.div
          variants={staggerContainer}
          initial="hidden"
          animate="visible"
        >
          <Grid cols={2} gap="md" className="md:grid-cols-3 lg:grid-cols-4">
            {balances
              .filter(balance => balance.balance > 0) // Only show non-zero balances
              .map((balance, index) => {
                const network = NETWORKS.find(n => n.slug === balance.network);
                
                return (
                  <motion.div
                    key={`${balance.token}-${balance.network}`}
                    variants={listItemVariants}
                    custom={index}
                  >
                    <BalanceCard
                      balance={balance}
                      network={network}
                      showActions={showActions}
                    />
                  </motion.div>
                );
              })}
          </Grid>
        </motion.div>

        {/* Empty State */}
        {balances.filter(b => b.balance > 0).length === 0 && (
          <motion.div
            variants={cardVariants}
            initial="initial"
            animate="animate"
            className="text-center py-12 bg-gray-800/30 rounded-xl"
          >
            <Wallet size={48} className="text-gray-500 mx-auto mb-4" />
            <h3 className="text-lg font-semibold text-white mb-2">No Token Balances</h3>
            <p className="text-gray-400 mb-6">
              Your wallet appears to be empty. Add some tokens to get started.
            </p>
            <Button
              leftIcon={<Download size={16} />}
              onClick={() => console.log('Add tokens')}
            >
              Add Tokens
            </Button>
          </motion.div>
        )}
      </div>
    </Section>
  );
};

// Individual Balance Card Component
const BalanceCard: React.FC<{
  balance: { token: string; balance: number; value: number; network: string };
  network?: typeof NETWORKS[0];
  showActions: boolean;
}> = ({ balance, network, showActions }) => {
  const getTokenIcon = (token: string) => {
    const icons: Record<string, string> = {
      'USDC': '💙',
      'USDT': '💚',
      'DAI': '🟡',
      'ETH': '♦️',
      'WETH': '🔷'
    };
    return icons[token] || '🪙';
  };

  const getNetworkIcon = (network?: typeof NETWORKS[0]) => {
    if (!network) return '🔗';
    const icons: Record<string, string> = {
      ethereum: '⟠',
      arbitrum: '🔵',
      polygon: '🟣',
      bsc: '🟡'
    };
    return icons[network.slug] || '🔗';
  };

  return (
    <Card variant="glass" className="h-full">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <span className="text-2xl">{getTokenIcon(balance.token)}</span>
            <CardTitle size="sm">{balance.token}</CardTitle>
          </div>
          
          {network && (
            <div className={clsx(
              'flex items-center space-x-1 px-2 py-1 rounded-full text-xs',
              getNetworkColor(network.slug)
            )}>
              <span>{getNetworkIcon(network)}</span>
              <span>{network.name.split(' ')[0]}</span>
            </div>
          )}
        </div>
      </CardHeader>

      <CardContent>
        <div className="space-y-2">
          <div>
            <motion.div
              key={balance.balance}
              initial={{ scale: 0.95 }}
              animate={{ scale: 1 }}
              transition={{ duration: 0.2 }}
              className="text-xl font-bold text-white"
            >
              {balance.balance.toLocaleString('en-US', {
                minimumFractionDigits: 2,
                maximumFractionDigits: 6
              })}
            </motion.div>
            <div className="text-sm text-gray-400">
              {formatCurrency(balance.value)}
            </div>
          </div>
        </div>
      </CardContent>

      {showActions && (
        <CardFooter>
          <div className="flex space-x-2 w-full">
            <Button
              variant="ghost"
              size="sm"
              className="flex-1"
              leftIcon={<Send size={14} />}
              onClick={() => console.log('Send', balance.token)}
            >
              Send
            </Button>
            <Button
              variant="ghost"
              size="sm"
              className="flex-1"
              leftIcon={<Download size={14} />}
              onClick={() => console.log('Receive', balance.token)}
            >
              Add
            </Button>
          </div>
        </CardFooter>
      )}
    </Card>
  );
};

export default BalanceCards;