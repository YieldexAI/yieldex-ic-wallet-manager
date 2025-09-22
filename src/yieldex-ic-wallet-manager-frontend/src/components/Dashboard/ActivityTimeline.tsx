import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  Clock,
  ExternalLink,
  ChevronDown,
  ChevronUp,
  Filter,
  RefreshCw,
  Activity as ActivityIcon
} from 'lucide-react';
import { useTransactionStore, useRecentActivity } from '@/stores/transactionStore';
import { Transaction, TransactionGroup, TransactionType } from '@/types/transactions';
import { formatCurrency, formatTimeAgo } from '@/utils/formatters';
import { fadeVariants, staggerContainer, listItemVariants } from '@/utils/animations';
import Card, { CardHeader, CardTitle, CardContent } from '@/components/UI/Card';
import Button from '@/components/UI/Button';
import Badge from '@/components/UI/Badge';
import { clsx } from 'clsx';

interface ActivityTimelineProps {
  positionId?: string;
  limit?: number;
  showFilters?: boolean;
  className?: string;
}

const ActivityTimeline: React.FC<ActivityTimelineProps> = ({
  positionId,
  limit = 10,
  showFilters = true,
  className
}) => {
  const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set());
  const [showFilterPanel, setShowFilterPanel] = useState(false);

  const {
    getTransactionsByPosition,
    getFilteredTransactions,
    activeFilter,
    setFilter,
    clearFilter,
    initializeWithDefaultActivity
  } = useTransactionStore();

  const recentActivity = useRecentActivity(limit);

  // Initialize default activity if no transactions exist
  useEffect(() => {
    initializeWithDefaultActivity(positionId);
  }, [initializeWithDefaultActivity, positionId]);

  // Get activity data based on positionId filter
  const activityData = positionId
    ? getTransactionsByPosition(positionId).slice(0, limit)
    : getFilteredTransactions().slice(0, limit);

  const toggleExpanded = (id: string) => {
    const newExpanded = new Set(expandedItems);
    if (newExpanded.has(id)) {
      newExpanded.delete(id);
    } else {
      newExpanded.add(id);
    }
    setExpandedItems(newExpanded);
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed': return 'text-green-400 bg-green-400/10';
      case 'pending': return 'text-yellow-400 bg-yellow-400/10';
      case 'processing': return 'text-blue-400 bg-blue-400/10';
      case 'failed': return 'text-red-400 bg-red-400/10';
      default: return 'text-gray-400 bg-gray-400/10';
    }
  };

  const handleFilterTypeToggle = (type: TransactionType) => {
    const currentTypes = activeFilter.types;
    const newTypes = currentTypes.includes(type)
      ? currentTypes.filter(t => t !== type)
      : [...currentTypes, type];

    setFilter({ types: newTypes });
  };

  if (activityData.length === 0) {
    return (
      <Card variant="glass" className={className}>
        <CardContent className="text-center py-8">
          <ActivityIcon size={48} className="text-gray-500 mx-auto mb-4" />
          <h3 className="text-lg font-semibold text-white mb-2">No Activity Yet</h3>
          <p className="text-gray-400">
            Transaction history will appear here as you use the platform.
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card variant="glass" className={className}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle size="md" className="flex items-center space-x-2">
            <Clock className="w-5 h-5 text-primary-400" />
            <span>Activity Timeline</span>
            {positionId && (
              <Badge variant="secondary" size="sm">
                Position Activity
              </Badge>
            )}
          </CardTitle>

          {showFilters && (
            <div className="flex items-center space-x-2">
              <Button
                variant="ghost"
                size="sm"
                leftIcon={<Filter size={16} />}
                onClick={() => setShowFilterPanel(!showFilterPanel)}
                className={clsx(showFilterPanel && 'bg-gray-700')}
              >
                Filter
              </Button>
              <Button
                variant="ghost"
                size="sm"
                leftIcon={<RefreshCw size={16} />}
                onClick={() => window.location.reload()}
              >
                Refresh
              </Button>
            </div>
          )}
        </div>

        {/* Filter Panel */}
        <AnimatePresence>
          {showFilterPanel && (
            <motion.div
              variants={fadeVariants}
              initial="initial"
              animate="animate"
              exit="exit"
              className="mt-4 p-4 bg-gray-800/50 rounded-lg"
            >
              <div className="space-y-3">
                <div>
                  <h4 className="text-sm font-medium text-white mb-2">Transaction Types</h4>
                  <div className="flex flex-wrap gap-2">
                    {['deposit', 'withdrawal', 'rebalancing', 'yield_collection', 'smart_wallet_creation'].map((type) => (
                      <Badge
                        key={type}
                        variant={activeFilter.types.includes(type as TransactionType) ? 'primary' : 'secondary'}
                        className="cursor-pointer"
                        onClick={() => handleFilterTypeToggle(type as TransactionType)}
                      >
                        {type.replace('_', ' ')}
                      </Badge>
                    ))}
                  </div>
                </div>

                <div className="flex justify-between">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={clearFilter}
                  >
                    Clear Filters
                  </Button>
                  <span className="text-sm text-gray-400">
                    {activityData.length} result{activityData.length !== 1 ? 's' : ''}
                  </span>
                </div>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </CardHeader>

      <CardContent>
        <motion.div
          variants={staggerContainer}
          initial="hidden"
          animate="visible"
          className="space-y-4"
        >
          {activityData.map((item, index) => (
            <motion.div
              key={item.id}
              variants={listItemVariants}
              custom={index}
            >
              {'transactions' in item ? (
                <TransactionGroupItem
                  group={item}
                  isExpanded={expandedItems.has(item.id)}
                  onToggleExpanded={() => toggleExpanded(item.id)}
                />
              ) : (
                <TransactionItem transaction={item} />
              )}
            </motion.div>
          ))}
        </motion.div>
      </CardContent>
    </Card>
  );
};

// Individual Transaction Item
const TransactionItem: React.FC<{ transaction: Transaction }> = ({ transaction }) => {
  return (
    <div className="flex items-start space-x-4 p-4 bg-gray-800/30 rounded-lg hover:bg-gray-800/50 transition-colors">
      <div className="flex-shrink-0">
        <div className={clsx(
          'w-8 h-8 rounded-full flex items-center justify-center text-lg',
          'bg-gray-700'
        )}>
          {transaction.icon}
        </div>
      </div>

      <div className="flex-1 min-w-0">
        <div className="flex items-center justify-between mb-1">
          <h4 className="text-sm font-medium text-white truncate">
            {transaction.description}
          </h4>
          <div className="flex items-center space-x-2 ml-4">
            {transaction.amount && (
              <span className={clsx('text-sm font-medium', transaction.color)}>
                {transaction.type === 'withdrawal' ? '-' : '+'}
                {formatCurrency(transaction.amount)} {transaction.token}
              </span>
            )}
            <Badge
              variant="secondary"
              size="sm"
              className={getStatusColor(transaction.status)}
            >
              {transaction.status}
            </Badge>
          </div>
        </div>

        <div className="flex items-center justify-between">
          <p className="text-xs text-gray-400">
            {formatTimeAgo(new Date(transaction.timestamp))}
          </p>

          <div className="flex items-center space-x-2">
            {transaction.blockchainRefs.map((ref, index) => (
              ref.explorerUrl && (
                <Button
                  key={index}
                  variant="ghost"
                  size="sm"
                  rightIcon={<ExternalLink size={12} />}
                  onClick={() => window.open(ref.explorerUrl, '_blank')}
                  className="text-xs text-blue-400 hover:text-blue-300"
                >
                  View on {ref.network}
                </Button>
              )
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

// Transaction Group Item (for rebalancing sequences)
const TransactionGroupItem: React.FC<{
  group: TransactionGroup;
  isExpanded: boolean;
  onToggleExpanded: () => void;
}> = ({ group, isExpanded, onToggleExpanded }) => {
  return (
    <div className="bg-gray-800/30 rounded-lg overflow-hidden">
      <div
        className="flex items-start space-x-4 p-4 cursor-pointer hover:bg-gray-800/50 transition-colors"
        onClick={onToggleExpanded}
      >
        <div className="flex-shrink-0">
          <div className="w-8 h-8 rounded-full bg-primary-500/20 flex items-center justify-center">
            🔄
          </div>
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between mb-1">
            <h4 className="text-sm font-medium text-white">
              {group.description}
            </h4>
            <div className="flex items-center space-x-2 ml-4">
              <Badge variant="secondary" size="sm">
                {group.transactions.length} steps
              </Badge>
              <Badge
                variant="secondary"
                size="sm"
                className={getStatusColor(group.status)}
              >
                {group.status}
              </Badge>
              {isExpanded ? (
                <ChevronUp size={16} className="text-gray-400" />
              ) : (
                <ChevronDown size={16} className="text-gray-400" />
              )}
            </div>
          </div>

          <div className="flex items-center justify-between">
            <p className="text-xs text-gray-400">
              {formatTimeAgo(new Date(group.timestamp))}
            </p>
            {group.totalGasCost && (
              <span className="text-xs text-gray-400">
                Gas: {group.totalGasCost} ETH
              </span>
            )}
          </div>
        </div>
      </div>

      <AnimatePresence>
        {isExpanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="border-t border-gray-700"
          >
            <div className="p-4 space-y-3">
              {group.transactions.map((transaction, index) => (
                <div key={transaction.id} className="flex items-center space-x-3 text-sm">
                  <div className="w-6 h-6 rounded-full bg-gray-700 flex items-center justify-center text-xs">
                    {index + 1}
                  </div>
                  <div className="flex-1">
                    <span className="text-white">{transaction.description}</span>
                    {transaction.amount && (
                      <span className="ml-2 text-gray-400">
                        ({formatCurrency(transaction.amount)} {transaction.token})
                      </span>
                    )}
                  </div>
                  {transaction.blockchainRefs[0]?.explorerUrl && (
                    <Button
                      variant="ghost"
                      size="sm"
                      rightIcon={<ExternalLink size={12} />}
                      onClick={() => window.open(transaction.blockchainRefs[0].explorerUrl, '_blank')}
                      className="text-xs text-blue-400 hover:text-blue-300"
                    >
                      View
                    </Button>
                  )}
                </div>
              ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

const getStatusColor = (status: string) => {
  switch (status) {
    case 'completed': return 'text-green-400 bg-green-400/10';
    case 'pending': return 'text-yellow-400 bg-yellow-400/10';
    case 'processing': return 'text-blue-400 bg-blue-400/10';
    case 'failed': return 'text-red-400 bg-red-400/10';
    default: return 'text-gray-400 bg-gray-400/10';
  }
};

export default ActivityTimeline;