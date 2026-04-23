import { useState, useEffect } from 'react';
import { VotingContractClient } from '../services/contract';

interface Vote {
  proposal_id: number;
  voter: string;
  choice: number;
  timestamp: number;
}

interface Proposal {
  id: number;
  title: string;
  options: string[];
}

interface VotingHistoryProps {
  voterAddress: string;
}

export const VotingHistory: React.FC<VotingHistoryProps> = ({ voterAddress }) => {
  const [votes, setVotes] = useState<Vote[]>([]);
  const [proposals, setProposals] = useState<Record<number, Proposal>>({});
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const fetchHistory = async () => {
      setLoading(true);
      try {
        const client = new VotingContractClient();
        
        const history = await client.getVoterHistory(voterAddress);
        setVotes(history as Vote[]);

        // Fetch proposals to map titles and options to the history UI
        const props = await client.getProposals(0, 100);
        const propMap: Record<number, Proposal> = {};
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        props.forEach((p: any) => {
          propMap[p.id] = p;
        });
        setProposals(propMap);

      } catch (err) {
        console.error('Failed to fetch voting history:', err);
      } finally {
        setLoading(false);
      }
    };
    if (voterAddress) {
      fetchHistory();
    }
  }, [voterAddress]);

  return (
    <div className="voting-history">
      <h2 style={{ marginBottom: '1.5rem', textAlign: 'center' }}>Your Voting History</h2>
      {loading ? (
        <p style={{ textAlign: 'center' }}>Loading history...</p>
      ) : votes.length === 0 ? (
        <p style={{ textAlign: 'center' }}>No voting history found</p>
      ) : (
        <div className="history-list">
          {votes.map((vote, index) => {
            const proposal = proposals[vote.proposal_id];
            const title = proposal ? proposal.title : `Proposal #${vote.proposal_id}`;
            const choiceText = proposal ? proposal.options[vote.choice] : `Option ${vote.choice}`;

            return (
              <div key={index} className="history-item">
                <h3 style={{ marginTop: 0, marginBottom: '0.75rem', fontSize: '1.25rem', color: '#f8fafc' }}>{title}</h3>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', color: '#cbd5e1' }}>
                  <span style={{ backgroundColor: 'rgba(59, 130, 246, 0.1)', color: '#60a5fa', padding: '0.25rem 0.75rem', borderRadius: '9999px', fontSize: '0.875rem', fontWeight: 500 }}>
                    Voted: {choiceText}
                  </span>
                  <span style={{ fontSize: '0.875rem', opacity: 0.8 }}>
                    {new Date(vote.timestamp * 1000).toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })}
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
