// Core data types
export interface Proposal {
  id: number;
  creator: string;
  title: string;
  description: string;
  voting_start: number;
  voting_end: number;
  created_at: number;
  options: string[];
}

export interface Vote {
  proposal_id: number;
  voter: string;
  choice: number;
  timestamp: number;
}

export interface VoteResult {
  proposal_id: number;
  option_counts: number[];
  total_votes: number;
  unique_voters: number;
}

// Application state types
export interface WalletState {
  address: string | null;
  connected: boolean;
  isFreighterInstalled: boolean;
}

export interface ProposalsState {
  items: Proposal[];
  loading: boolean;
  error: string | null;
  filter: 'all' | 'active' | 'closed';
  currentPage: number;
  totalPages: number;
}

export interface SelectedProposalState {
  proposal: Proposal | null;
  voteResults: VoteResult | null;
  userVote: Vote | null;
  loading: boolean;
  error: string | null;
}

export interface VotingHistoryState {
  votes: Vote[];
  loading: boolean;
  error: string | null;
}

export interface TransactionState {
  pending: boolean;
  hash: string | null;
  error: string | null;
  success: boolean;
}

export interface AppState {
  wallet: WalletState;
  proposals: ProposalsState;
  selectedProposal: SelectedProposalState;
  votingHistory: VotingHistoryState;
  transaction: TransactionState;
}
