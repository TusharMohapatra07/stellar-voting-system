import { useParams } from 'react-router-dom';
import { ProposalDetail } from '../components/ProposalDetail';

interface ProposalPageProps {
  connectedAddress: string | null;
}

export const ProposalPage: React.FC<ProposalPageProps> = ({ connectedAddress }) => {
  const { id } = useParams<{ id: string }>();
  const proposalId = id ? parseInt(id, 10) : 0;

  return (
    <div className="proposal-page">
      <ProposalDetail proposalId={proposalId} connectedAddress={connectedAddress} />
    </div>
  );
};
