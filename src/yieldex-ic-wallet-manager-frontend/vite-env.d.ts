/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_WALLETCONNECT_PROJECT_ID: string
  readonly VITE_DEMO_MODE: string
  readonly VITE_IC_CANISTER_ID: string
  readonly VITE_NETWORK_RPC_URLS: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

// Extend Window interface for wallet detection
declare global {
  interface Window {
    ethereum?: {
      isMetaMask?: boolean
      isTrust?: boolean
      isCoinbaseWallet?: boolean
      request?: (args: { method: string; params?: any[] }) => Promise<any>
    }
  }
}