import { useState, useEffect } from 'react';
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

interface ProposalListProps {
  onSelectProposal: (proposalId: number) => void;
}

export const ProposalList: React.FC<ProposalListProps> = ({ onSelectProposal }) => {
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [loading, setLoading] = useState(false);
  const [filter, setFilter] = useState<'all' | 'active' | 'closed'>('all');

  const [currentTime, setCurrentTime] = useState(() => Date.now() / 1000);

  useEffect(() => {
    const fetchProposals = async () => {
      setLoading(true);
      try {
        const client = new VotingContractClient();
        const props = await client.getProposals(0, 100);
        setProposals(props);
      } catch (err) {
        console.error('Failed to fetch proposals:', err);
      } finally {
        setLoading(false);
      }
    };
    fetchProposals();
    
    const interval = setInterval(() => setCurrentTime(Date.now() / 1000), 1000);
    return () => clearInterval(interval);
  }, [filter]);

  const isActive = (proposal: Proposal) => {
    return currentTime < proposal.voting_end;
  };

  const filteredProposals = proposals.filter((p) => {
    if (filter === 'all') return true;
    if (filter === 'active') return isActive(p);
    if (filter === 'closed') return !isActive(p);
    return true;
  });

  return (
    <div className="proposal-list">
      <div className="filters">
        <button onClick={() => setFilter('all')}>All</button>
        <button onClick={() => setFilter('active')}>Active</button>
        <button onClick={() => setFilter('closed')}>Closed</button>
      </div>
      {loading ? (
        <p>Loading proposals...</p>
      ) : (
        <div className="proposals">
          {filteredProposals.length === 0 ? (
            <p>No proposals found</p>
          ) : (
            filteredProposals.map((proposal) => (
              <div
                key={proposal.id}
                className="proposal-item"
                onClick={() => onSelectProposal(proposal.id)}
              >
                <h3>{proposal.title}</h3>
                <p>Creator: {proposal.creator.slice(0, 8)}...</p>
                <p>Status: {isActive(proposal) ? 'Active' : 'Closed'}</p>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
};
