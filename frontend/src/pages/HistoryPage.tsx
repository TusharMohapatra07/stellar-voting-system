import { VotingHistory } from '../components/VotingHistory';

interface HistoryPageProps {
  connectedAddress: string | null;
}

export const HistoryPage: React.FC<HistoryPageProps> = ({ connectedAddress }) => {
  if (!connectedAddress) {
    return (
      <div className="history-page">
        <p>Please connect your wallet to view voting history</p>
      </div>
    );
  }

  return (
    <div className="history-page">
      <VotingHistory voterAddress={connectedAddress} />
    </div>
  );
};
