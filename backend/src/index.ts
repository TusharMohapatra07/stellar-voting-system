import express from 'express';
import cors from 'cors';

const app = express();
const PORT = process.env.PORT || 3001;

app.use(cors());
app.use(express.json());

// In-memory data store for POC
let nextProposalId = 1;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const proposals: any[] = [];
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const votes: any[] = [];

const NOW = Math.floor(Date.now() / 1000);
const DAY = 86400;

// Proposal 1: Network Upgrade (Closed)
proposals.push({
  id: nextProposalId++,
  creator: 'GCQYDZ2Q...',
  title: 'Protocol 21 Upgrade Adoption',
  description: 'This proposal outlines the implementation of Protocol 21 on the Stellar network, introducing state archives and expanded smart contract functionality. Please review the technical documentation on the Stellar Developer portal before voting.',
  voting_start: NOW - 10 * DAY,
  voting_end: NOW - 2 * DAY,
  created_at: NOW - 11 * DAY,
  options: ['Support', 'Reject', 'Abstain'],
});

// Proposal 2: Community Fund Allocation (Active)
proposals.push({
  id: nextProposalId++,
  creator: 'GA7D49P...',
  title: 'Allocate 500,000 XLM for Developer Grants',
  description: 'Proposal to allocate 500,000 XLM from the community fund to support new Soroban smart contract developers. The funds will be distributed over 6 months through the Stellar Community Fund (SCF) review process.',
  voting_start: NOW - 2 * DAY,
  voting_end: NOW + 5 * DAY,
  created_at: NOW - 3 * DAY,
  options: ['Approve', 'Reject', 'Need More Details'],
});

// Proposal 3: Protocol Adjustment (Active)
proposals.push({
  id: nextProposalId++,
  creator: 'GB3AM8B...',
  title: 'Reduce Base Reserve Requirement',
  description: 'Proposing a reduction in the base reserve requirement from 0.5 XLM to 0.1 XLM to lower the barrier to entry for new network participants while maintaining sufficient spam protection.',
  voting_start: NOW - 1 * DAY,
  voting_end: NOW + 14 * DAY,
  created_at: NOW - 2 * DAY,
  options: ['Approve', 'Reject'],
});

// Votes for Proposal 1
for (let i=0; i<45; i++) votes.push({ proposalId: 1, voter: `G_VOTER_${i}`, choice: 0 });
for (let i=0; i<12; i++) votes.push({ proposalId: 1, voter: `G_VOTER_${i+100}`, choice: 1 });
for (let i=0; i<5; i++) votes.push({ proposalId: 1, voter: `G_VOTER_${i+200}`, choice: 2 });

// Votes for Proposal 2
for (let i=0; i<18; i++) votes.push({ proposalId: 2, voter: `G_VOTER_${i}`, choice: 0 });
for (let i=0; i<4; i++) votes.push({ proposalId: 2, voter: `G_VOTER_${i+100}`, choice: 1 });
for (let i=0; i<8; i++) votes.push({ proposalId: 2, voter: `G_VOTER_${i+200}`, choice: 2 });

// Votes for Proposal 3
for (let i=0; i<12; i++) votes.push({ proposalId: 3, voter: `G_VOTER_${i}`, choice: 0 });
for (let i=0; i<2; i++) votes.push({ proposalId: 3, voter: `G_VOTER_${i+100}`, choice: 1 });

app.get('/api/health', (req, res) => {
  res.json({ status: 'healthy', message: 'Backend API POC running' });
});

app.get('/api/proposals', (req, res) => {
  res.json(proposals);
});

app.get('/api/proposals/:id', (req, res) => {
  const id = parseInt(req.params.id);
  const proposal = proposals.find(p => p.id === id);
  if (proposal) res.json(proposal);
  else res.status(404).json({ error: 'Not found' });
});

app.get('/api/proposals/:id/results', (req, res) => {
  const id = parseInt(req.params.id);
  const proposal = proposals.find(p => p.id === id);
  if (!proposal) return res.status(404).json({ error: 'Not found' });
  
  const proposalVotes = votes.filter(v => v.proposalId === id);
  const option_counts = new Array(proposal.options.length).fill(0);
  const unique_voters = new Set();
  
  proposalVotes.forEach(v => {
    option_counts[v.choice]++;
    unique_voters.add(v.voter);
  });

  res.json({
    proposal_id: id,
    option_counts,
    total_votes: proposalVotes.length,
    unique_voters: unique_voters.size
  });
});

app.get('/api/voters/:address/history', (req, res) => {
  const address = req.params.address;
  const userVotes = votes.filter(v => v.voter === address);
  res.json(userVotes);
});

// Mock write endpoints
app.post('/api/proposals', (req, res) => {
  const { creator, title, description, voting_end, options } = req.body;
  const newProposal = {
    id: nextProposalId++,
    creator,
    title,
    description,
    voting_start: Math.floor(Date.now() / 1000),
    voting_end,
    created_at: Math.floor(Date.now() / 1000),
    options,
  };
  proposals.push(newProposal);
  res.json({ hash: 'mock_tx_' + Date.now(), proposalId: newProposal.id });
});

app.post('/api/votes', (req, res) => {
  const { proposalId, voter, choice } = req.body;
  if (votes.some(v => v.proposalId === proposalId && v.voter === voter)) {
    return res.status(400).json({ error: 'Already voted' });
  }
  votes.push({ proposalId, voter, choice });
  res.json({ hash: 'mock_vote_' + Date.now() });
});

if (process.env.NODE_ENV !== 'test') {
  app.listen(PORT, () => {
    console.log(`Backend API POC running on port ${PORT}`);
  });
}

export default app;
