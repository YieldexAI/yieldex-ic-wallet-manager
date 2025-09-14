// Number and currency formatting utilities

export const formatNumber = (
  value: number,
  options?: Intl.NumberFormatOptions
): string => {
  return new Intl.NumberFormat('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
    ...options
  }).format(value);
};

export const formatCurrency = (
  value: number,
  currency: string = 'USD',
  options?: Intl.NumberFormatOptions
): string => {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency,
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
    ...options
  }).format(value);
};

export const formatCompactNumber = (value: number): string => {
  return new Intl.NumberFormat('en-US', {
    notation: 'compact',
    maximumFractionDigits: 1
  }).format(value);
};

export const formatPercentage = (
  value: number,
  decimals: number = 2
): string => {
  return new Intl.NumberFormat('en-US', {
    style: 'percent',
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals
  }).format(value / 100);
};

export const formatTokenAmount = (
  amount: number,
  symbol: string,
  decimals: number = 6
): string => {
  const formatted = formatNumber(amount, {
    minimumFractionDigits: decimals === 18 ? 4 : 2,
    maximumFractionDigits: decimals === 18 ? 6 : decimals
  });
  return `${formatted} ${symbol}`;
};

export const formatAPY = (apy: number): string => {
  return `${formatNumber(apy)}%`;
};

export const formatTVL = (tvl: number): string => {
  if (tvl >= 1_000_000_000) {
    return `$${formatNumber(tvl / 1_000_000_000, { maximumFractionDigits: 1 })}B`;
  } else if (tvl >= 1_000_000) {
    return `$${formatNumber(tvl / 1_000_000, { maximumFractionDigits: 1 })}M`;
  } else if (tvl >= 1_000) {
    return `$${formatNumber(tvl / 1_000, { maximumFractionDigits: 1 })}K`;
  }
  return formatCurrency(tvl);
};

export const formatAddress = (
  address: string,
  startLength: number = 6,
  endLength: number = 4
): string => {
  if (address.length <= startLength + endLength) {
    return address;
  }
  return `${address.slice(0, startLength)}...${address.slice(-endLength)}`;
};

export const formatTimeAgo = (date: Date | string): string => {
  const now = new Date();
  const then = new Date(date);
  const diffMs = now.getTime() - then.getTime();
  
  const diffMinutes = Math.floor(diffMs / (1000 * 60));
  const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
  
  if (diffMinutes < 1) {
    return 'Just now';
  } else if (diffMinutes < 60) {
    return `${diffMinutes}m ago`;
  } else if (diffHours < 24) {
    return `${diffHours}h ago`;
  } else if (diffDays < 7) {
    return `${diffDays}d ago`;
  } else {
    return then.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: then.getFullYear() !== now.getFullYear() ? 'numeric' : undefined
    });
  }
};

export const formatDuration = (seconds: number): string => {
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);
  
  if (days > 0) {
    return `${days}d ${hours % 24}h`;
  } else if (hours > 0) {
    return `${hours}h ${minutes % 60}m`;
  } else {
    return `${minutes}m ${seconds % 60}s`;
  }
};

export const formatGas = (gasUsed: number, gasPrice: number): string => {
  const gasCost = (gasUsed * gasPrice) / 1e9; // Convert to ETH
  return `${formatNumber(gasCost, { maximumFractionDigits: 6 })} ETH`;
};

export const formatTransactionHash = (hash: string): string => {
  return formatAddress(hash, 8, 6);
};

// Color utilities for APY display
export const getAPYColor = (apy: number): string => {
  if (apy >= 20) return 'text-red-400'; // High risk
  if (apy >= 10) return 'text-yellow-400'; // Moderate risk
  if (apy >= 5) return 'text-green-400'; // Conservative
  return 'text-gray-400'; // Low yield
};

export const getRiskColor = (risk: 'conservative' | 'moderate' | 'aggressive'): string => {
  switch (risk) {
    case 'conservative':
      return 'text-green-400 bg-green-400/10 border-green-400/20';
    case 'moderate':
      return 'text-yellow-400 bg-yellow-400/10 border-yellow-400/20';
    case 'aggressive':
      return 'text-red-400 bg-red-400/10 border-red-400/20';
    default:
      return 'text-gray-400 bg-gray-400/10 border-gray-400/20';
  }
};

export const getNetworkColor = (networkSlug: string): string => {
  const colors: Record<string, string> = {
    ethereum: 'text-gray-300 bg-gray-600/20 border-gray-500/30',
    arbitrum: 'text-blue-300 bg-blue-600/20 border-blue-500/30',
    polygon: 'text-purple-300 bg-purple-600/20 border-purple-500/30',
    bsc: 'text-yellow-300 bg-yellow-600/20 border-yellow-500/30'
  };
  return colors[networkSlug] || 'text-gray-300 bg-gray-600/20 border-gray-500/30';
};

// Animation-friendly number formatting
export const formatAnimatedNumber = (
  value: number,
  formatFn: (val: number) => string = formatNumber
): string => {
  // For smooth animations, we might want to interpolate between values
  return formatFn(value);
};

// Validation utilities
export const isValidAmount = (amount: string | number): boolean => {
  const num = typeof amount === 'string' ? parseFloat(amount) : amount;
  return !isNaN(num) && num > 0 && isFinite(num);
};

export const parseTokenAmount = (amount: string, decimals: number = 18): bigint => {
  if (!isValidAmount(amount)) {
    throw new Error('Invalid amount');
  }
  
  const num = parseFloat(amount);
  const multiplier = Math.pow(10, decimals);
  return BigInt(Math.floor(num * multiplier));
};

export const formatTokenFromBigInt = (
  amount: bigint,
  decimals: number = 18,
  displayDecimals: number = 6
): string => {
  const divisor = BigInt(Math.pow(10, decimals));
  const wholePart = amount / divisor;
  const fractionalPart = amount % divisor;
  
  const fractionalString = fractionalPart.toString().padStart(decimals, '0');
  const trimmedFractional = fractionalString.slice(0, displayDecimals);
  
  const result = `${wholePart}.${trimmedFractional}`;
  return parseFloat(result).toString();
};