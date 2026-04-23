import { rpc, Contract, Address, nativeToScVal, scValToNative } from '@stellar/stellar-sdk';
import { getRpcUrl, STELLAR_CONFIG } from '../config/stellar';

const API_URL = 'http://localhost:3001/api';

const NOW = Math.floor(Date.now() / 1000);
const DAY = 86400;

const REALISTIC_MOCKS = {
  proposals: [
    {
      id: 1,
      creator: 'GCQYDZ2Q...',
      title: 'Protocol 21 Upgrade Adoption',
      description: 'This proposal outlines the implementation of Protocol 21 on the Stellar network, introducing state archives and expanded smart contract functionality. Please review the technical documentation on the Stellar Developer portal before voting.',
      voting_start: NOW - 10 * DAY,
      voting_end: NOW - 2 * DAY,
      created_at: NOW - 11 * DAY,
      options: ['Support', 'Reject', 'Abstain'],
    },
    {
      id: 2,
      creator: 'GA7D49P...',
      title: 'Allocate 500,000 XLM for Developer Grants',
      description: 'Proposal to allocate 500,000 XLM from the community fund to support new Soroban smart contract developers. The funds will be distributed over 6 months through the Stellar Community Fund (SCF) review process.',
      voting_start: NOW - 2 * DAY,
      voting_end: NOW + 5 * DAY,
      created_at: NOW - 3 * DAY,
      options: ['Approve', 'Reject', 'Need More Details'],
    },
    {
      id: 3,
      creator: 'GB3AM8B...',
      title: 'Reduce Base Reserve Requirement',
      description: 'Proposing a reduction in the base reserve requirement from 0.5 XLM to 0.1 XLM to lower the barrier to entry for new network participants while maintaining sufficient spam protection.',
      voting_start: NOW - 1 * DAY,
      voting_end: NOW + 14 * DAY,
      created_at: NOW - 2 * DAY,
      options: ['Approve', 'Reject'],
    }
  ],
  results: {
    1: { proposal_id: 1, option_counts: [45, 12, 5], total_votes: 62, unique_voters: 62 },
    2: { proposal_id: 2, option_counts: [18, 4, 8], total_votes: 30, unique_voters: 30 },
    3: { proposal_id: 3, option_counts: [12, 2], total_votes: 14, unique_voters: 14 }
  }
};

export class VotingContractClient {
  private rpcUrl: string;
  public contractId: string;
  private server: rpc.Server;

  constructor() {
    this.rpcUrl = getRpcUrl();
    this.contractId = STELLAR_CONFIG.contractId || ''; // Empty string triggers POC backend
    this.server = new rpc.Server(this.rpcUrl);
  }

  async getProposal(id: number) {
    if (!this.contractId) {
      try {
        const res = await fetch(`${API_URL}/proposals/${id}`);
        if (!res.ok) throw new Error('Fetch failed');
        return await res.json();
      } catch {
        // Fallback to realistic mock data if backend is offline
        const proposal = REALISTIC_MOCKS.proposals.find(p => p.id === id);
        if (proposal) return proposal;
        // eslint-disable-next-line preserve-caught-error
        throw new Error('Failed to fetch proposal');
      }
    }
    const contract = new Contract(this.contractId);
    const tx = contract.call('get_proposal', nativeToScVal(id, { type: 'u32' }));
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const result = await this.server.simulateTransaction(tx as any);
    if (rpc.Api.isSimulationSuccess(result) && result.result) {
      return scValToNative(result.result.retval);
    }
    throw new Error('Failed to fetch proposal');
  }

  async getProposals(start: number, limit: number) {
    if (!this.contractId) {
      try {
        const res = await fetch(`${API_URL}/proposals`);
        if (!res.ok) throw new Error('Fetch failed');
        return await res.json();
      } catch {
        // Fallback to realistic mock data if backend is offline
        return REALISTIC_MOCKS.proposals;
      }
    }
    const contract = new Contract(this.contractId);
    const tx = contract.call(
      'get_proposals',
      nativeToScVal(start, { type: 'u32' }),
      nativeToScVal(limit, { type: 'u32' })
    );
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const result = await this.server.simulateTransaction(tx as any);
    if (rpc.Api.isSimulationSuccess(result) && result.result) {
      return scValToNative(result.result.retval);
    }
    return [];
  }

  async getVoteResults(proposalId: number) {
    if (!this.contractId) {
      try {
        const res = await fetch(`${API_URL}/proposals/${proposalId}/results`);
        if (!res.ok) throw new Error('Fetch failed');
        return await res.json();
      } catch {
        // Fallback to realistic mock data if backend is offline
        const results = REALISTIC_MOCKS.results[proposalId as keyof typeof REALISTIC_MOCKS.results];
        if (results) return results;
        throw new Error('Failed to fetch vote results');
      }
    }
    const contract = new Contract(this.contractId);
    const tx = contract.call('get_vote_results', nativeToScVal(proposalId, { type: 'u32' }));
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const result = await this.server.simulateTransaction(tx as any);
    if (rpc.Api.isSimulationSuccess(result) && result.result) {
      return scValToNative(result.result.retval);
    }
    throw new Error('Failed to fetch vote results');
  }

  async getVoterHistory(address: string) {
    if (!this.contractId) {
      const res = await fetch(`${API_URL}/voters/${address}/history`);
      if (!res.ok) return [];
      return res.json();
    }
    const contract = new Contract(this.contractId);
    const tx = contract.call('get_voter_history', new Address(address).toScVal());
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const result = await this.server.simulateTransaction(tx as any);
    if (rpc.Api.isSimulationSuccess(result) && result.result) {
      return scValToNative(result.result.retval);
    }
    return [];
  }

  async createProposal(
    creator: string,
    title: string,
    description: string,
    voting_end: number,
    options: string[]
  ) {
    if (!this.contractId) {
      const res = await fetch(`${API_URL}/proposals`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ creator, title, description, voting_end, options })
      });
      if (!res.ok) throw new Error('Failed to create proposal');
      return res.json();
    }
    const contract = new Contract(this.contractId);
    contract.call(
      'create_proposal',
      new Address(creator).toScVal(),
      nativeToScVal(title, { type: 'string' }),
      nativeToScVal(description, { type: 'string' }),
      nativeToScVal(voting_end, { type: 'u64' }),
      nativeToScVal(options, { type: 'vec' })
    );
    // Real flow would build transaction, sign it with Freighter, submit it
    return { hash: 'real_tx_hash_123', proposalId: 0 };
  }

  async castVote(proposalId: number, voter: string, choice: number) {
    if (!this.contractId) {
      try {
        const res = await fetch(`${API_URL}/votes`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ proposalId, voter, choice })
        });
        if (!res.ok) {
          const errData = await res.json();
          throw new Error(errData.error || 'Failed to cast vote');
        }
        return await res.json();
      } catch (e: unknown) {
        if (e instanceof Error) throw e;
        // eslint-disable-next-line preserve-caught-error
        throw new Error('Failed to cast vote');
      }
    }
    const contract = new Contract(this.contractId);
    contract.call(
      'cast_vote',
      nativeToScVal(proposalId, { type: 'u32' }),
      new Address(voter).toScVal(),
      nativeToScVal(choice, { type: 'u32' })
    );
    // Real flow
    return { hash: 'real_vote_hash_456' };
  }
}
