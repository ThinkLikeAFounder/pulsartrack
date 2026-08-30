//! PulsarTrack - Governance DAO (Soroban)
//! On-chain DAO governance with proposals and voting on Stellar.
//!
//! Events:
//! - ("proposal", "created"): [proposal_id: u64, proposer: Address]
//! - ("gov", "voted"): [proposal_id: u64, voter: Address, power: i128]
//! - ("proposal", "finalized"): [proposal_id: u64, status: ProposalStatus]

#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, IntoVal, String,
};

// ============================================================
// Governance token voting interface
// ============================================================

/// The subset of the governance token used for vote weighting.
///
/// `get_past_votes` resolves an account's voting power from the token's
/// checkpoint history, using only checkpoints strictly older than the ledger
/// asked about. Governance must weight votes with this rather than with a live
/// `balance()` read, which is what made flash-loan vote inflation possible.
#[soroban_sdk::contractclient(name = "GovTokenClient")]
pub trait GovTokenInterface {
    fn get_past_votes(env: Env, account: Address, ledger_sequence: u32) -> i128;
}

// ============================================================
// Data Types
// ============================================================

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
    Cancelled,
    Expired,
}

#[contracttype]
#[derive(Clone)]
pub enum VoteChoice {
    For,
    Against,
    Abstain,
}

#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    pub proposer: Address,
    pub title: String,
    pub description: String,
    pub target_contract: Option<Address>,
    pub status: ProposalStatus,
    pub votes_for: i128,
    pub votes_against: i128,
    pub votes_abstain: i128,
    pub quorum_bps: u32,
    pub threshold_pct: u32, // percentage to pass
    pub start_ledger: u32,
    pub end_ledger: u32,
    /// Ledger the vote weights are measured at. Voting power is read strictly
    /// before this ledger, so tokens acquired at or after proposal creation
    /// carry no weight.
    pub snapshot_ledger: u32,
    pub created_at: u64,
    pub executed_at: Option<u64>,
}

#[contracttype]
#[derive(Clone)]
pub struct Vote {
    pub choice: VoteChoice,
    pub power: i128,
    pub voted_at: u64,
}

// ============================================================
// Storage Keys
// ============================================================

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    GovernanceToken,
    ProposalCounter,
    VotingPeriod,
    QuorumBps,
    PassThreshold,
    ProposerMinTokens,
    Proposal(u64),
    Vote(u64, Address),
    HasVoted(u64, Address),
    LockedTokens(u64, Address), // proposal_id, voter
}

// ============================================================
// Contract
// ============================================================

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960; // ~7 days
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200; // ~61 days

#[contract]
pub struct GovernanceDaoContract;

#[contractimpl]
impl GovernanceDaoContract {
    /// Initialize governance DAO
    pub fn initialize(
        env: Env,
        admin: Address,
        governance_token: Address,
        voting_period: u32,  // in ledgers
        quorum_bps: u32,     // basis points (e.g., 1000 = 10%)
        pass_threshold: u32, // percentage (e.g., 51)
        proposer_min: i128,  // min tokens to create proposal
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        if quorum_bps == 0 {
            panic!("quorum must be > 0");
        }
        if pass_threshold == 0 || pass_threshold > 100 {
            panic!("pass_threshold must be 1-100");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::GovernanceToken, &governance_token);
        env.storage()
            .instance()
            .set(&DataKey::ProposalCounter, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::VotingPeriod, &voting_period);
        env.storage()
            .instance()
            .set(&DataKey::QuorumBps, &quorum_bps);
        env.storage()
            .instance()
            .set(&DataKey::PassThreshold, &pass_threshold);
        env.storage()
            .instance()
            .set(&DataKey::ProposerMinTokens, &proposer_min);
    }

    /// Create a new governance proposal
    pub fn create_proposal(
        env: Env,
        proposer: Address,
        title: String,
        description: String,
        target_contract: Option<Address>,
    ) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        proposer.require_auth();

        // Enforce minimum token requirement for proposal creation
        let min_tokens: i128 = env
            .storage()
            .instance()
            .get(&DataKey::ProposerMinTokens)
            .unwrap_or(0);
        if min_tokens > 0 {
            let gov_token: Address = env
                .storage()
                .instance()
                .get(&DataKey::GovernanceToken)
                .unwrap();
            let balance = token::Client::new(&env, &gov_token).balance(&proposer);
            if balance < min_tokens {
                panic!("insufficient tokens to create proposal");
            }
        }

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCounter)
            .unwrap_or(0);
        let proposal_id = counter + 1;

        let voting_period: u32 = env
            .storage()
            .instance()
            .get(&DataKey::VotingPeriod)
            .unwrap_or(1_000);
        let quorum_bps: u32 = env
            .storage()
            .instance()
            .get(&DataKey::QuorumBps)
            .unwrap_or(0);
        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::PassThreshold)
            .unwrap_or(51);

        let start = env.ledger().sequence();
        let proposal = Proposal {
            proposer: proposer.clone(),
            title,
            description,
            target_contract,
            status: ProposalStatus::Active,
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            quorum_bps,
            threshold_pct: threshold,
            start_ledger: start,
            end_ledger: start + voting_period,
            snapshot_ledger: start,
            created_at: env.ledger().timestamp(),
            executed_at: None,
        };

        let _ttl_key = DataKey::Proposal(proposal_id);
        env.storage().persistent().set(&_ttl_key, &proposal);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::ProposalCounter, &proposal_id);

        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("created")),
            (proposal_id, proposer),
        );

        proposal_id
    }

    /// Cast a vote on a proposal
    pub fn cast_vote(env: Env, voter: Address, proposal_id: u64, choice: VoteChoice, power: i128) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        voter.require_auth();

        // Check not already voted
        if env
            .storage()
            .persistent()
            .has(&DataKey::HasVoted(proposal_id, voter.clone()))
        {
            panic!("already voted");
        }

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        if proposal.status != ProposalStatus::Active {
            panic!("proposal not active");
        }

        if env.ledger().sequence() > proposal.end_ledger {
            panic!("voting period ended");
        }

        let gov_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::GovernanceToken)
            .unwrap();

        // Weight the vote by the power held strictly BEFORE this proposal's
        // snapshot ledger, not by the voter's live balance. A live read let an
        // attacker borrow tokens, vote on the inflated balance, and repay in the
        // same transaction; power acquired at or after the snapshot ledger now
        // resolves to the pre-existing checkpoint instead.
        let voting_power = GovTokenClient::new(&env, &gov_token)
            .get_past_votes(&voter, &proposal.snapshot_ledger);
        if power > voting_power {
            panic!("insufficient governance tokens");
        }

        if power <= 0 {
            panic!("invalid voting power");
        }

        let vote = Vote {
            choice: choice.clone(),
            power,
            voted_at: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::Vote(proposal_id, voter.clone());
        env.storage().persistent().set(&_ttl_key, &vote);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        let _ttl_key = DataKey::HasVoted(proposal_id, voter.clone());
        env.storage().persistent().set(&_ttl_key, &true);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        // Record locked tokens
        let _ttl_key = DataKey::LockedTokens(proposal_id, voter.clone());
        env.storage().persistent().set(&_ttl_key, &power);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        // Record vote
        match choice {
            VoteChoice::For => proposal.votes_for += power,
            VoteChoice::Against => proposal.votes_against += power,
            VoteChoice::Abstain => proposal.votes_abstain += power,
        }

        let _ttl_key = DataKey::Proposal(proposal_id);
        env.storage().persistent().set(&_ttl_key, &proposal);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        // Lock tokens after recording the vote guard.
        token::Client::new(&env, &gov_token).transfer(
            &voter,
            &env.current_contract_address(),
            &power,
        );

        env.events().publish(
            (symbol_short!("gov"), symbol_short!("voted")),
            (proposal_id, voter, power),
        );
    }

    /// Finalize a proposal after voting period
    pub fn finalize_proposal(env: Env, proposal_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        if proposal.status != ProposalStatus::Active {
            panic!("proposal not active");
        }

        if env.ledger().sequence() <= proposal.end_ledger {
            panic!("voting period not ended");
        }

        let token_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::GovernanceToken)
            .unwrap();
        let total_supply: i128 = env.invoke_contract(
            &token_address,
            &soroban_sdk::Symbol::new(&env, "total_supply"),
            soroban_sdk::vec![&env],
        );

        let total_votes = proposal.votes_for + proposal.votes_against + proposal.votes_abstain;

        let quorum_met = (total_votes * 10_000) >= (total_supply * (proposal.quorum_bps as i128));

        let for_bps = if total_votes > 0 {
            (proposal.votes_for * 10_000) / total_votes
        } else {
            0
        };

        proposal.status = if quorum_met && for_bps as u32 >= proposal.threshold_pct * 100 {
            ProposalStatus::Passed
        } else if !quorum_met {
            ProposalStatus::Rejected
        } else {
            ProposalStatus::Rejected
        };

        let _ttl_key = DataKey::Proposal(proposal_id);
        env.storage().persistent().set(&_ttl_key, &proposal);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("finalized")),
            (proposal_id, proposal.status),
        );
    }

    /// Mark proposal as executed (admin only)
    pub fn execute_proposal(env: Env, admin: Address, proposal_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        if proposal.status != ProposalStatus::Passed {
            panic!("proposal not passed");
        }

        // CEI: write the terminal status to storage BEFORE any side effects so
        // that a re-entrant or concurrent call within the same transaction sees
        // `Executed` and is rejected by the status check above.
        proposal.status = ProposalStatus::Executed;
        proposal.executed_at = Some(env.ledger().timestamp());

        let _ttl_key = DataKey::Proposal(proposal_id);
        env.storage().persistent().set(&_ttl_key, &proposal);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        // Any execution side effects (e.g. calling proposal.target_contract)
        // must be placed here — after the status has been committed — so they
        // cannot be replayed if they succeed but the status write were to fail.
        if let Some(ref target) = proposal.target_contract {
            env.invoke_contract::<()>(
                target,
                &soroban_sdk::Symbol::new(&env, "execute_governance"),
                soroban_sdk::vec![&env, proposal_id.into_val(&env)],
            );
        }

        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("executed")),
            (proposal_id, proposal.target_contract),
        );
    }

    /// Unlock tokens after proposal is finalized
    pub fn unlock_tokens(env: Env, voter: Address, proposal_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        voter.require_auth();

        let proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        // Can only unlock if the proposal is no longer Active
        if proposal.status == ProposalStatus::Active
            && env.ledger().sequence() <= proposal.end_ledger
        {
            panic!("proposal still active");
        }

        let key = DataKey::LockedTokens(proposal_id, voter.clone());
        let amount: i128 = env
            .storage()
            .persistent()
            .get(&key)
            .expect("no tokens locked for this proposal");

        // Remove record before transfer (CEI)
        env.storage().persistent().remove(&key);

        let gov_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::GovernanceToken)
            .unwrap();
        let token_client = token::Client::new(&env, &gov_token);
        token_client.transfer(&env.current_contract_address(), &voter, &amount);

        env.events().publish(
            (symbol_short!("gov"), symbol_short!("unlocked")),
            (proposal_id, voter, amount),
        );
    }

    /// Cancel a proposal (proposer or admin)
    pub fn cancel_proposal(env: Env, caller: Address, proposal_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        caller.require_auth();

        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        if caller != proposal.proposer && caller != admin {
            panic!("unauthorized");
        }

        if proposal.status != ProposalStatus::Active {
            panic!("can only cancel an active proposal");
        }

        proposal.status = ProposalStatus::Cancelled;
        let _ttl_key = DataKey::Proposal(proposal_id);
        env.storage().persistent().set(&_ttl_key, &proposal);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    // ============================================================
    // Read-Only Functions
    // ============================================================

    pub fn get_proposal(env: Env, proposal_id: u64) -> Option<Proposal> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
    }

    pub fn get_vote(env: Env, proposal_id: u64, voter: Address) -> Option<Vote> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Vote(proposal_id, voter))
    }

    pub fn has_voted(env: Env, proposal_id: u64, voter: Address) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::HasVoted(proposal_id, voter))
            .unwrap_or(false)
    }

    pub fn get_proposal_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .instance()
            .get(&DataKey::ProposalCounter)
            .unwrap_or(0)
    }

    pub fn propose_admin(env: Env, current_admin: Address, new_admin: Address) {
        pulsar_common_admin::propose_admin(
            &env,
            &DataKey::Admin,
            &DataKey::PendingAdmin,
            current_admin,
            new_admin,
        );
    }

    pub fn accept_admin(env: Env, new_admin: Address) {
        pulsar_common_admin::accept_admin(&env, &DataKey::Admin, &DataKey::PendingAdmin, new_admin);
    }
}

mod test;
