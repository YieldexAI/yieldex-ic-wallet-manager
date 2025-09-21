import { Alchemy, Network } from 'alchemy-sdk';
import {
  TokenBalancesResponse,
  ProcessedTokenBalance,
  PortfolioSummary,
  NetworkConfig,
  StablecoinConfig
} from './types';
import {
  NETWORKS,
  STABLECOINS,
  getStablecoinContractsForNetwork,
  getStablecoinByContract,
  DEFAULT_NETWORKS
} from './constants';

// Create Alchemy instances for different networks
const createAlchemyInstances = (): Record<string, Alchemy> => {
  const instances: Record<string, Alchemy> = {};
  const apiKey = import.meta.env.VITE_ALCHEMY_API_KEY;

  if (!apiKey) {
    throw new Error('VITE_ALCHEMY_API_KEY is not set');
  }

  // Create instances for each supported network
  instances.ethereum = new Alchemy({
    apiKey,
    network: Network.ETH_MAINNET
  });

  instances.arbitrum = new Alchemy({
    apiKey,
    network: Network.ARB_MAINNET
  });

  instances.polygon = new Alchemy({
    apiKey,
    network: Network.MATIC_MAINNET
  });

  instances.base = new Alchemy({
    apiKey,
    network: Network.BASE_MAINNET
  });

  return instances;
};

const alchemyInstances = createAlchemyInstances();

// Utility function to convert raw balance to human-readable format
const formatTokenBalance = (
  rawBalance: string,
  decimals: number
): string => {
  if (!rawBalance || rawBalance === '0') return '0';

  const balance = BigInt(rawBalance);
  const divisor = BigInt(10 ** decimals);
  const quotient = balance / divisor;
  const remainder = balance % divisor;

  if (remainder === BigInt(0)) {
    return quotient.toString();
  }

  const remainderStr = remainder.toString().padStart(decimals, '0');
  const trimmedRemainder = remainderStr.replace(/0+$/, '');

  if (trimmedRemainder === '') {
    return quotient.toString();
  }

  return `${quotient}.${trimmedRemainder}`;
};

// Fetch token balances for a specific network
export const fetchTokenBalancesForNetwork = async (
  walletAddress: string,
  networkSlug: string
): Promise<ProcessedTokenBalance[]> => {
  try {
    const alchemy = alchemyInstances[networkSlug];
    const contractAddresses = getStablecoinContractsForNetwork(networkSlug);

    if (!alchemy || contractAddresses.length === 0) {
      return [];
    }

    // Get token balances using Alchemy SDK
    const balances = await alchemy.core.getTokenBalances(walletAddress, contractAddresses);

    const processedBalances: ProcessedTokenBalance[] = [];

    for (const tokenBalance of balances.tokenBalances) {
      if (tokenBalance.error || !tokenBalance.tokenBalance || tokenBalance.tokenBalance === '0x0') {
        continue;
      }

      const stablecoin = getStablecoinByContract(tokenBalance.contractAddress, networkSlug);

      if (!stablecoin) {
        continue;
      }

      // Convert hex to decimal if needed
      const rawBalance = tokenBalance.tokenBalance.startsWith('0x')
        ? BigInt(tokenBalance.tokenBalance).toString()
        : tokenBalance.tokenBalance;

      const formattedBalance = formatTokenBalance(rawBalance, stablecoin.decimals);

      // Skip if balance is zero
      if (formattedBalance === '0') {
        continue;
      }

      processedBalances.push({
        contractAddress: tokenBalance.contractAddress,
        symbol: stablecoin.symbol,
        name: stablecoin.name,
        balance: formattedBalance,
        rawBalance,
        network: networkSlug,
        decimals: stablecoin.decimals,
        logo: stablecoin.logo
      });
    }

    return processedBalances;
  } catch (error) {
    console.error(`Error fetching balances for ${networkSlug}:`, error);
    return [];
  }
};

// Fetch token balances across all supported networks
export const fetchAllTokenBalances = async (
  walletAddress: string,
  networks: string[] = DEFAULT_NETWORKS
): Promise<ProcessedTokenBalance[]> => {
  try {
    const promises = networks.map(network =>
      fetchTokenBalancesForNetwork(walletAddress, network)
    );

    const results = await Promise.allSettled(promises);
    const allBalances: ProcessedTokenBalance[] = [];

    results.forEach((result, index) => {
      if (result.status === 'fulfilled') {
        allBalances.push(...result.value);
      } else {
        console.error(`Failed to fetch balances for ${networks[index]}:`, result.reason);
      }
    });

    return allBalances;
  } catch (error) {
    console.error('Error fetching all token balances:', error);
    return [];
  }
};

// Get portfolio summary with aggregated stablecoin balances
export const getPortfolioSummary = async (
  walletAddress: string
): Promise<PortfolioSummary> => {
  const allBalances = await fetchAllTokenBalances(walletAddress);

  // Group balances by token symbol and sum them up
  const tokenSummary: Record<string, {
    totalBalance: number;
    networks: string[];
    details: ProcessedTokenBalance[];
  }> = {};

  allBalances.forEach(balance => {
    const symbol = balance.symbol;
    const numericBalance = parseFloat(balance.balance);

    if (!tokenSummary[symbol]) {
      tokenSummary[symbol] = {
        totalBalance: 0,
        networks: [],
        details: []
      };
    }

    tokenSummary[symbol].totalBalance += numericBalance;
    if (!tokenSummary[symbol].networks.includes(balance.network)) {
      tokenSummary[symbol].networks.push(balance.network);
    }
    tokenSummary[symbol].details.push(balance);
  });

  // For now, we'll estimate USD value as 1:1 for stablecoins
  // In a real implementation, you'd fetch actual prices
  const totalUsdValue = Object.values(tokenSummary).reduce(
    (sum, token) => sum + token.totalBalance,
    0
  );

  return {
    totalUsdValue,
    totalTokens: Object.keys(tokenSummary).length,
    stablecoins: allBalances,
    otherTokens: [],
    networks: [...new Set(allBalances.map(b => b.network))]
  };
};

// Get aggregated balance for a specific stablecoin across all networks
export const getStablecoinTotalBalance = async (
  walletAddress: string,
  stablecoinSymbol: string
): Promise<{
  totalBalance: string;
  networkBreakdown: Array<{
    network: string;
    balance: string;
    usdValue: number;
  }>;
}> => {
  const allBalances = await fetchAllTokenBalances(walletAddress);
  const stablecoinBalances = allBalances.filter(
    balance => balance.symbol === stablecoinSymbol
  );

  let totalBalance = 0;
  const networkBreakdown = stablecoinBalances.map(balance => {
    const numericBalance = parseFloat(balance.balance);
    totalBalance += numericBalance;

    return {
      network: balance.network,
      balance: balance.balance,
      usdValue: numericBalance // 1:1 for stablecoins
    };
  });

  return {
    totalBalance: totalBalance.toFixed(6),
    networkBreakdown
  };
};

// Test function to verify API connection
export const testAlchemyConnection = async (): Promise<boolean> => {
  try {
    // Test with a known address that has USDC balance (Binance hot wallet)
    const testAddress = '0xF977814e90dA44bFA03b6295A0616a897441aceC';
    const testBalance = await fetchTokenBalancesForNetwork(testAddress, 'ethereum');

    console.log('Alchemy connection test:', testBalance.length > 0 ? 'SUCCESS' : 'NO BALANCES');
    return true;
  } catch (error) {
    console.error('Alchemy connection test failed:', error);
    return false;
  }
};