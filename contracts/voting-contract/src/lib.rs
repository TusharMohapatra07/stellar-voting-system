#![no_std]

use soroban_sdk::{contract, contractimpl, contracterror, contracttype, symbol_short, Address, Env, String, Symbol, Vec};

/// Proposal structure stored on-chain
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub creator: Address,
    pub title: String,
    pub description: String,
    pub voting_start: u64,  // Unix timestamp
    pub voting_end: u64,    // Unix timestamp
    pub created_at: u64,    // Unix timestamp
    pub options: Vec<String>,
}

/// Vote structure stored on-chain
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vote {
    pub proposal_id: u64,
    pub voter: Address,
    pub choice: u32,        // Index into proposal options
    pub timestamp: u64,     // Unix timestamp
}

/// Vote result aggregation
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteResult {
    pub proposal_id: u64,
    pub option_counts: Vec<u64>,
    pub total_votes: u64,
    pub unique_voters: u64,
}

/// Contract error codes
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    ProposalNotFound = 1,
    ProposalNotActive = 2,
    VotingPeriodEnded = 3,
    AlreadyVoted = 4,
    InvalidChoice = 5,
    InvalidVotingPeriod = 6,
    InvalidTitle = 7,
    InvalidDescription = 8,
    Unauthorized = 9,
    InvalidInput = 10,
}

#[contract]
pub struct VotingContract;

// Storage key constants
const PROPOSAL_COUNT: Symbol = symbol_short!("PROP_CNT");

#[contractimpl]
impl VotingContract {
    /// Initialize contract (Task 4.1)
    /// Sets up initial contract state and initializes proposal counter to 0
    pub fn initialize(env: Env) {
        // Initialize proposal counter to 0
        env.storage().persistent().set(&PROPOSAL_COUNT, &0u64);
    }

    /// Create a new proposal (Task 4.2)
    /// Validates all inputs and stores the proposal with auto-generated ID
    /// Returns the proposal ID
    pub fn create_proposal(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        voting_end: u64,
        options: Vec<String>,
    ) -> Result<u64, ContractError> {
        // Require authentication from creator
        creator.require_auth();

        // Validate title length (5-200 characters)
        let title_len = title.len();
        if title_len < 5 || title_len > 200 {
            return Err(ContractError::InvalidTitle);
        }

        // Validate description length (20-5000 characters)
        let description_len = description.len();
        if description_len < 20 || description_len > 5000 {
            return Err(ContractError::InvalidDescription);
        }

        // Get current block timestamp
        let current_time = env.ledger().timestamp();

        // Validate voting_end is between 1 hour and 90 days in future
        let one_hour = 3600u64;
        let ninety_days = 90 * 24 * 3600u64;
        let min_end = current_time + one_hour;
        let max_end = current_time + ninety_days;

        if voting_end < min_end || voting_end > max_end {
            return Err(ContractError::InvalidVotingPeriod);
        }

        // Validate options array (2-10 options, each 1-100 characters)
        let num_options = options.len();
        if num_options < 2 || num_options > 10 {
            return Err(ContractError::InvalidInput);
        }

        for i in 0..num_options {
            let option = options.get(i).unwrap();
            let option_len = option.len();
            if option_len < 1 || option_len > 100 {
                return Err(ContractError::InvalidInput);
            }
        }

        // Generate new proposal ID
        let proposal_id = Self::increment_proposal_count(&env);

        // Create proposal with current timestamp as both voting_start and created_at
        let proposal = Proposal {
            id: proposal_id,
            creator: creator.clone(),
            title: title.clone(),
            description,
            voting_start: current_time,
            voting_end,
            created_at: current_time,
            options: options.clone(),
        };

        // Store proposal
        Self::store_proposal(&env, &proposal);

        // Initialize vote count cache for this proposal
        Self::initialize_vote_counts(&env, proposal_id, num_options);

        // Emit ProposalCreated event (Task 4.3)
        env.events().publish(
            (symbol_short!("PROP_CRT"), proposal_id),
            (creator, title, voting_end),
        );

        Ok(proposal_id)
    }

    // ========== Proposal Storage Functions (Task 3.1) ==========

    /// Store a proposal in contract storage
    /// Uses key format: PROPOSAL_{id}
    fn store_proposal(env: &Env, proposal: &Proposal) {
        let key = Self::proposal_key(proposal.id);
        env.storage().persistent().set(&key, proposal);
    }

    /// Retrieve a proposal from contract storage
    /// Returns None if proposal doesn't exist
    fn get_proposal_internal(env: &Env, proposal_id: u64) -> Option<Proposal> {
        let key = Self::proposal_key(proposal_id);
        env.storage().persistent().get(&key)
    }

    /// Get the current proposal count (next ID to use)
    fn get_proposal_count(env: &Env) -> u64 {
        env.storage().persistent().get(&PROPOSAL_COUNT).unwrap_or(0)
    }

    /// Increment and return the next proposal ID
    fn increment_proposal_count(env: &Env) -> u64 {
        let current = Self::get_proposal_count(env);
        let next = current + 1;
        env.storage().persistent().set(&PROPOSAL_COUNT, &next);
        next
    }

    /// Generate storage key for a proposal
    fn proposal_key(proposal_id: u64) -> (Symbol, u64) {
        (symbol_short!("PROPOSAL"), proposal_id)
    }

    // ========== Vote Storage Functions (Task 3.2) ==========

    /// Store a vote in contract storage
    /// Uses key format: VOTE_{proposal_id}_{voter_address}
    fn store_vote(env: &Env, vote: &Vote) {
        let key = Self::vote_key(vote.proposal_id, &vote.voter);
        env.storage().persistent().set(&key, vote);
        
        // Update secondary indices
        Self::add_vote_to_proposal_index(env, vote.proposal_id, &vote.voter);
        Self::add_vote_to_voter_index(env, &vote.voter, vote.proposal_id);
    }

    /// Retrieve a vote from contract storage
    /// Returns None if vote doesn't exist
    fn get_vote_internal(env: &Env, proposal_id: u64, voter: &Address) -> Option<Vote> {
        let key = Self::vote_key(proposal_id, voter);
        env.storage().persistent().get(&key)
    }

    /// Check if a voter has already voted on a proposal
    fn has_voted_internal(env: &Env, proposal_id: u64, voter: &Address) -> bool {
        let key = Self::vote_key(proposal_id, voter);
        env.storage().persistent().has(&key)
    }

    /// Generate storage key for a vote
    fn vote_key(proposal_id: u64, voter: &Address) -> (Symbol, u64, Address) {
        (symbol_short!("VOTE"), proposal_id, voter.clone())
    }

    /// Add vote to proposal index (for querying all votes by proposal)
    fn add_vote_to_proposal_index(env: &Env, proposal_id: u64, voter: &Address) {
        let key = Self::votes_by_proposal_key(proposal_id);
        let mut voters: Vec<Address> = env.storage().persistent().get(&key).unwrap_or(Vec::new(env));
        voters.push_back(voter.clone());
        env.storage().persistent().set(&key, &voters);
    }

    /// Get all voters for a proposal
    fn get_proposal_voters(env: &Env, proposal_id: u64) -> Vec<Address> {
        let key = Self::votes_by_proposal_key(proposal_id);
        env.storage().persistent().get(&key).unwrap_or(Vec::new(env))
    }

    /// Generate storage key for votes by proposal index
    fn votes_by_proposal_key(proposal_id: u64) -> (Symbol, u64) {
        (symbol_short!("VOTESPRO"), proposal_id)
    }

    /// Add vote to voter index (for querying all votes by voter)
    fn add_vote_to_voter_index(env: &Env, voter: &Address, proposal_id: u64) {
        let key = Self::votes_by_voter_key(voter);
        let mut proposal_ids: Vec<u64> = env.storage().persistent().get(&key).unwrap_or(Vec::new(env));
        proposal_ids.push_back(proposal_id);
        env.storage().persistent().set(&key, &proposal_ids);
    }

    /// Get all proposal IDs a voter has voted on
    fn get_voter_proposal_ids(env: &Env, voter: &Address) -> Vec<u64> {
        let key = Self::votes_by_voter_key(voter);
        env.storage().persistent().get(&key).unwrap_or(Vec::new(env))
    }

    /// Generate storage key for votes by voter index
    fn votes_by_voter_key(voter: &Address) -> (Symbol, Address) {
        (symbol_short!("VOTESVOT"), voter.clone())
    }

    // ========== Vote Count Cache Storage (Task 3.3) ==========

    /// Initialize vote count cache for a proposal
    /// Creates counters for each option initialized to 0
    fn initialize_vote_counts(env: &Env, proposal_id: u64, num_options: u32) {
        for option_index in 0..num_options {
            let key = Self::vote_count_key(proposal_id, option_index);
            env.storage().persistent().set(&key, &0u64);
        }
    }

    /// Increment vote count for a specific option (atomic operation)
    fn increment_vote_count(env: &Env, proposal_id: u64, option_index: u32) {
        let key = Self::vote_count_key(proposal_id, option_index);
        let current: u64 = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage().persistent().set(&key, &(current + 1));
    }

    /// Get vote count for a specific option
    fn get_vote_count(env: &Env, proposal_id: u64, option_index: u32) -> u64 {
        let key = Self::vote_count_key(proposal_id, option_index);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    /// Get all vote counts for a proposal
    fn get_all_vote_counts(env: &Env, proposal_id: u64, num_options: u32) -> Vec<u64> {
        let mut counts = Vec::new(env);
        for option_index in 0..num_options {
            let count = Self::get_vote_count(env, proposal_id, option_index);
            counts.push_back(count);
        }
        counts
    }

    /// Generate storage key for vote count cache
    fn vote_count_key(proposal_id: u64, option_index: u32) -> (Symbol, u64, u32) {
        (symbol_short!("VOTECNT"), proposal_id, option_index)
    }

    // ========== Voting Logic Functions (Task 5) ==========

    /// Check if a proposal is currently active (Task 5.1)
    /// Compares current block timestamp to voting_end
    /// Returns true if current time is before voting_end, false otherwise
    pub fn is_proposal_active(env: Env, proposal_id: u64) -> Result<bool, ContractError> {
        // Get the proposal
        let proposal = Self::get_proposal_internal(&env, proposal_id)
            .ok_or(ContractError::ProposalNotFound)?;
        
        // Get current block timestamp
        let current_time = env.ledger().timestamp();
        
        // Return true if current time is before voting_end
        Ok(current_time < proposal.voting_end)
    }

    /// Check if a voter has already voted on a proposal (Task 5.2)
    /// Checks if VOTE_{proposal_id}_{voter_address} key exists in storage
    /// Returns boolean indicating if voter has already voted
    pub fn has_voted(env: Env, voter: Address, proposal_id: u64) -> Result<bool, ContractError> {
        // Verify proposal exists
        Self::get_proposal_internal(&env, proposal_id)
            .ok_or(ContractError::ProposalNotFound)?;
        
        // Check if vote exists
        Ok(Self::has_voted_internal(&env, proposal_id, &voter))
    }

    /// Cast a vote on a proposal (Task 5.3)
    /// Performs all validation checks and records the vote
    pub fn cast_vote(
        env: Env,
        voter: Address,
        proposal_id: u64,
        choice: u32,
    ) -> Result<(), ContractError> {
        // Verify transaction is signed by voter address
        voter.require_auth();

        // Verify proposal exists
        let proposal = Self::get_proposal_internal(&env, proposal_id)
            .ok_or(ContractError::ProposalNotFound)?;

        // Verify proposal is active
        let current_time = env.ledger().timestamp();
        if current_time >= proposal.voting_end {
            return Err(ContractError::VotingPeriodEnded);
        }

        // Verify voter hasn't already voted
        if Self::has_voted_internal(&env, proposal_id, &voter) {
            return Err(ContractError::AlreadyVoted);
        }

        // Verify choice is valid index into proposal options
        if choice >= proposal.options.len() {
            return Err(ContractError::InvalidChoice);
        }

        // Create vote record with current block timestamp
        let vote = Vote {
            proposal_id,
            voter: voter.clone(),
            choice,
            timestamp: current_time,
        };

        // Store vote record
        Self::store_vote(&env, &vote);

        // Increment vote count cache for chosen option
        Self::increment_vote_count(&env, proposal_id, choice);

        // Emit VoteCast event (Task 5.4)
        env.events().publish(
            (symbol_short!("VOTE_CST"), proposal_id),
            (voter, choice, current_time),
        );

        Ok(())
    }

    // ========== Query Functions (Task 6) ==========

    /// Get a proposal by ID (Task 6.1)
    /// Retrieves proposal from storage and returns ProposalNotFound error if doesn't exist
    pub fn get_proposal(env: Env, proposal_id: u64) -> Result<Proposal, ContractError> {
        Self::get_proposal_internal(&env, proposal_id)
            .ok_or(ContractError::ProposalNotFound)
    }

    /// Get proposals with pagination (Task 6.2)
    /// Accepts start and limit parameters, retrieves proposals in batches
    /// Returns array of proposals
    pub fn get_proposals(env: Env, start: u64, limit: u64) -> Vec<Proposal> {
        let mut proposals = Vec::new(&env);
        let proposal_count = Self::get_proposal_count(&env);
        
        // Calculate the range of proposals to retrieve
        let end = if start + limit > proposal_count {
            proposal_count
        } else {
            start + limit
        };
        
        // Retrieve proposals in the specified range
        for id in start..end {
            // Proposal IDs start at 1, not 0
            let proposal_id = id + 1;
            if let Some(proposal) = Self::get_proposal_internal(&env, proposal_id) {
                proposals.push_back(proposal);
            }
        }
        
        proposals
    }

    /// Get all votes for a proposal (Task 6.3)
    /// Queries all votes for a proposal_id using secondary index
    /// Returns array of Vote structs
    pub fn get_proposal_votes(env: Env, proposal_id: u64) -> Vec<Vote> {
        let mut votes = Vec::new(&env);
        
        // Get all voters for this proposal from the secondary index
        let voters = Self::get_proposal_voters(&env, proposal_id);
        
        // Retrieve each vote
        for i in 0..voters.len() {
            let voter = voters.get(i).unwrap();
            if let Some(vote) = Self::get_vote_internal(&env, proposal_id, &voter) {
                votes.push_back(vote);
            }
        }
        
        votes
    }

    /// Get voting history for a voter (Task 6.4)
    /// Queries all votes by voter address using secondary index
    /// Returns array of Vote structs
    pub fn get_voter_history(env: Env, voter: Address) -> Vec<Vote> {
        let mut votes = Vec::new(&env);
        
        // Get all proposal IDs the voter has voted on from the secondary index
        let proposal_ids = Self::get_voter_proposal_ids(&env, &voter);
        
        // Retrieve each vote
        for i in 0..proposal_ids.len() {
            let proposal_id = proposal_ids.get(i).unwrap();
            if let Some(vote) = Self::get_vote_internal(&env, proposal_id, &voter) {
                votes.push_back(vote);
            }
        }
        
        votes
    }

    /// Get vote results for a proposal (Task 6.5)
    /// Retrieves vote counts from cache, calculates total votes and unique voters
    /// Returns VoteResult struct
    pub fn get_vote_results(env: Env, proposal_id: u64) -> Result<VoteResult, ContractError> {
        // Verify proposal exists
        let proposal = Self::get_proposal_internal(&env, proposal_id)
            .ok_or(ContractError::ProposalNotFound)?;
        
        // Get vote counts for all options
        let num_options = proposal.options.len();
        let option_counts = Self::get_all_vote_counts(&env, proposal_id, num_options);
        
        // Calculate total votes
        let mut total_votes = 0u64;
        for i in 0..option_counts.len() {
            total_votes += option_counts.get(i).unwrap();
        }
        
        // Get unique voters count from the secondary index
        let voters = Self::get_proposal_voters(&env, proposal_id);
        let unique_voters = voters.len() as u64;
        
        Ok(VoteResult {
            proposal_id,
            option_counts,
            total_votes,
            unique_voters,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, testutils::Events, testutils::Ledger, Env};

    #[test]
    fn test_contract_initialization() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        // Test that initialize can be called without panicking
        client.initialize();
    }

    #[test]
    fn test_proposal_serialization() {
        let env = Env::default();
        
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Test Proposal");
        let description = String::from_str(&env, "This is a test proposal description");
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Option 1"),
            String::from_str(&env, "Option 2"),
        ]);
        
        let proposal = Proposal {
            id: 1,
            creator: creator.clone(),
            title: title.clone(),
            description: description.clone(),
            voting_start: 1000,
            voting_end: 2000,
            created_at: 1000,
            options: options.clone(),
        };
        
        // Clone to simulate serialization/deserialization
        let cloned = proposal.clone();
        
        // Verify round-trip consistency
        assert_eq!(proposal.id, cloned.id);
        assert_eq!(proposal.creator, cloned.creator);
        assert_eq!(proposal.title, cloned.title);
        assert_eq!(proposal.description, cloned.description);
        assert_eq!(proposal.voting_start, cloned.voting_start);
        assert_eq!(proposal.voting_end, cloned.voting_end);
        assert_eq!(proposal.created_at, cloned.created_at);
        assert_eq!(proposal.options, cloned.options);
    }

    #[test]
    fn test_vote_serialization() {
        let env = Env::default();
        
        let voter = Address::generate(&env);
        
        let vote = Vote {
            proposal_id: 1,
            voter: voter.clone(),
            choice: 0,
            timestamp: 1500,
        };
        
        // Clone to simulate serialization/deserialization
        let cloned = vote.clone();
        
        // Verify round-trip consistency
        assert_eq!(vote.proposal_id, cloned.proposal_id);
        assert_eq!(vote.voter, cloned.voter);
        assert_eq!(vote.choice, cloned.choice);
        assert_eq!(vote.timestamp, cloned.timestamp);
    }

    #[test]
    fn test_vote_result_serialization() {
        let env = Env::default();
        
        let option_counts = Vec::from_array(&env, [10u64, 20u64, 30u64]);
        
        let vote_result = VoteResult {
            proposal_id: 1,
            option_counts: option_counts.clone(),
            total_votes: 60,
            unique_voters: 60,
        };
        
        // Clone to simulate serialization/deserialization
        let cloned = vote_result.clone();
        
        // Verify round-trip consistency
        assert_eq!(vote_result.proposal_id, cloned.proposal_id);
        assert_eq!(vote_result.option_counts, cloned.option_counts);
        assert_eq!(vote_result.total_votes, cloned.total_votes);
        assert_eq!(vote_result.unique_voters, cloned.unique_voters);
    }

    // ========== Storage Layer Tests (Task 3.4) ==========

    #[test]
    fn test_proposal_storage_and_retrieval() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Test Proposal");
        let description = String::from_str(&env, "This is a test proposal description");
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Yes"),
            String::from_str(&env, "No"),
        ]);
        
        let proposal = Proposal {
            id: 1,
            creator: creator.clone(),
            title: title.clone(),
            description: description.clone(),
            voting_start: 1000,
            voting_end: 2000,
            created_at: 1000,
            options: options.clone(),
        };
        
        // Store and retrieve proposal within contract context
        env.as_contract(&contract_id, || {
            VotingContract::store_proposal(&env, &proposal);
            
            let retrieved = VotingContract::get_proposal_internal(&env, 1);
            assert!(retrieved.is_some());
            
            let retrieved_proposal = retrieved.unwrap();
            assert_eq!(retrieved_proposal.id, proposal.id);
            assert_eq!(retrieved_proposal.creator, proposal.creator);
            assert_eq!(retrieved_proposal.title, proposal.title);
            assert_eq!(retrieved_proposal.description, proposal.description);
        });
    }

    #[test]
    fn test_proposal_count_increment() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        
        env.as_contract(&contract_id, || {
            // Initial count should be 0
            assert_eq!(VotingContract::get_proposal_count(&env), 0);
            
            // Increment and check
            let id1 = VotingContract::increment_proposal_count(&env);
            assert_eq!(id1, 1);
            assert_eq!(VotingContract::get_proposal_count(&env), 1);
            
            // Increment again
            let id2 = VotingContract::increment_proposal_count(&env);
            assert_eq!(id2, 2);
            assert_eq!(VotingContract::get_proposal_count(&env), 2);
        });
    }

    #[test]
    fn test_vote_storage_and_retrieval() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        env.mock_all_auths();
        
        let voter = Address::generate(&env);
        
        let vote = Vote {
            proposal_id: 1,
            voter: voter.clone(),
            choice: 0,
            timestamp: 1500,
        };
        
        env.as_contract(&contract_id, || {
            // Store vote
            VotingContract::store_vote(&env, &vote);
            
            // Retrieve vote
            let retrieved = VotingContract::get_vote_internal(&env, 1, &voter);
            assert!(retrieved.is_some());
            
            let retrieved_vote = retrieved.unwrap();
            assert_eq!(retrieved_vote.proposal_id, vote.proposal_id);
            assert_eq!(retrieved_vote.voter, vote.voter);
            assert_eq!(retrieved_vote.choice, vote.choice);
            assert_eq!(retrieved_vote.timestamp, vote.timestamp);
        });
    }

    #[test]
    fn test_has_voted_check() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        env.mock_all_auths();
        
        let voter = Address::generate(&env);
        
        env.as_contract(&contract_id, || {
            // Initially should not have voted
            assert!(!VotingContract::has_voted_internal(&env, 1, &voter));
            
            // Store a vote
            let vote = Vote {
                proposal_id: 1,
                voter: voter.clone(),
                choice: 0,
                timestamp: 1500,
            };
            VotingContract::store_vote(&env, &vote);
            
            // Now should have voted
            assert!(VotingContract::has_voted_internal(&env, 1, &voter));
            
            // Different proposal should still be false
            assert!(!VotingContract::has_voted_internal(&env, 2, &voter));
        });
    }

    #[test]
    fn test_votes_by_proposal_index() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        env.mock_all_auths();
        
        let voter1 = Address::generate(&env);
        let voter2 = Address::generate(&env);
        let voter3 = Address::generate(&env);
        
        env.as_contract(&contract_id, || {
            // Store votes for proposal 1
            let vote1 = Vote {
                proposal_id: 1,
                voter: voter1.clone(),
                choice: 0,
                timestamp: 1500,
            };
            let vote2 = Vote {
                proposal_id: 1,
                voter: voter2.clone(),
                choice: 1,
                timestamp: 1600,
            };
            let vote3 = Vote {
                proposal_id: 1,
                voter: voter3.clone(),
                choice: 0,
                timestamp: 1700,
            };
            
            VotingContract::store_vote(&env, &vote1);
            VotingContract::store_vote(&env, &vote2);
            VotingContract::store_vote(&env, &vote3);
            
            // Get all voters for proposal 1
            let voters = VotingContract::get_proposal_voters(&env, 1);
            assert_eq!(voters.len(), 3);
            assert_eq!(voters.get(0).unwrap(), voter1);
            assert_eq!(voters.get(1).unwrap(), voter2);
            assert_eq!(voters.get(2).unwrap(), voter3);
        });
    }

    #[test]
    fn test_votes_by_voter_index() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        env.mock_all_auths();
        
        let voter = Address::generate(&env);
        
        env.as_contract(&contract_id, || {
            // Store votes for different proposals
            let vote1 = Vote {
                proposal_id: 1,
                voter: voter.clone(),
                choice: 0,
                timestamp: 1500,
            };
            let vote2 = Vote {
                proposal_id: 2,
                voter: voter.clone(),
                choice: 1,
                timestamp: 1600,
            };
            let vote3 = Vote {
                proposal_id: 3,
                voter: voter.clone(),
                choice: 0,
                timestamp: 1700,
            };
            
            VotingContract::store_vote(&env, &vote1);
            VotingContract::store_vote(&env, &vote2);
            VotingContract::store_vote(&env, &vote3);
            
            // Get all proposal IDs voter has voted on
            let proposal_ids = VotingContract::get_voter_proposal_ids(&env, &voter);
            assert_eq!(proposal_ids.len(), 3);
            assert_eq!(proposal_ids.get(0).unwrap(), 1);
            assert_eq!(proposal_ids.get(1).unwrap(), 2);
            assert_eq!(proposal_ids.get(2).unwrap(), 3);
        });
    }

    #[test]
    fn test_vote_count_cache_initialization() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        
        env.as_contract(&contract_id, || {
            // Initialize vote counts for a proposal with 3 options
            VotingContract::initialize_vote_counts(&env, 1, 3);
            
            // All counts should be 0
            assert_eq!(VotingContract::get_vote_count(&env, 1, 0), 0);
            assert_eq!(VotingContract::get_vote_count(&env, 1, 1), 0);
            assert_eq!(VotingContract::get_vote_count(&env, 1, 2), 0);
        });
    }

    #[test]
    fn test_vote_count_increment() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        
        env.as_contract(&contract_id, || {
            // Initialize vote counts
            VotingContract::initialize_vote_counts(&env, 1, 2);
            
            // Increment option 0 multiple times
            VotingContract::increment_vote_count(&env, 1, 0);
            VotingContract::increment_vote_count(&env, 1, 0);
            VotingContract::increment_vote_count(&env, 1, 0);
            
            // Increment option 1 once
            VotingContract::increment_vote_count(&env, 1, 1);
            
            // Check counts
            assert_eq!(VotingContract::get_vote_count(&env, 1, 0), 3);
            assert_eq!(VotingContract::get_vote_count(&env, 1, 1), 1);
        });
    }

    #[test]
    fn test_get_all_vote_counts() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        
        env.as_contract(&contract_id, || {
            // Initialize and set some counts
            VotingContract::initialize_vote_counts(&env, 1, 3);
            VotingContract::increment_vote_count(&env, 1, 0);
            VotingContract::increment_vote_count(&env, 1, 0);
            VotingContract::increment_vote_count(&env, 1, 1);
            VotingContract::increment_vote_count(&env, 1, 2);
            VotingContract::increment_vote_count(&env, 1, 2);
            VotingContract::increment_vote_count(&env, 1, 2);
            
            // Get all counts
            let counts = VotingContract::get_all_vote_counts(&env, 1, 3);
            assert_eq!(counts.len(), 3);
            assert_eq!(counts.get(0).unwrap(), 2);
            assert_eq!(counts.get(1).unwrap(), 1);
            assert_eq!(counts.get(2).unwrap(), 3);
        });
    }

    // ========== Proposal Creation Tests (Task 4.4) ==========

    #[test]
    fn test_successful_proposal_creation() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Valid Proposal Title");
        let description = String::from_str(&env, "This is a valid proposal description with sufficient length");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200; // 2 hours in future
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Option A"),
            String::from_str(&env, "Option B"),
        ]);
        
        // Create proposal
        let proposal_id = client.create_proposal(&creator, &title, &description, &voting_end, &options);
        
        // Verify proposal ID is 1 (first proposal)
        assert_eq!(proposal_id, 1);
        
        // Verify proposal was stored correctly
        env.as_contract(&contract_id, || {
            let stored_proposal = VotingContract::get_proposal_internal(&env, proposal_id);
            assert!(stored_proposal.is_some());
            
            let proposal = stored_proposal.unwrap();
            assert_eq!(proposal.id, proposal_id);
            assert_eq!(proposal.creator, creator);
            assert_eq!(proposal.title, title);
            assert_eq!(proposal.description, description);
            assert_eq!(proposal.voting_end, voting_end);
            assert_eq!(proposal.options.len(), 2);
        });
    }

    #[test]
    fn test_reject_short_title() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hi"); // Too short (< 5 chars)
        let description = String::from_str(&env, "This is a valid proposal description with sufficient length");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Option A"),
            String::from_str(&env, "Option B"),
        ]);
        
        let result = client.try_create_proposal(&creator, &title, &description, &voting_end, &options);
        assert_eq!(result, Err(Ok(ContractError::InvalidTitle)));
    }

    #[test]
    fn test_reject_long_title() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        // Title with 201 characters (> 200 chars)
        let long_title = "a".repeat(201);
        let title = String::from_str(&env, &long_title);
        let description = String::from_str(&env, "This is a valid proposal description with sufficient length");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Option A"),
            String::from_str(&env, "Option B"),
        ]);
        
        let result = client.try_create_proposal(&creator, &title, &description, &voting_end, &options);
        assert_eq!(result, Err(Ok(ContractError::InvalidTitle)));
    }

    #[test]
    fn test_reject_short_description() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Valid Title");
        let description = String::from_str(&env, "Too short"); // < 20 chars
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Option A"),
            String::from_str(&env, "Option B"),
        ]);
        
        let result = client.try_create_proposal(&creator, &title, &description, &voting_end, &options);
        assert_eq!(result, Err(Ok(ContractError::InvalidDescription)));
    }

    #[test]
    fn test_reject_voting_end_too_soon() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Valid Title");
        let description = String::from_str(&env, "This is a valid proposal description with sufficient length");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 1800; // 30 minutes (< 1 hour)
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Option A"),
            String::from_str(&env, "Option B"),
        ]);
        
        let result = client.try_create_proposal(&creator, &title, &description, &voting_end, &options);
        assert_eq!(result, Err(Ok(ContractError::InvalidVotingPeriod)));
    }

    #[test]
    fn test_reject_voting_end_too_far() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Valid Title");
        let description = String::from_str(&env, "This is a valid proposal description with sufficient length");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + (91 * 24 * 3600); // 91 days (> 90 days)
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Option A"),
            String::from_str(&env, "Option B"),
        ]);
        
        let result = client.try_create_proposal(&creator, &title, &description, &voting_end, &options);
        assert_eq!(result, Err(Ok(ContractError::InvalidVotingPeriod)));
    }

    #[test]
    fn test_reject_too_few_options() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Valid Title");
        let description = String::from_str(&env, "This is a valid proposal description with sufficient length");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Only One Option"), // < 2 options
        ]);
        
        let result = client.try_create_proposal(&creator, &title, &description, &voting_end, &options);
        assert_eq!(result, Err(Ok(ContractError::InvalidInput)));
    }

    #[test]
    fn test_reject_too_many_options() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Valid Title");
        let description = String::from_str(&env, "This is a valid proposal description with sufficient length");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        // 11 options (> 10)
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Option 1"),
            String::from_str(&env, "Option 2"),
            String::from_str(&env, "Option 3"),
            String::from_str(&env, "Option 4"),
            String::from_str(&env, "Option 5"),
            String::from_str(&env, "Option 6"),
            String::from_str(&env, "Option 7"),
            String::from_str(&env, "Option 8"),
            String::from_str(&env, "Option 9"),
            String::from_str(&env, "Option 10"),
            String::from_str(&env, "Option 11"),
        ]);
        
        let result = client.try_create_proposal(&creator, &title, &description, &voting_end, &options);
        assert_eq!(result, Err(Ok(ContractError::InvalidInput)));
    }

    #[test]
    fn test_reject_option_too_long() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Valid Title");
        let description = String::from_str(&env, "This is a valid proposal description with sufficient length");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        // Option with 101 characters (> 100)
        let long_option = "a".repeat(101);
        let options = Vec::from_array(&env, [
            String::from_str(&env, &long_option),
            String::from_str(&env, "Option B"),
        ]);
        
        let result = client.try_create_proposal(&creator, &title, &description, &voting_end, &options);
        assert_eq!(result, Err(Ok(ContractError::InvalidInput)));
    }

    #[test]
    fn test_proposal_event_emission() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Event Test Proposal");
        let description = String::from_str(&env, "This proposal tests event emission functionality");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Yes"),
            String::from_str(&env, "No"),
        ]);
        
        // Create proposal
        let proposal_id = client.create_proposal(&creator, &title, &description, &voting_end, &options);
        
        // Verify event was emitted
        let events = env.events().all();
        let event = events.last().unwrap();
        
        // Event is a tuple (contract_address, topics, data)
        // Verify we have topics (the event was published)
        assert!(event.1.len() > 0);
        
        // Verify proposal_id is 1
        assert_eq!(proposal_id, 1);
    }

    // ========== Voting Logic Tests (Task 5.5) ==========

    #[test]
    fn test_is_proposal_active() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Active Proposal Test");
        let description = String::from_str(&env, "Testing proposal active status check");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200; // 2 hours in future
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Yes"),
            String::from_str(&env, "No"),
        ]);
        
        // Create proposal
        let proposal_id = client.create_proposal(&creator, &title, &description, &voting_end, &options);
        
        // Proposal should be active
        let is_active = client.is_proposal_active(&proposal_id);
        assert!(is_active);
    }

    #[test]
    fn test_is_proposal_not_active() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Expired Proposal Test");
        let description = String::from_str(&env, "Testing expired proposal status check");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 3600; // 1 hour in future
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Yes"),
            String::from_str(&env, "No"),
        ]);
        
        // Create proposal
        let proposal_id = client.create_proposal(&creator, &title, &description, &voting_end, &options);
        
        // Fast forward time past voting_end
        env.ledger().with_mut(|li| {
            li.timestamp = voting_end + 1;
        });
        
        // Proposal should not be active
        let is_active = client.is_proposal_active(&proposal_id);
        assert!(!is_active);
    }

    #[test]
    fn test_has_voted_false() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let voter = Address::generate(&env);
        let title = String::from_str(&env, "Has Voted Test");
        let description = String::from_str(&env, "Testing has_voted check");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Yes"),
            String::from_str(&env, "No"),
        ]);
        
        // Create proposal
        let proposal_id = client.create_proposal(&creator, &title, &description, &voting_end, &options);
        
        // Voter should not have voted yet
        let has_voted = client.has_voted(&voter, &proposal_id);
        assert!(!has_voted);
    }

    #[test]
    fn test_has_voted_true() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let voter = Address::generate(&env);
        let title = String::from_str(&env, "Has Voted True Test");
        let description = String::from_str(&env, "Testing has_voted after voting");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Yes"),
            String::from_str(&env, "No"),
        ]);
        
        // Create proposal
        let proposal_id = client.create_proposal(&creator, &title, &description, &voting_end, &options);
        
        // Cast vote
        client.cast_vote(&voter, &proposal_id, &0);
        
        // Voter should have voted
        let has_voted = client.has_voted(&voter, &proposal_id);
        assert!(has_voted);
    }

    #[test]
    fn test_successful_vote_casting() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let voter = Address::generate(&env);
        let title = String::from_str(&env, "Vote Casting Test");
        let description = String::from_str(&env, "Testing successful vote casting");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Option A"),
            String::from_str(&env, "Option B"),
            String::from_str(&env, "Option C"),
        ]);
        
        // Create proposal
        let proposal_id = client.create_proposal(&creator, &title, &description, &voting_end, &options);
        
        // Cast vote
        client.cast_vote(&voter, &proposal_id, &1);
        
        // Verify vote was recorded
        env.as_contract(&contract_id, || {
            let vote = VotingContract::get_vote_internal(&env, proposal_id, &voter);
            assert!(vote.is_some());
            
            let vote_record = vote.unwrap();
            assert_eq!(vote_record.proposal_id, proposal_id);
            assert_eq!(vote_record.voter, voter);
            assert_eq!(vote_record.choice, 1);
            
            // Verify vote count was incremented
            let count = VotingContract::get_vote_count(&env, proposal_id, 1);
            assert_eq!(count, 1);
        });
    }

    #[test]
    fn test_reject_vote_on_closed_proposal() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let voter = Address::generate(&env);
        let title = String::from_str(&env, "Closed Proposal Test");
        let description = String::from_str(&env, "Testing vote rejection on closed proposal");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 3600; // 1 hour in future
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Yes"),
            String::from_str(&env, "No"),
        ]);
        
        // Create proposal
        let proposal_id = client.create_proposal(&creator, &title, &description, &voting_end, &options);
        
        // Fast forward time past voting_end
        env.ledger().with_mut(|li| {
            li.timestamp = voting_end + 1;
        });
        
        // Attempt to vote should fail
        let result = client.try_cast_vote(&voter, &proposal_id, &0);
        assert_eq!(result, Err(Ok(ContractError::VotingPeriodEnded)));
    }

    #[test]
    fn test_reject_duplicate_vote() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let voter = Address::generate(&env);
        let title = String::from_str(&env, "Duplicate Vote Test");
        let description = String::from_str(&env, "Testing duplicate vote rejection");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Yes"),
            String::from_str(&env, "No"),
        ]);
        
        // Create proposal
        let proposal_id = client.create_proposal(&creator, &title, &description, &voting_end, &options);
        
        // Cast first vote
        client.cast_vote(&voter, &proposal_id, &0);
        
        // Attempt to vote again should fail
        let result = client.try_cast_vote(&voter, &proposal_id, &1);
        assert_eq!(result, Err(Ok(ContractError::AlreadyVoted)));
    }

    #[test]
    fn test_reject_invalid_choice() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let voter = Address::generate(&env);
        let title = String::from_str(&env, "Invalid Choice Test");
        let description = String::from_str(&env, "Testing invalid choice rejection");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Yes"),
            String::from_str(&env, "No"),
        ]);
        
        // Create proposal with 2 options (indices 0 and 1)
        let proposal_id = client.create_proposal(&creator, &title, &description, &voting_end, &options);
        
        // Attempt to vote with invalid choice (index 2)
        let result = client.try_cast_vote(&voter, &proposal_id, &2);
        assert_eq!(result, Err(Ok(ContractError::InvalidChoice)));
    }

    #[test]
    fn test_reject_vote_on_nonexistent_proposal() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let voter = Address::generate(&env);
        
        // Attempt to vote on non-existent proposal
        let result = client.try_cast_vote(&voter, &999, &0);
        assert_eq!(result, Err(Ok(ContractError::ProposalNotFound)));
    }

    #[test]
    fn test_vote_cast_event_emission() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let voter = Address::generate(&env);
        let title = String::from_str(&env, "Vote Event Test");
        let description = String::from_str(&env, "Testing vote event emission");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Yes"),
            String::from_str(&env, "No"),
        ]);
        
        // Create proposal
        let proposal_id = client.create_proposal(&creator, &title, &description, &voting_end, &options);
        
        // Cast vote
        client.cast_vote(&voter, &proposal_id, &0);
        
        // Verify event was emitted
        let events = env.events().all();
        let event = events.last().unwrap();
        
        // Verify we have topics (the event was published)
        assert!(event.1.len() > 0);
    }

    #[test]
    fn test_multiple_voters_vote_count() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let voter1 = Address::generate(&env);
        let voter2 = Address::generate(&env);
        let voter3 = Address::generate(&env);
        let title = String::from_str(&env, "Multiple Voters Test");
        let description = String::from_str(&env, "Testing vote count with multiple voters");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Option A"),
            String::from_str(&env, "Option B"),
        ]);
        
        // Create proposal
        let proposal_id = client.create_proposal(&creator, &title, &description, &voting_end, &options);
        
        // Cast votes
        client.cast_vote(&voter1, &proposal_id, &0); // Vote for Option A
        client.cast_vote(&voter2, &proposal_id, &0); // Vote for Option A
        client.cast_vote(&voter3, &proposal_id, &1); // Vote for Option B
        
        // Verify vote counts
        env.as_contract(&contract_id, || {
            let count_a = VotingContract::get_vote_count(&env, proposal_id, 0);
            let count_b = VotingContract::get_vote_count(&env, proposal_id, 1);
            
            assert_eq!(count_a, 2);
            assert_eq!(count_b, 1);
        });
    }

    // ========== Query Functions Tests (Task 6.6) ==========

    #[test]
    fn test_get_proposal() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Query Test Proposal");
        let description = String::from_str(&env, "Testing get_proposal query function");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Yes"),
            String::from_str(&env, "No"),
        ]);
        
        // Create proposal
        let proposal_id = client.create_proposal(&creator, &title, &description, &voting_end, &options);
        
        // Query proposal
        let retrieved_proposal = client.get_proposal(&proposal_id);
        
        // Verify proposal data
        assert_eq!(retrieved_proposal.id, proposal_id);
        assert_eq!(retrieved_proposal.creator, creator);
        assert_eq!(retrieved_proposal.title, title);
        assert_eq!(retrieved_proposal.description, description);
        assert_eq!(retrieved_proposal.voting_end, voting_end);
        assert_eq!(retrieved_proposal.options.len(), 2);
    }

    #[test]
    fn test_get_proposal_not_found() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        // Query non-existent proposal
        let result = client.try_get_proposal(&999);
        assert_eq!(result, Err(Ok(ContractError::ProposalNotFound)));
    }

    #[test]
    fn test_get_proposals_pagination() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Yes"),
            String::from_str(&env, "No"),
        ]);
        
        // Create 5 proposals
        client.create_proposal(
            &creator,
            &String::from_str(&env, "Proposal 1"),
            &String::from_str(&env, "Description for proposal 1"),
            &voting_end,
            &options
        );
        client.create_proposal(
            &creator,
            &String::from_str(&env, "Proposal 2"),
            &String::from_str(&env, "Description for proposal 2"),
            &voting_end,
            &options
        );
        client.create_proposal(
            &creator,
            &String::from_str(&env, "Proposal 3"),
            &String::from_str(&env, "Description for proposal 3"),
            &voting_end,
            &options
        );
        client.create_proposal(
            &creator,
            &String::from_str(&env, "Proposal 4"),
            &String::from_str(&env, "Description for proposal 4"),
            &voting_end,
            &options
        );
        client.create_proposal(
            &creator,
            &String::from_str(&env, "Proposal 5"),
            &String::from_str(&env, "Description for proposal 5"),
            &voting_end,
            &options
        );
        
        // Query first 3 proposals (start=0, limit=3)
        let proposals = client.get_proposals(&0, &3);
        assert_eq!(proposals.len(), 3);
        assert_eq!(proposals.get(0).unwrap().id, 1);
        assert_eq!(proposals.get(1).unwrap().id, 2);
        assert_eq!(proposals.get(2).unwrap().id, 3);
        
        // Query next 2 proposals (start=3, limit=3)
        let proposals = client.get_proposals(&3, &3);
        assert_eq!(proposals.len(), 2);
        assert_eq!(proposals.get(0).unwrap().id, 4);
        assert_eq!(proposals.get(1).unwrap().id, 5);
        
        // Query beyond available proposals
        let proposals = client.get_proposals(&10, &5);
        assert_eq!(proposals.len(), 0);
    }

    #[test]
    fn test_get_proposal_votes() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let voter1 = Address::generate(&env);
        let voter2 = Address::generate(&env);
        let voter3 = Address::generate(&env);
        let title = String::from_str(&env, "Votes Query Test");
        let description = String::from_str(&env, "Testing get_proposal_votes function");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Option A"),
            String::from_str(&env, "Option B"),
        ]);
        
        // Create proposal
        let proposal_id = client.create_proposal(&creator, &title, &description, &voting_end, &options);
        
        // Cast votes
        client.cast_vote(&voter1, &proposal_id, &0);
        client.cast_vote(&voter2, &proposal_id, &1);
        client.cast_vote(&voter3, &proposal_id, &0);
        
        // Query all votes for the proposal
        let votes = client.get_proposal_votes(&proposal_id);
        
        // Verify we got all 3 votes
        assert_eq!(votes.len(), 3);
        
        // Verify vote data
        let vote1 = votes.get(0).unwrap();
        assert_eq!(vote1.proposal_id, proposal_id);
        assert_eq!(vote1.voter, voter1);
        assert_eq!(vote1.choice, 0);
    }

    #[test]
    fn test_get_voter_history() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let voter = Address::generate(&env);
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Yes"),
            String::from_str(&env, "No"),
        ]);
        
        // Create 3 proposals
        let proposal_id1 = client.create_proposal(
            &creator,
            &String::from_str(&env, "Proposal 1"),
            &String::from_str(&env, "First proposal for voter history test"),
            &voting_end,
            &options
        );
        let _proposal_id2 = client.create_proposal(
            &creator,
            &String::from_str(&env, "Proposal 2"),
            &String::from_str(&env, "Second proposal for voter history test"),
            &voting_end,
            &options
        );
        let proposal_id3 = client.create_proposal(
            &creator,
            &String::from_str(&env, "Proposal 3"),
            &String::from_str(&env, "Third proposal for voter history test"),
            &voting_end,
            &options
        );
        
        // Voter votes on proposals 1 and 3
        client.cast_vote(&voter, &proposal_id1, &0);
        client.cast_vote(&voter, &proposal_id3, &1);
        
        // Query voter history
        let history = client.get_voter_history(&voter);
        
        // Verify we got 2 votes
        assert_eq!(history.len(), 2);
        
        // Verify vote data
        let vote1 = history.get(0).unwrap();
        assert_eq!(vote1.proposal_id, proposal_id1);
        assert_eq!(vote1.voter, voter);
        assert_eq!(vote1.choice, 0);
        
        let vote2 = history.get(1).unwrap();
        assert_eq!(vote2.proposal_id, proposal_id3);
        assert_eq!(vote2.voter, voter);
        assert_eq!(vote2.choice, 1);
    }

    #[test]
    fn test_get_vote_results() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let voter1 = Address::generate(&env);
        let voter2 = Address::generate(&env);
        let voter3 = Address::generate(&env);
        let voter4 = Address::generate(&env);
        let title = String::from_str(&env, "Vote Results Test");
        let description = String::from_str(&env, "Testing get_vote_results function");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Option A"),
            String::from_str(&env, "Option B"),
            String::from_str(&env, "Option C"),
        ]);
        
        // Create proposal
        let proposal_id = client.create_proposal(&creator, &title, &description, &voting_end, &options);
        
        // Cast votes: 2 for A, 1 for B, 1 for C
        client.cast_vote(&voter1, &proposal_id, &0);
        client.cast_vote(&voter2, &proposal_id, &0);
        client.cast_vote(&voter3, &proposal_id, &1);
        client.cast_vote(&voter4, &proposal_id, &2);
        
        // Query vote results
        let results = client.get_vote_results(&proposal_id);
        
        // Verify results
        assert_eq!(results.proposal_id, proposal_id);
        assert_eq!(results.total_votes, 4);
        assert_eq!(results.unique_voters, 4);
        assert_eq!(results.option_counts.len(), 3);
        assert_eq!(results.option_counts.get(0).unwrap(), 2); // Option A
        assert_eq!(results.option_counts.get(1).unwrap(), 1); // Option B
        assert_eq!(results.option_counts.get(2).unwrap(), 1); // Option C
    }

    #[test]
    fn test_get_vote_results_no_votes() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "No Votes Test");
        let description = String::from_str(&env, "Testing vote results with no votes");
        let current_time = env.ledger().timestamp();
        let voting_end = current_time + 7200;
        let options = Vec::from_array(&env, [
            String::from_str(&env, "Yes"),
            String::from_str(&env, "No"),
        ]);
        
        // Create proposal
        let proposal_id = client.create_proposal(&creator, &title, &description, &voting_end, &options);
        
        // Query vote results without any votes
        let results = client.get_vote_results(&proposal_id);
        
        // Verify results
        assert_eq!(results.proposal_id, proposal_id);
        assert_eq!(results.total_votes, 0);
        assert_eq!(results.unique_voters, 0);
        assert_eq!(results.option_counts.len(), 2);
        assert_eq!(results.option_counts.get(0).unwrap(), 0);
        assert_eq!(results.option_counts.get(1).unwrap(), 0);
    }

    #[test]
    fn test_get_vote_results_proposal_not_found() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VotingContract);
        let client = VotingContractClient::new(&env, &contract_id);
        
        env.mock_all_auths();
        
        // Query results for non-existent proposal
        let result = client.try_get_vote_results(&999);
        assert_eq!(result, Err(Ok(ContractError::ProposalNotFound)));
    }
}

