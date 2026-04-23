import { ProposalList } from '../components/ProposalList';
import { useNavigate } from 'react-router-dom';

export const HomePage: React.FC = () => {
  const navigate = useNavigate();

  const handleSelectProposal = (proposalId: number) => {
    navigate(`/proposal/${proposalId}`);
  };

  return (
    <div className="home-page">
      <h1>Stellar Voting DApp</h1>
      <ProposalList onSelectProposal={handleSelectProposal} />
    </div>
  );
};
