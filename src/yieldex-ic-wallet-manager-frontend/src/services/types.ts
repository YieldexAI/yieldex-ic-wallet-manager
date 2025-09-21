// Alchemy API Types for Token Balances

export interface TokenBalance {
  contractAddress: string;
  tokenBalance: string; // Raw balance in string format (e.g., "21000000")
  error?: string | null;
}

export interface TokenBalancesResponse {
  address: string;
  tokenBalances: TokenBalance[];
}

export interface TokenMetadata {
  decimals: number;
  logo?: string;
  name: string;
  symbol: string;
}

export interface TokenMetadataResponse {
  [contractAddress: string]: TokenMetadata;
}

// Processed token balance with human-readable amount
export interface ProcessedTokenBalance {
  contractAddress: string;
  symbol: string;
  name: string;
  balance: string; // Human-readable balance (e.g., "21.0")
  rawBalance: string; // Raw balance from API
  usdValue?: number; // USD value if available
  network: string;
  logo?: string;
  decimals: number;
}

// Network configuration
export interface NetworkConfig {
  id: number;
  name: string;
  slug: string;
  alchemyNetwork: string; // Alchemy-specific network identifier
  rpcUrl: string;
}

// Stablecoin configuration
export interface StablecoinConfig {
  symbol: string;
  name: string;
  contracts: {
    [networkSlug: string]: string; // Contract address per network
  };
  decimals: number;
  logo?: string;
}

// API Error types
export interface AlchemyError {
  code: number;
  message: string;
}

// Portfolio summary
export interface PortfolioSummary {
  totalUsdValue: number;
  totalTokens: number;
  stablecoins: ProcessedTokenBalance[];
  otherTokens: ProcessedTokenBalance[];
  networks: string[];
}