import { useEffect } from 'react';
import { useAccount, useChainId } from 'wagmi';
import { useWalletStore } from '@/stores/walletStore';
import { NETWORKS } from '@/mock/protocols';

/**
 * Integration hook between wagmi (real wallet) and our Zustand store (demo data)
 * This bridges the gap between real wallet connection and our demo interface
 */
export const useWalletIntegration = () => {
  const { address, isConnected } = useAccount();
  const chainId = useChainId();
  
  // Get our store actions
  const {
    isConnected: storeConnected,
    address: storeAddress,
    networkId: storeNetworkId,
    switchNetwork,
    // We'll add a new action to sync real wallet data
  } = useWalletStore();

  // Sync real wallet connection to our store
  useEffect(() => {
    if (isConnected && address && chainId) {
      // Map wagmi chain to our network system
      const supportedNetwork = NETWORKS.find(n => n.id === chainId);
      
      if (supportedNetwork) {
        // Update our store with real wallet data
        useWalletStore.setState({
          isConnected: true,
          address: address,
          evmAddress: address, // Same as address for real wallets
          networkId: chainId,
          principal: `ic-${address.slice(0, 8)}`, // Generate mock principal
        });
      }
    } else if (!isConnected) {
      // Clear store when wallet disconnects
      useWalletStore.setState({
        isConnected: false,
        address: '',
        evmAddress: '',
        principal: '',
        networkId: 1, // Default to Ethereum
      });
    }
  }, [isConnected, address, chainId]);

  // Sync network changes
  useEffect(() => {
    if (chainId && chainId !== storeNetworkId) {
      switchNetwork(chainId);
    }
  }, [chainId, storeNetworkId, switchNetwork]);

  return {
    // Real wallet data
    realAddress: address,
    realChainId: chainId,
    realIsConnected: isConnected,
    
    // Store data (for demo interface compatibility)
    storeAddress,
    storeNetworkId,
    storeConnected,
    
    // Combined state
    isWalletReady: isConnected && chainId && storeConnected,
    currentNetwork: NETWORKS.find(n => n.id === (chainId || storeNetworkId)),
  };
};