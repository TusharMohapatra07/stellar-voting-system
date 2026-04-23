import { useState } from 'react';
import { useAppContext } from '../context/AppContext';
import { VotingContractClient } from '../services/contract';

interface CreateProposalProps {
  onProposalCreated: () => void;
}

export const CreateProposal: React.FC<CreateProposalProps> = ({ onProposalCreated }) => {
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [votingEnd, setVotingEnd] = useState('');
  const [options, setOptions] = useState<string[]>(['', '']);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [submitting, setSubmitting] = useState(false);

  const validateForm = () => {
    const newErrors: Record<string, string> = {};

    if (title.length < 5 || title.length > 200) {
      newErrors.title = 'Title must be between 5 and 200 characters';
    }

    if (description.length < 20 || description.length > 5000) {
      newErrors.description = 'Description must be between 20 and 5000 characters';
    }

    const votingEndDate = new Date(votingEnd);
    const now = new Date();
    const oneHour = 60 * 60 * 1000;
    const ninetyDays = 90 * 24 * 60 * 60 * 1000;
    const timeDiff = votingEndDate.getTime() - now.getTime();

    if (timeDiff < oneHour || timeDiff > ninetyDays) {
      newErrors.votingEnd = 'Voting end must be between 1 hour and 90 days from now';
    }

    const validOptions = options.filter((opt) => opt.trim().length > 0);
    if (validOptions.length < 2 || validOptions.length > 10) {
      newErrors.options = 'Must have between 2 and 10 options';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const { state, setTransactionPending, setTransactionHash, setTransactionError, setTransactionSuccess } = useAppContext();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!validateForm()) return;

    const creator = state.wallet.address;
    if (!creator) {
      setErrors({ form: 'Wallet not connected' });
      return;
    }

    setSubmitting(true);
    setTransactionPending(true);
    setTransactionError(null);
    setTransactionSuccess(false);

    try {
      const client = new VotingContractClient();
      const endTimestamp = Math.floor(new Date(votingEnd).getTime() / 1000);
      const validOptions = options.filter(o => o.trim());
      const res = await client.createProposal(creator, title, description, endTimestamp, validOptions);
      
      setTransactionHash(res.hash);
      setTransactionSuccess(true);
      onProposalCreated();
    } catch (err: unknown) {
      const error = err as Error;
      setTransactionError(error.message || 'Failed to create proposal');
      setErrors({ form: error.message || 'Failed to create proposal' });
    } finally {
      setSubmitting(false);
      setTransactionPending(false);
    }
  };

  const addOption = () => {
    if (options.length < 10) {
      setOptions([...options, '']);
    }
  };

  const removeOption = (index: number) => {
    if (options.length > 2) {
      setOptions(options.filter((_, i) => i !== index));
    }
  };

  const updateOption = (index: number, value: string) => {
    const newOptions = [...options];
    newOptions[index] = value;
    setOptions(newOptions);
  };

  return (
    <div className="create-proposal">
      <h2 style={{ marginBottom: '1.5rem', textAlign: 'center' }}>Create New Proposal</h2>
      <form onSubmit={handleSubmit}>
        <div className="form-group">
          <label>Proposal Title</label>
          <input
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            maxLength={200}
            placeholder="e.g., Allocate Community Treasury Funds"
          />
          {errors.title && <p className="error">{errors.title}</p>}
        </div>

        <div className="form-group">
          <label>Detailed Description</label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            maxLength={5000}
            rows={5}
            placeholder="Provide context, goals, and implications of this proposal..."
          />
          {errors.description && <p className="error">{errors.description}</p>}
        </div>

        <div className="form-group">
          <label>Voting End Date & Time</label>
          <input
            type="datetime-local"
            value={votingEnd}
            onChange={(e) => setVotingEnd(e.target.value)}
          />
          {errors.votingEnd && <p className="error">{errors.votingEnd}</p>}
        </div>

        <div className="form-group">
          <label>Voting Options</label>
          <div className="options-container">
            {options.map((option, index) => (
              <div key={index} className="option-row">
                <input
                  type="text"
                  value={option}
                  onChange={(e) => updateOption(index, e.target.value)}
                  placeholder={`Option ${index + 1}`}
                  maxLength={100}
                />
                {options.length > 2 && (
                  <button type="button" className="btn-remove" onClick={() => removeOption(index)} title="Remove Option">
                    ✕
                  </button>
                )}
              </div>
            ))}
            {options.length < 10 && (
              <button type="button" className="btn-add" onClick={addOption}>
                + Add Another Option
              </button>
            )}
          </div>
          {errors.options && <p className="error">{errors.options}</p>}
        </div>

        {errors.form && <p className="error" style={{ textAlign: 'center' }}>{errors.form}</p>}

        <button type="submit" className="btn-submit" disabled={submitting}>
          {submitting ? 'Creating Proposal...' : 'Submit Proposal'}
        </button>
      </form>
    </div>
  );
};
