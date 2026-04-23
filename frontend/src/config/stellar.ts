// Stellar network configuration
export const STELLAR_CONFIG = {
  // Network settings
  network: {
    testnet: {
      networkPassphrase: 'Test SDF Network ; September 2015',
      rpcUrl: import.meta.env.VITE_STELLAR_RPC_URL || 'https://soroban-testnet.stellar.org',
      horizonUrl: 'https://horizon-testnet.stellar.org',
    },
    mainnet: {
      networkPassphrase: 'Public Global Stellar Network ; September 2015',
      rpcUrl: 'https://soroban.stellar.org',
      horizonUrl: 'https://horizon.stellar.org',
    },
  },

  // Current network (can be changed via environment variable)
  currentNetwork: (import.meta.env.VITE_STELLAR_NETWORK || 'testnet') as 'testnet' | 'mainnet',

  // Contract ID (to be set after deployment)
  contractId: import.meta.env.VITE_CONTRACT_ID || '',

  // Network passphrase from env (for flexibility)
  networkPassphrase: import.meta.env.VITE_STELLAR_NETWORK_PASSPHRASE || 'Test SDF Network ; September 2015',
};

// Helper to get current network config
export const getNetworkConfig = () => {
  return STELLAR_CONFIG.network[STELLAR_CONFIG.currentNetwork];
};

// Helper to get network passphrase
export const getNetworkPassphrase = () => {
  return getNetworkConfig().networkPassphrase;
};

// Helper to get RPC URL
export const getRpcUrl = () => {
  return getNetworkConfig().rpcUrl;
};

// Helper to get Horizon URL
export const getHorizonUrl = () => {
  return getNetworkConfig().horizonUrl;
};
