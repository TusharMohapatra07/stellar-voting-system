import { useNavigate } from 'react-router-dom';
import { CreateProposal } from '../components/CreateProposal';

export const CreateProposalPage: React.FC = () => {
  const navigate = useNavigate();

  const handleProposalCreated = () => {
    navigate('/');
  };

  return (
    <div className="create-proposal-page">
      <CreateProposal onProposalCreated={handleProposalCreated} />
    </div>
  );
};
