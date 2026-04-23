import React, { createContext, useContext, useState, ReactNode } from 'react';
import type {
  AppState,
  WalletState,
  ProposalsState,
  SelectedProposalState,
  VotingHistoryState,
  TransactionState,
  Proposal,
  Vote,
  VoteResult,
} from '../types';

// Initial states
const initialWalletState: WalletState = {
  address: null,
  connected: false,
  isFreighterInstalled: false,
};

const initialProposalsState: ProposalsState = {
  items: [],
  loading: false,
  error: null,
  filter: 'all',
  currentPage: 1,
  totalPages: 1,
};

const initialSelectedProposalState: SelectedProposalState = {
  proposal: null,
  voteResults: null,
  userVote: null,
  loading: false,
  error: null,
};

const initialVotingHistoryState: VotingHistoryState = {
  votes: [],
  loading: false,
  error: null,
};

const initialTransactionState: TransactionState = {
  pending: false,
  hash: null,
  error: null,
  success: false,
};

// Context type
interface AppContextType {
  state: AppState;
  // Wallet actions
  connectWallet: (address: string) => void;
  disconnectWallet: () => void;
  setFreighterInstalled: (installed: boolean) => void;
  // Proposals actions
  setProposals: (proposals: Proposal[]) => void;
  setProposalsLoading: (loading: boolean) => void;
  setProposalsError: (error: string | null) => void;
  setProposalsFilter: (filter: 'all' | 'active' | 'closed') => void;
  setCurrentPage: (page: number) => void;
  // Selected proposal actions
  setSelectedProposal: (proposal: Proposal | null) => void;
  setVoteResults: (results: VoteResult | null) => void;
  setUserVote: (vote: Vote | null) => void;
  setSelectedProposalLoading: (loading: boolean) => void;
  setSelectedProposalError: (error: string | null) => void;
  // Voting history actions
  setVotingHistory: (votes: Vote[]) => void;
  setVotingHistoryLoading: (loading: boolean) => void;
  setVotingHistoryError: (error: string | null) => void;
  // Transaction actions
  setTransactionPending: (pending: boolean) => void;
  setTransactionHash: (hash: string | null) => void;
  setTransactionError: (error: string | null) => void;
  setTransactionSuccess: (success: boolean) => void;
  clearTransaction: () => void;
}

const AppContext = createContext<AppContextType | undefined>(undefined);

export const AppProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [state, setState] = useState<AppState>({
    wallet: initialWalletState,
    proposals: initialProposalsState,
    selectedProposal: initialSelectedProposalState,
    votingHistory: initialVotingHistoryState,
    transaction: initialTransactionState,
  });

  // Wallet actions
  const connectWallet = (address: string) => {
    setState((prev) => ({
      ...prev,
      wallet: { ...prev.wallet, address, connected: true },
    }));
  };

  const disconnectWallet = () => {
    setState((prev) => ({
      ...prev,
      wallet: { ...prev.wallet, address: null, connected: false },
    }));
  };

  const setFreighterInstalled = (installed: boolean) => {
    setState((prev) => ({
      ...prev,
      wallet: { ...prev.wallet, isFreighterInstalled: installed },
    }));
  };

  // Proposals actions
  const setProposals = (proposals: Proposal[]) => {
    setState((prev) => ({
      ...prev,
      proposals: { ...prev.proposals, items: proposals },
    }));
  };

  const setProposalsLoading = (loading: boolean) => {
    setState((prev) => ({
      ...prev,
      proposals: { ...prev.proposals, loading },
    }));
  };

  const setProposalsError = (error: string | null) => {
    setState((prev) => ({
      ...prev,
      proposals: { ...prev.proposals, error },
    }));
  };

  const setProposalsFilter = (filter: 'all' | 'active' | 'closed') => {
    setState((prev) => ({
      ...prev,
      proposals: { ...prev.proposals, filter },
    }));
  };

  const setCurrentPage = (page: number) => {
    setState((prev) => ({
      ...prev,
      proposals: { ...prev.proposals, currentPage: page },
    }));
  };

  // Selected proposal actions
  const setSelectedProposal = (proposal: Proposal | null) => {
    setState((prev) => ({
      ...prev,
      selectedProposal: { ...prev.selectedProposal, proposal },
    }));
  };

  const setVoteResults = (results: VoteResult | null) => {
    setState((prev) => ({
      ...prev,
      selectedProposal: { ...prev.selectedProposal, voteResults: results },
    }));
  };

  const setUserVote = (vote: Vote | null) => {
    setState((prev) => ({
      ...prev,
      selectedProposal: { ...prev.selectedProposal, userVote: vote },
    }));
  };

  const setSelectedProposalLoading = (loading: boolean) => {
    setState((prev) => ({
      ...prev,
      selectedProposal: { ...prev.selectedProposal, loading },
    }));
  };

  const setSelectedProposalError = (error: string | null) => {
    setState((prev) => ({
      ...prev,
      selectedProposal: { ...prev.selectedProposal, error },
    }));
  };

  // Voting history actions
  const setVotingHistory = (votes: Vote[]) => {
    setState((prev) => ({
      ...prev,
      votingHistory: { ...prev.votingHistory, votes },
    }));
  };

  const setVotingHistoryLoading = (loading: boolean) => {
    setState((prev) => ({
      ...prev,
      votingHistory: { ...prev.votingHistory, loading },
    }));
  };

  const setVotingHistoryError = (error: string | null) => {
    setState((prev) => ({
      ...prev,
      votingHistory: { ...prev.votingHistory, error },
    }));
  };

  // Transaction actions
  const setTransactionPending = (pending: boolean) => {
    setState((prev) => ({
      ...prev,
      transaction: { ...prev.transaction, pending },
    }));
  };

  const setTransactionHash = (hash: string | null) => {
    setState((prev) => ({
      ...prev,
      transaction: { ...prev.transaction, hash },
    }));
  };

  const setTransactionError = (error: string | null) => {
    setState((prev) => ({
      ...prev,
      transaction: { ...prev.transaction, error },
    }));
  };

  const setTransactionSuccess = (success: boolean) => {
    setState((prev) => ({
      ...prev,
      transaction: { ...prev.transaction, success },
    }));
  };

  const clearTransaction = () => {
    setState((prev) => ({
      ...prev,
      transaction: initialTransactionState,
    }));
  };

  const value: AppContextType = {
    state,
    connectWallet,
    disconnectWallet,
    setFreighterInstalled,
    setProposals,
    setProposalsLoading,
    setProposalsError,
    setProposalsFilter,
    setCurrentPage,
    setSelectedProposal,
    setVoteResults,
    setUserVote,
    setSelectedProposalLoading,
    setSelectedProposalError,
    setVotingHistory,
    setVotingHistoryLoading,
    setVotingHistoryError,
    setTransactionPending,
    setTransactionHash,
    setTransactionError,
    setTransactionSuccess,
    clearTransaction,
  };

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
};

// eslint-disable-next-line react-refresh/only-export-components
export const useAppContext = () => {
  const context = useContext(AppContext);
  if (context === undefined) {
    throw new Error('useAppContext must be used within an AppProvider');
  }
  return context;
};
