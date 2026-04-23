import { useState, useEffect } from 'react';

interface ErrorDisplayProps {
  error: string | null;
  onRetry?: () => void;
  autoDismiss?: boolean;
  dismissTimeout?: number;
}

export const ErrorDisplay: React.FC<ErrorDisplayProps> = ({
  error,
  onRetry,
  autoDismiss = false,
  dismissTimeout = 5000,
}) => {
  const [hidden, setHidden] = useState(false);
  const [lastError, setLastError] = useState(error);

  if (error !== lastError) {
    setLastError(error);
    setHidden(false);
  }

  useEffect(() => {
    if (error && autoDismiss && !hidden) {
      const timer = setTimeout(() => {
        setHidden(true);
      }, dismissTimeout);
      return () => clearTimeout(timer);
    }
  }, [error, autoDismiss, dismissTimeout, hidden]);

  if (!error || hidden) {
    return null;
  }

  return (
    <div className="error-display">
      <div className="error-content">
        <p>{error}</p>
        <div className="error-actions">
          {onRetry && <button onClick={onRetry}>Retry</button>}
          <button onClick={() => setHidden(true)}>Dismiss</button>
        </div>
      </div>
    </div>
  );
};
