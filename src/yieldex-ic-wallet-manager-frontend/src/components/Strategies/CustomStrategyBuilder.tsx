import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Plus, Minus, Settings, TrendingUp, AlertTriangle, Send, Shield, CheckCircle, Save } from 'lucide-react';
import { PROTOCOLS, Protocol, getProtocolsByRisk } from '@/mock/protocols';
import { Strategy } from '@/mock/strategies';
import { fadeVariants, staggerContainer, listItemVariants } from '@/utils/animations';
import Button from '@/components/UI/Button';
import { clsx } from 'clsx';

interface SelectedProtocol {
  protocol: Protocol;
  allocation: number; // Процент от общего капитала
}

interface ProtocolRequestModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (request: ProtocolRequest) => void;
}

interface ProtocolRequest {
  protocolName: string;
  protocolUrl: string;
  description: string;
  requestedBy: string;
}

const ProtocolRequestModal: React.FC<ProtocolRequestModalProps> = ({ isOpen, onClose, onSubmit }) => {
  const [formData, setFormData] = useState<ProtocolRequest>({
    protocolName: '',
    protocolUrl: '',
    description: '',
    requestedBy: ''
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit(formData);
    setFormData({ protocolName: '', protocolUrl: '', description: '', requestedBy: '' });
    onClose();
  };

  if (!isOpen) return null;

  return (
    <motion.div
      variants={fadeVariants}
      initial="initial"
      animate="animate"
      exit="exit"
      className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4"
      onClick={onClose}
    >
      <motion.div
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        exit={{ opacity: 0, scale: 0.9 }}
        className="bg-gray-800/90 backdrop-blur-xl border border-gray-700/50 rounded-xl p-6 max-w-md w-full shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="text-center space-y-4">
          <div className="w-12 h-12 bg-primary-500/20 rounded-full flex items-center justify-center mx-auto">
            <Send size={24} className="text-primary-400" />
          </div>

          <div>
            <h3 className="text-lg font-semibold text-white mb-2">Request New Protocol</h3>
            <p className="text-gray-400 text-sm">
              Submit a request for a protocol you'd like to see integrated
            </p>
          </div>

          <form onSubmit={handleSubmit} className="space-y-4 text-left">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Protocol Name *
              </label>
              <input
                type="text"
                value={formData.protocolName}
                onChange={(e) => setFormData(prev => ({ ...prev, protocolName: e.target.value }))}
                className="w-full px-3 py-2 bg-gray-700/50 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-primary-500"
                placeholder="e.g., Lido Finance"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Protocol Website
              </label>
              <input
                type="url"
                value={formData.protocolUrl}
                onChange={(e) => setFormData(prev => ({ ...prev, protocolUrl: e.target.value }))}
                className="w-full px-3 py-2 bg-gray-700/50 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-primary-500"
                placeholder="https://..."
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Description *
              </label>
              <textarea
                value={formData.description}
                onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
                className="w-full px-3 py-2 bg-gray-700/50 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-primary-500 resize-none"
                rows={3}
                placeholder="Why should we integrate this protocol?"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Your Contact (optional)
              </label>
              <input
                type="text"
                value={formData.requestedBy}
                onChange={(e) => setFormData(prev => ({ ...prev, requestedBy: e.target.value }))}
                className="w-full px-3 py-2 bg-gray-700/50 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-primary-500"
                placeholder="Email or Telegram"
              />
            </div>

            <div className="flex space-x-3 pt-4">
              <Button
                type="button"
                variant="ghost"
                onClick={onClose}
                className="flex-1"
              >
                Cancel
              </Button>
              <Button
                type="submit"
                leftIcon={<Send size={16} />}
                className="flex-1"
              >
                Submit Request
              </Button>
            </div>
          </form>
        </div>
      </motion.div>
    </motion.div>
  );
};

interface CreateStrategyModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: (strategyData: { name: string; description: string }) => void;
  estimatedAPY: number;
  riskLevel: string;
  protocolsCount: number;
}

const CreateStrategyModal: React.FC<CreateStrategyModalProps> = ({
  isOpen,
  onClose,
  onConfirm,
  estimatedAPY,
  riskLevel,
  protocolsCount
}) => {
  const [formData, setFormData] = useState({ name: '', description: '' });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onConfirm(formData);
    setFormData({ name: '', description: '' });
    onClose();
  };

  if (!isOpen) return null;

  return (
    <motion.div
      variants={fadeVariants}
      initial="initial"
      animate="animate"
      exit="exit"
      className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4"
      onClick={onClose}
    >
      <motion.div
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        exit={{ opacity: 0, scale: 0.9 }}
        className="bg-gray-800/90 backdrop-blur-xl border border-gray-700/50 rounded-xl p-6 max-w-md w-full shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="text-center space-y-4">
          <div className="w-12 h-12 bg-primary-500/20 rounded-full flex items-center justify-center mx-auto">
            <Save size={24} className="text-primary-400" />
          </div>

          <div>
            <h3 className="text-lg font-semibold text-white mb-2">Create Custom Strategy</h3>
            <p className="text-gray-400 text-sm">
              Give your strategy a name and description
            </p>
          </div>

          {/* Strategy Summary */}
          <div className="bg-gray-700/50 rounded-lg p-4 border border-gray-600/50 text-left">
            <h4 className="text-white font-medium mb-3">Strategy Summary</h4>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-300">Estimated APY:</span>
                <span className="text-primary-400 font-medium">{estimatedAPY.toFixed(2)}%</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-300">Risk Level:</span>
                <span className={clsx(
                  'capitalize font-medium',
                  riskLevel === 'conservative' && 'text-green-400',
                  riskLevel === 'moderate' && 'text-yellow-400',
                  riskLevel === 'aggressive' && 'text-red-400'
                )}>
                  {riskLevel}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-300">Protocols:</span>
                <span className="text-white font-medium">{protocolsCount}</span>
              </div>
            </div>
          </div>

          <form onSubmit={handleSubmit} className="space-y-4 text-left">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Strategy Name *
              </label>
              <input
                type="text"
                value={formData.name}
                onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                className="w-full px-3 py-2 bg-gray-700/50 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-primary-500"
                placeholder="e.g., My Custom Strategy"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Description *
              </label>
              <textarea
                value={formData.description}
                onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
                className="w-full px-3 py-2 bg-gray-700/50 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-primary-500 resize-none"
                rows={3}
                placeholder="Describe your strategy goals and approach"
                required
              />
            </div>

            <div className="flex space-x-3 pt-4">
              <Button
                type="button"
                variant="ghost"
                onClick={onClose}
                className="flex-1"
              >
                Cancel
              </Button>
              <Button
                type="submit"
                leftIcon={<Save size={16} />}
                className="flex-1"
              >
                Create Strategy
              </Button>
            </div>
          </form>
        </div>
      </motion.div>
    </motion.div>
  );
};

const CustomStrategyBuilder: React.FC = () => {
  const [selectedProtocols, setSelectedProtocols] = useState<SelectedProtocol[]>([]);
  const [showRequestModal, setShowRequestModal] = useState(false);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [createdStrategy, setCreatedStrategy] = useState<Strategy | null>(null);
  const [riskFilter, setRiskFilter] = useState<'all' | 'conservative' | 'moderate' | 'aggressive'>('all');

  const availableProtocols = riskFilter === 'all'
    ? PROTOCOLS.filter(p => p.isActive)
    : getProtocolsByRisk(riskFilter);

  const totalAllocation = selectedProtocols.reduce((sum, p) => sum + p.allocation, 0);
  const isValidStrategy = selectedProtocols.length >= 2 && totalAllocation === 100;

  const addProtocol = (protocol: Protocol) => {
    if (selectedProtocols.find(p => p.protocol.id === protocol.id)) return;

    const remainingAllocation = 100 - totalAllocation;
    const defaultAllocation = Math.min(remainingAllocation, 50);

    setSelectedProtocols(prev => [
      ...prev,
      { protocol, allocation: defaultAllocation }
    ]);
  };

  const removeProtocol = (protocolId: string) => {
    setSelectedProtocols(prev => prev.filter(p => p.protocol.id !== protocolId));
  };

  const updateAllocation = (protocolId: string, allocation: number) => {
    setSelectedProtocols(prev => prev.map(p =>
      p.protocol.id === protocolId ? { ...p, allocation } : p
    ));
  };

  const handleProtocolRequest = (request: ProtocolRequest) => {
    // В реальном приложении здесь бы отправлялся запрос на сервер
    console.log('Protocol request submitted:', request);
    // Показать уведомление об успешной отправке
  };

  const handleCreateStrategy = (strategyData: { name: string; description: string }) => {
    // Создаем новую стратегию
    const allocation: Record<string, number> = {};
    selectedProtocols.forEach(sp => {
      allocation[sp.protocol.id] = sp.allocation;
    });

    const newStrategy: Strategy = {
      id: `custom-${Date.now()}`,
      name: strategyData.name,
      description: strategyData.description,
      risk: getRiskLabel(averageRisk) as 'conservative' | 'moderate' | 'aggressive',
      expectedApy: estimatedAPY,
      minDeposit: 100,
      maxDeposit: 1000000,
      protocols: selectedProtocols.map(sp => sp.protocol.id),
      allocation,
      supportedTokens: ['USDC', 'USDT', 'DAI'],
      isActive: true,
      totalDeposited: 0,
      performanceHistory: [],
      features: [
        'Custom allocation',
        `${selectedProtocols.length} protocols`,
        'Personalized strategy',
        'Real-time rebalancing'
      ]
    };

    setCreatedStrategy(newStrategy);
    console.log('Custom strategy created:', newStrategy);

    // В реальном приложении здесь бы стратегия сохранялась в store/backend
    // Можно добавить в локальный стейт или отправить на сервер
  };

  const getRiskColor = (risk: string) => {
    switch (risk) {
      case 'conservative': return 'text-green-400 bg-green-500/20';
      case 'moderate': return 'text-yellow-400 bg-yellow-500/20';
      case 'aggressive': return 'text-red-400 bg-red-500/20';
      default: return 'text-gray-400 bg-gray-500/20';
    }
  };

  const estimatedAPY = selectedProtocols.length > 0
    ? selectedProtocols.reduce((sum, p) => sum + (p.protocol.apy * p.allocation / 100), 0)
    : 0;

  const averageRisk = selectedProtocols.length > 0
    ? selectedProtocols.reduce((sum, p) => {
        const riskValue = p.protocol.risk === 'conservative' ? 1 : p.protocol.risk === 'moderate' ? 2 : 3;
        return sum + (riskValue * p.allocation / 100);
      }, 0)
    : 0;

  const getRiskLabel = (value: number) => {
    if (value <= 1.5) return 'conservative';
    if (value <= 2.5) return 'moderate';
    return 'aggressive';
  };

  return (
    <motion.div
      variants={staggerContainer}
      initial="hidden"
      animate="visible"
      className="space-y-6"
    >
      {/* Header */}
      <motion.div variants={fadeVariants} className="text-center space-y-4">
        <div className="w-16 h-16 bg-primary-500/20 rounded-full flex items-center justify-center mx-auto">
          <Settings className="w-8 h-8 text-primary-400" />
        </div>
        <div>
          <h3 className="text-xl font-semibold text-white mb-2">Build Your Custom Strategy</h3>
          <p className="text-gray-400">
            Select protocols and set allocation percentages to create your personalized DeFi strategy
          </p>
        </div>
      </motion.div>

      {/* Strategy Summary */}
      {selectedProtocols.length > 0 && (
        <motion.div
          variants={fadeVariants}
          className="bg-gray-800/50 border border-gray-700/50 rounded-xl p-6"
        >
          <h4 className="text-white font-semibold mb-4">Strategy Overview</h4>
          <div className="grid grid-cols-3 gap-4 text-center">
            <div>
              <div className="text-2xl font-bold text-primary-400">
                {estimatedAPY.toFixed(2)}%
              </div>
              <div className="text-sm text-gray-400">Estimated APY</div>
            </div>
            <div>
              <div className={clsx(
                'text-2xl font-bold capitalize',
                getRiskColor(getRiskLabel(averageRisk)).split(' ')[0]
              )}>
                {getRiskLabel(averageRisk)}
              </div>
              <div className="text-sm text-gray-400">Risk Level</div>
            </div>
            <div>
              <div className="text-2xl font-bold text-white">
                {selectedProtocols.length}
              </div>
              <div className="text-sm text-gray-400">Protocols</div>
            </div>
          </div>

          {totalAllocation !== 100 && (
            <div className="mt-4 p-3 bg-yellow-500/10 border border-yellow-500/30 rounded-lg">
              <div className="flex items-center text-yellow-400 text-sm">
                <AlertTriangle size={16} className="mr-2" />
                Allocation must total 100% (currently {totalAllocation}%)
              </div>
            </div>
          )}
        </motion.div>
      )}

      {/* Selected Protocols */}
      {selectedProtocols.length > 0 && (
        <motion.div variants={fadeVariants} className="space-y-4">
          <h4 className="text-white font-semibold">Selected Protocols</h4>
          {selectedProtocols.map((selectedProtocol, index) => (
            <motion.div
              key={selectedProtocol.protocol.id}
              variants={listItemVariants}
              custom={index}
              className="bg-gray-800/30 border border-gray-700/50 rounded-lg p-4"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <div>
                    <div className="text-white font-medium">{selectedProtocol.protocol.name}</div>
                    <div className="text-sm text-gray-400">
                      {selectedProtocol.protocol.apy.toFixed(2)}% APY
                    </div>
                  </div>
                  <span className={clsx(
                    'px-2 py-1 rounded-full text-xs font-medium',
                    getRiskColor(selectedProtocol.protocol.risk)
                  )}>
                    {selectedProtocol.protocol.risk}
                  </span>
                </div>

                <div className="flex items-center space-x-3">
                  <div className="flex items-center space-x-2">
                    <input
                      type="number"
                      min="0"
                      max="100"
                      value={selectedProtocol.allocation}
                      onChange={(e) => updateAllocation(
                        selectedProtocol.protocol.id,
                        Math.max(0, Math.min(100, parseInt(e.target.value) || 0))
                      )}
                      className="w-16 px-2 py-1 bg-gray-700/50 border border-gray-600 rounded text-white text-sm text-center"
                    />
                    <span className="text-gray-400 text-sm">%</span>
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => removeProtocol(selectedProtocol.protocol.id)}
                    leftIcon={<Minus size={14} />}
                  >
                    Remove
                  </Button>
                </div>
              </div>
            </motion.div>
          ))}
        </motion.div>
      )}

      {/* Risk Filter */}
      <motion.div variants={fadeVariants} className="space-y-4">
        <div className="flex items-center justify-between">
          <h4 className="text-white font-semibold">Available Protocols</h4>
          <div className="flex items-center space-x-2">
            <select
              value={riskFilter}
              onChange={(e) => setRiskFilter(e.target.value as any)}
              className="px-3 py-2 bg-gray-700/50 border border-gray-600 rounded-lg text-white text-sm"
            >
              <option value="all">All Risk Levels</option>
              <option value="conservative">Conservative</option>
              <option value="moderate">Moderate</option>
              <option value="aggressive">Aggressive</option>
            </select>
          </div>
        </div>

        {/* Available Protocols Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {availableProtocols.map((protocol, index) => {
            const isSelected = selectedProtocols.find(p => p.protocol.id === protocol.id);

            return (
              <motion.div
                key={protocol.id}
                variants={listItemVariants}
                custom={index}
                className={clsx(
                  'bg-gray-800/30 border rounded-lg p-4 transition-all duration-200',
                  isSelected
                    ? 'border-primary-500/50 bg-primary-500/5'
                    : 'border-gray-700/50 hover:border-gray-600/50 hover:bg-gray-800/50'
                )}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3">
                    <div>
                      <div className="text-white font-medium text-sm">{protocol.name}</div>
                      <div className="text-xs text-gray-400">
                        {protocol.apy.toFixed(2)}% APY
                      </div>
                    </div>
                  </div>

                  <div className="flex items-center space-x-2">
                    <span className={clsx(
                      'px-2 py-1 rounded-full text-xs font-medium',
                      getRiskColor(protocol.risk)
                    )}>
                      {protocol.risk}
                    </span>

                    {isSelected ? (
                      <Button
                        variant="ghost"
                        size="sm"
                        leftIcon={<Minus size={14} />}
                        onClick={() => removeProtocol(protocol.id)}
                      >
                        Remove
                      </Button>
                    ) : (
                      <Button
                        variant="ghost"
                        size="sm"
                        leftIcon={<Plus size={14} />}
                        onClick={() => addProtocol(protocol)}
                      >
                        Add
                      </Button>
                    )}
                  </div>
                </div>
              </motion.div>
            );
          })}
        </div>

        {/* Protocol Request Button */}
        <motion.div variants={fadeVariants} className="text-center">
          <Button
            variant="outline"
            onClick={() => setShowRequestModal(true)}
            leftIcon={<Send size={16} />}
          >
            Don't see your protocol? Submit a request
          </Button>
        </motion.div>
      </motion.div>

      {/* Created Strategy Success */}
      {createdStrategy && (
        <motion.div
          variants={fadeVariants}
          initial="initial"
          animate="animate"
          className="bg-green-500/10 border border-green-500/30 rounded-xl p-6"
        >
          <div className="flex items-center space-x-3 mb-4">
            <CheckCircle className="w-8 h-8 text-green-400" />
            <div>
              <h4 className="text-white font-semibold">Strategy Created Successfully!</h4>
              <p className="text-green-400 text-sm">{createdStrategy.name}</p>
            </div>
          </div>

          <div className="bg-gray-800/50 rounded-lg p-4 mb-4">
            <div className="grid grid-cols-3 gap-4 text-center text-sm">
              <div>
                <div className="text-primary-400 font-bold">{createdStrategy.expectedApy.toFixed(2)}%</div>
                <div className="text-gray-400">Expected APY</div>
              </div>
              <div>
                <div className={clsx(
                  'font-bold capitalize',
                  createdStrategy.risk === 'conservative' && 'text-green-400',
                  createdStrategy.risk === 'moderate' && 'text-yellow-400',
                  createdStrategy.risk === 'aggressive' && 'text-red-400'
                )}>
                  {createdStrategy.risk}
                </div>
                <div className="text-gray-400">Risk Level</div>
              </div>
              <div>
                <div className="text-white font-bold">{createdStrategy.protocols.length}</div>
                <div className="text-gray-400">Protocols</div>
              </div>
            </div>
          </div>

          <div className="flex space-x-3">
            <Button
              variant="ghost"
              onClick={() => {
                setCreatedStrategy(null);
                setSelectedProtocols([]);
              }}
              className="flex-1"
            >
              Create Another
            </Button>
            <Button
              leftIcon={<TrendingUp size={16} />}
              className="flex-1"
              onClick={() => {
                // В реальном приложении здесь бы происходил переход к инвестированию
                console.log('Start investing with strategy:', createdStrategy.id);
              }}
            >
              Start Investing
            </Button>
          </div>
        </motion.div>
      )}

      {/* Action Buttons */}
      {!createdStrategy && (
        <motion.div variants={fadeVariants} className="flex space-x-4">
          <Button
            variant="ghost"
            className="flex-1"
            onClick={() => setSelectedProtocols([])}
            disabled={selectedProtocols.length === 0}
          >
            Clear All
          </Button>
          <Button
            className="flex-1"
            leftIcon={<Save size={16} />}
            disabled={!isValidStrategy}
            onClick={() => setShowCreateModal(true)}
          >
            Create Strategy
          </Button>
        </motion.div>
      )}

      {/* Protocol Request Modal */}
      <AnimatePresence>
        <ProtocolRequestModal
          isOpen={showRequestModal}
          onClose={() => setShowRequestModal(false)}
          onSubmit={handleProtocolRequest}
        />
      </AnimatePresence>

      {/* Create Strategy Modal */}
      <AnimatePresence>
        <CreateStrategyModal
          isOpen={showCreateModal}
          onClose={() => setShowCreateModal(false)}
          onConfirm={handleCreateStrategy}
          estimatedAPY={estimatedAPY}
          riskLevel={getRiskLabel(averageRisk)}
          protocolsCount={selectedProtocols.length}
        />
      </AnimatePresence>
    </motion.div>
  );
};

export default CustomStrategyBuilder;