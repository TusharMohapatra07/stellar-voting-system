import { useState, useEffect } from 'react';
import { connectWallet, checkConnection } from '../services/wallet';

interface WalletConnectProps {
  onConnect: (address: string) => void;
  onDisconnect: () => void;
  connectedAddress: string | null;
}

export const WalletConnect: React.FC<WalletConnectProps> = ({
  onConnect,
  onDisconnect,
  connectedAddress,
}) => {
  const [isFreighterInstalled, setIsFreighterInstalled] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const checkFreighter = async () => {
      const installed = !!(window as typeof window & { freighter?: unknown }).freighter;
      setIsFreighterInstalled(installed);
    };
    checkFreighter();
  }, []);

  const handleConnect = async () => {
    setError(null);
    
    if (!isFreighterInstalled) {
      setError('Freighter wallet not detected. Please install it to continue.');
      return;
    }

    try {
      if (!(await checkConnection())) {
        setError('Freighter is locked or not connected.');
        return;
      }
      const address = await connectWallet();
      if (typeof address === 'string') {
        onConnect(address);
      } else if (typeof address === 'object' && address !== null && 'address' in address) {
        onConnect((address as { address: string }).address);
      } else {
        setError('Failed to connect to Freighter.');
      }
    } catch (err: unknown) {
      const error = err as Error;
      setError(error.message || 'Failed to connect');
    }
  };

  const handleDisconnect = () => {
    onDisconnect();
  };

  return (
    <div className="wallet-connect">
      {connectedAddress ? (
        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
          <span style={{ fontSize: '0.9rem', color: '#94a3b8' }}>
            {connectedAddress.slice(0, 6)}...{connectedAddress.slice(-4)}
          </span>
          <button 
            onClick={handleDisconnect}
            style={{ backgroundColor: 'rgba(255, 255, 255, 0.05)', color: '#f8fafc' }}
          >
            Disconnect
          </button>
        </div>
      ) : (
        <button onClick={handleConnect}>Connect Wallet</button>
      )}
      
      {error && (
        <div style={{ position: 'absolute', top: '100%', right: 0, marginTop: '0.5rem', width: '250px' }}>
          <p className="error" style={{ margin: 0, fontSize: '0.8rem', textAlign: 'right' }}>
            {error}
            {!isFreighterInstalled && (
              <>
                {' '}
                <a href="https://www.freighter.app/" target="_blank" rel="noopener noreferrer" style={{ color: '#3b82f6' }}>
                  Install here
                </a>
              </>
            )}
          </p>
        </div>
      )}
    </div>
  );
};
