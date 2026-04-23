import { useState, useEffect } from 'react';
import { useAppContext } from '../context/AppContext';
import { VotingContractClient } from '../services/contract';

interface Proposal {
  id: number;
  creator: string;
  title: string;
  description: string;
  voting_start: number;
  voting_end: number;
  created_at: number;
  options: string[];
}

interface VoteResult {
  proposal_id: number;
  option_counts: number[];
  total_votes: number;
  unique_voters: number;
}

interface ProposalDetailProps {
  proposalId: number;
  connectedAddress: string | null;
}

export const ProposalDetail: React.FC<ProposalDetailProps> = ({
  proposalId,
  connectedAddress,
}) => {
  const [proposal, setProposal] = useState<Proposal | null>(null);
  const [voteResults, setVoteResults] = useState<VoteResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [selectedOption, setSelectedOption] = useState<number | null>(null);
  const [currentTime, setCurrentTime] = useState(() => Date.now() / 1000);

  useEffect(() => {
    const fetchData = async () => {
      setLoading(true);
      try {
        const client = new VotingContractClient();
        const prop = await client.getProposal(proposalId);
        const results = await client.getVoteResults(proposalId);
        setProposal(prop as Proposal);
        setVoteResults(results as VoteResult);
      } catch (err) {
        console.error('Failed to fetch proposal details:', err);
      } finally {
        setLoading(false);
      }
    };
    fetchData();

    const interval = setInterval(() => setCurrentTime(Date.now() / 1000), 1000);
    return () => clearInterval(interval);
  }, [proposalId]);

  const { setTransactionPending, setTransactionHash, setTransactionError, setTransactionSuccess } = useAppContext();

  const handleVote = async () => {
    if (selectedOption === null || !connectedAddress) return;
    
    setTransactionPending(true);
    setTransactionError(null);
    setTransactionSuccess(false);

    try {
      const client = new VotingContractClient();
      const res = await client.castVote(proposalId, connectedAddress, selectedOption);
      
      setTransactionHash(res.hash);
      setTransactionSuccess(true);
      
      // Optimistically update
      if (voteResults) {
        const newCounts = [...voteResults.option_counts];
        newCounts[selectedOption] = (newCounts[selectedOption] || 0) + 1;
        setVoteResults({
          ...voteResults,
          option_counts: newCounts,
          total_votes: voteResults.total_votes + 1,
          unique_voters: voteResults.unique_voters + 1,
        });
      }
    } catch (err: unknown) {
      const error = err as Error;
      setTransactionError(error.message || 'Failed to cast vote');
      console.error('Vote error:', err);
    } finally {
      setTransactionPending(false);
    }
  };

  if (loading) {
    return <div>Loading proposal...</div>;
  }

  if (!proposal) {
    return <div>Proposal not found</div>;
  }

  const isActive = currentTime < proposal.voting_end;

  return (
    <div className="proposal-detail">
      <h2>{proposal.title}</h2>
      <p>{proposal.description}</p>
      <p>Creator: {proposal.creator}</p>
      <p>Status: {isActive ? 'Active' : 'Closed'}</p>

      <div className="vote-results">
        <h3>Vote Results</h3>
        {voteResults && (
          <div>
            <p style={{ marginBottom: '1.5rem', color: '#aaa' }}>
              Total Votes: {voteResults.total_votes} | Unique Voters: {voteResults.unique_voters}
            </p>
            {proposal.options.map((option, index) => {
              const count = voteResults.option_counts[index] || 0;
              const percentage = voteResults.total_votes > 0 
                ? Math.round((count / voteResults.total_votes) * 100) 
                : 0;
                
              return (
                <div key={index} className="vote-option-result">
                  <div className="vote-option-header">
                    <span>{option}</span>
                    <span>{count} votes ({percentage}%)</span>
                  </div>
                  <div className="progress-bar-container">
                    <div 
                      className="progress-bar-fill" 
                      style={{ width: `${percentage}%` }}
                    ></div>
                    <div className="progress-bar-text">
                      {percentage}%
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {isActive && connectedAddress && (
        <div className="voting-section">
          <h3>Cast Your Vote</h3>
          {proposal.options.map((option, index) => (
            <div key={index}>
              <label>
                <input
                  type="radio"
                  name="vote"
                  value={index}
                  onChange={() => setSelectedOption(index)}
                />
                {option}
              </label>
            </div>
          ))}
          <button onClick={handleVote} disabled={selectedOption === null}>
            Submit Vote
          </button>
        </div>
      )}
    </div>
  );
};
