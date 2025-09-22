import React, { useState } from 'react';
import { motion } from 'framer-motion';
import {
  History,
  DollarSign,
  Activity,
  Settings,
  Plus,
  Minus,
  Download,
  RefreshCw
} from 'lucide-react';
import { fadeVariants } from '@/utils/animations';
import Button from '@/components/UI/Button';
import Modal from '@/components/UI/Modal';
import ActivityTimeline from './ActivityTimeline';
import { clsx } from 'clsx';

interface QuickActionButtonsProps {
  positionId: string;
  onAddMore: () => void;
  onWithdraw: () => void;
  onWithdrawAll: () => void;
  onManage: () => void;
  isWithdrawing?: boolean;
  className?: string;
}

const QuickActionButtons: React.FC<QuickActionButtonsProps> = ({
  positionId,
  onAddMore,
  onWithdraw,
  onWithdrawAll,
  onManage,
  isWithdrawing = false,
  className
}) => {
  const [showHistoryModal, setShowHistoryModal] = useState(false);
  const [showActivityModal, setShowActivityModal] = useState(false);

  return (
    <>
      <motion.div
        variants={fadeVariants}
        initial="initial"
        animate="animate"
        className={clsx("grid grid-cols-4 gap-2", className)}
      >
        {/* History Button */}
        <Button
          variant="outline"
          size="sm"
          leftIcon={<History size={16} />}
          onClick={() => setShowHistoryModal(true)}
          className="text-blue-400 border-blue-400/30 hover:bg-blue-400/10"
          disabled={isWithdrawing}
        >
          History
        </Button>

        {/* Manage Button */}
        <Button
          variant="outline"
          size="sm"
          leftIcon={<DollarSign size={16} />}
          onClick={onManage}
          className="text-green-400 border-green-400/30 hover:bg-green-400/10"
          disabled={isWithdrawing}
        >
          Manage
        </Button>

        {/* Activity Button */}
        <Button
          variant="outline"
          size="sm"
          leftIcon={<Activity size={16} />}
          onClick={() => setShowActivityModal(true)}
          className="text-purple-400 border-purple-400/30 hover:bg-purple-400/10"
          disabled={isWithdrawing}
        >
          Activity
        </Button>

        {/* Quick Actions Dropdown */}
        <QuickActionsDropdown
          onAddMore={onAddMore}
          onWithdraw={onWithdraw}
          onWithdrawAll={onWithdrawAll}
          isWithdrawing={isWithdrawing}
        />
      </motion.div>

      {/* History Modal */}
      <Modal
        isOpen={showHistoryModal}
        onClose={() => setShowHistoryModal(false)}
        title="Transaction History"
        size="lg"
      >
        <ActivityTimeline
          positionId={positionId}
          limit={20}
          showFilters={true}
          className="bg-transparent border-0"
        />
      </Modal>

      {/* Activity Modal */}
      <Modal
        isOpen={showActivityModal}
        onClose={() => setShowActivityModal(false)}
        title="Live Activity"
        size="md"
      >
        <div className="space-y-4">
          <div className="flex items-center justify-between p-4 bg-gray-800/50 rounded-lg">
            <div className="flex items-center space-x-3">
              <div className="w-3 h-3 bg-green-400 rounded-full animate-pulse" />
              <div>
                <h3 className="text-white font-medium">Position Active</h3>
                <p className="text-sm text-gray-400">Earning yield in real-time</p>
              </div>
            </div>
            <Button
              variant="ghost"
              size="sm"
              leftIcon={<RefreshCw size={16} />}
              onClick={() => window.location.reload()}
            >
              Refresh
            </Button>
          </div>

          <ActivityTimeline
            positionId={positionId}
            limit={5}
            showFilters={false}
            className="bg-transparent border-0"
          />
        </div>
      </Modal>
    </>
  );
};

// Quick Actions Dropdown Component
const QuickActionsDropdown: React.FC<{
  onAddMore: () => void;
  onWithdraw: () => void;
  onWithdrawAll: () => void;
  isWithdrawing: boolean;
}> = ({ onAddMore, onWithdraw, onWithdrawAll, isWithdrawing }) => {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="relative">
      <Button
        variant="outline"
        size="sm"
        leftIcon={<Settings size={16} />}
        onClick={() => setIsOpen(!isOpen)}
        className="text-gray-400 border-gray-400/30 hover:bg-gray-400/10 w-full"
        disabled={isWithdrawing}
      >
        Actions
      </Button>

      {isOpen && (
        <motion.div
          variants={fadeVariants}
          initial="initial"
          animate="animate"
          className="absolute top-full left-0 right-0 mt-2 bg-gray-800 border border-gray-700 rounded-lg shadow-lg z-50"
        >
          <div className="p-2 space-y-1">
            <Button
              variant="ghost"
              size="sm"
              leftIcon={<Plus size={16} />}
              onClick={() => {
                onAddMore();
                setIsOpen(false);
              }}
              className="w-full justify-start text-green-400 hover:bg-green-400/10"
              disabled={isWithdrawing}
            >
              Add More
            </Button>

            <Button
              variant="ghost"
              size="sm"
              leftIcon={<Download size={16} />}
              onClick={() => {
                onWithdraw();
                setIsOpen(false);
              }}
              className="w-full justify-start text-blue-400 hover:bg-blue-400/10"
              disabled={isWithdrawing}
            >
              Withdraw
            </Button>

            <Button
              variant="ghost"
              size="sm"
              leftIcon={<Minus size={16} />}
              onClick={() => {
                onWithdrawAll();
                setIsOpen(false);
              }}
              className="w-full justify-start text-orange-400 hover:bg-orange-400/10"
              disabled={isWithdrawing}
            >
              Withdraw All
            </Button>
          </div>
        </motion.div>
      )}

      {/* Backdrop to close dropdown */}
      {isOpen && (
        <div
          className="fixed inset-0 z-40"
          onClick={() => setIsOpen(false)}
        />
      )}
    </div>
  );
};

export default QuickActionButtons;