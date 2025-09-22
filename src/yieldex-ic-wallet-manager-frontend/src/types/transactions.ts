export type TransactionType =
  | 'smart_wallet_creation'
  | 'deposit'
  | 'withdrawal'
  | 'rebalancing'
  | 'ai_decision'
  | 'opportunity_detection'
  | 'yield_collection'
  | 'protocol_switch';

export type TransactionStatus = 'pending' | 'completed' | 'failed' | 'processing';

export type NetworkType = 'ethereum' | 'arbitrum' | 'icp';

export interface BlockchainReference {
  network: NetworkType;
  txHash?: string;
  blockNumber?: number;
  explorerUrl?: string;
}

export interface RebalanceDetails {
  fromProtocol: string;
  toProtocol: string;
  reason: string;
  oldApy: number;
  newApy: number;
  amount: number;
  token: string;
  potentialSavings?: number;
  actualSavings?: number;
  confidenceScore?: number;
}

export interface Transaction {
  id: string;
  positionId?: string;
  type: TransactionType;
  status: TransactionStatus;
  timestamp: string;

  // Transaction details
  amount?: number;
  token?: string;
  description: string;

  // Blockchain references
  blockchainRefs: BlockchainReference[];

  // Type-specific data
  rebalanceDetails?: RebalanceDetails;

  // UI helpers
  icon: string;
  color: string;

  // Gas and fees
  gasUsed?: number;
  gasCost?: number;
  protocolFee?: number;
}

export interface TransactionGroup {
  id: string;
  type: 'rebalancing_sequence' | 'compound_operation';
  description: string;
  timestamp: string;
  transactions: Transaction[];
  totalGasCost?: number;
  status: TransactionStatus;
  rebalanceDetails?: RebalanceDetails;
  aiDecisionId?: string;
  groupIcon?: string;
}

export interface ActivityFilter {
  types: TransactionType[];
  status: TransactionStatus[];
  dateRange: {
    from: string;
    to: string;
  };
  positionIds: string[];
}