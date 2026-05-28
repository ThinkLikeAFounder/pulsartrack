#![cfg(test)]
use super::*;
use soroban_sdk::{
    contract as soroban_contract, contractimpl as soroban_contractimpl,
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

// ─── Mock governance token ───────────────────────────────────────────────────
// The governance-dao's finalize_proposal invokes `total_supply` on the
// governance token via cross-contract call.  The built-in SAC in SDK 22
// testutils does not expose that function through the normal dispatch table, so
// we register a tiny mock contract that does.

#[soroban_contract]
pub struct MockGovToken;

#[soroban_contractimpl]
impl MockGovToken {
    /// Store total supply so finalize_proposal can read it.
    pub fn set_supply(env: Env, amount: i128) {
        env.storage().instance().set(&0u32, &amount);
    }

    /// Called by GovernanceDaoContract::finalize_proposal via invoke_contract.
    pub fn total_supply(env: Env) -> i128 {
        env.storage()
            .instance()
            .get::<u32, i128>(&0u32)
            .unwrap_or(0)
    }

    /// Set per-address balance for proposer-min-token tests.
    pub fn set_balance(env: Env, addr: Address, amount: i128) {
        env.storage().persistent().set(&addr, &amount);
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        env.storage()
            .persistent()
            .get::<Address, i128>(&id)
            .unwrap_or(1_000_000)
    }

    /// Token transfer: move `amount` from `from` to `to`.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let from_bal = env
            .storage()
            .persistent()
            .get::<Address, i128>(&from)
            .unwrap_or(1_000_000);
        if from_bal < amount {
            panic!("insufficient balance");
        }
        env.storage().persistent().set(&from, &(from_bal - amount));
        let to_bal = env
            .storage()
            .persistent()
            .get::<Address, i128>(&to)
            .unwrap_or(0);
        env.storage().persistent().set(&to, &(to_bal + amount));
    }
}

#[soroban_contract]
pub struct GovernanceExecutionTarget;

#[soroban_contractimpl]
impl GovernanceExecutionTarget {
    pub fn execute_governance(env: Env, proposal_id: u64) {
        env.storage().instance().set(&0u32, &proposal_id);
    }

    pub fn last_executed(env: Env) -> Option<u64> {
        env.storage().instance().get(&0u32)
    }
}

// ─── helpers ─────────────────────────────────────────────────────────────────

/// Deploy MockGovToken and set its total supply.
fn deploy_mock_gov_token(env: &Env, total_supply: i128) -> Address {
    let id = env.register_contract(None, MockGovToken);
    let client = MockGovTokenClient::new(env, &id);
    client.set_supply(&total_supply);
    id
}

/// Deploy + initialize with: voting_period=100 ledgers, quorum=10%, pass=51%.
/// Uses a plain address as the governance token for tests that don't need token gating.
/// Sets proposer_min = 0 so the token-balance check is skipped.
fn setup(env: &Env) -> (GovernanceDaoContractClient<'_>, Address, Address, Address) {
    let admin = Address::generate(env);
    let token_addr = deploy_mock_gov_token(env, 10_000_000);

    let contract_id = env.register_contract(None, GovernanceDaoContract);
    let client = GovernanceDaoContractClient::new(env, &contract_id);
    client.initialize(&admin, &token_addr, &100u32, &1_000u32, &51u32, &0i128);

    (client, admin, Address::generate(env), token_addr)
}

/// Deploy governance-dao initialized with a real MockGovToken (needed for finalize tests).
/// Sets proposer_min = 0 since these tests focus on finalize/execute, not token gating.
fn setup_with_mock_token(
    env: &Env,
    total_supply: i128,
) -> (GovernanceDaoContractClient<'_>, Address, Address) {
    let admin = Address::generate(env);
    let token_addr = deploy_mock_gov_token(env, total_supply);

    let contract_id = env.register_contract(None, GovernanceDaoContract);
    let client = GovernanceDaoContractClient::new(env, &contract_id);
    client.initialize(&admin, &token_addr, &100u32, &1_000u32, &51u32, &0i128);

    (client, admin, token_addr)
}

fn make_title(env: &Env) -> String {
    String::from_str(env, "Increase treasury allocation")
}

fn make_desc(env: &Env) -> String {
    String::from_str(env, "Proposal to increase budget by 10%")
}

// ─── initialize ──────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let token = Address::generate(&env);

    let contract_id = env.register_contract(None, GovernanceDaoContract);
    let client = GovernanceDaoContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token, &3600u32, &1000u32, &51u32, &100i128);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let token = Address::generate(&env);

    let contract_id = env.register_contract(None, GovernanceDaoContract);
    let client = GovernanceDaoContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token, &3600u32, &1000u32, &51u32, &100i128);
    client.initialize(&admin, &token, &3600u32, &1000u32, &51u32, &100i128);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    // no mock_all_auths → require_auth panics
    let admin = Address::generate(&env);
    let token = Address::generate(&env);

    let contract_id = env.register_contract(None, GovernanceDaoContract);
    let client = GovernanceDaoContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token, &3600u32, &1000u32, &51u32, &100i128);
}

// ─── create_proposal ─────────────────────────────────────────────────────────

#[test]
fn test_create_proposal() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    assert_eq!(proposal_id, 1);
    assert_eq!(client.get_proposal_count(), 1);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.proposer, proposer);
    assert!(matches!(proposal.status, ProposalStatus::Active));
    assert_eq!(proposal.votes_for, 0);
    assert_eq!(proposal.votes_against, 0);
    assert_eq!(proposal.votes_abstain, 0);
}

#[test]
fn test_create_multiple_proposals() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);

    client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);
    client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    assert_eq!(client.get_proposal_count(), 2);
}

// ─── cast_vote ────────────────────────────────────────────────────────────────

#[test]
fn test_cast_vote_for() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cast_vote(&voter, &proposal_id, &VoteChoice::For, &1_000i128);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.votes_for, 1_000);
    assert_eq!(proposal.votes_against, 0);
    assert_eq!(proposal.votes_abstain, 0);
    assert!(client.has_voted(&proposal_id, &voter));

    let vote = client.get_vote(&proposal_id, &voter).unwrap();
    assert_eq!(vote.power, 1_000);
}

#[test]
fn test_cast_vote_against() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cast_vote(&voter, &proposal_id, &VoteChoice::Against, &500i128);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.votes_against, 500);
    assert_eq!(proposal.votes_for, 0);
}

#[test]
fn test_cast_vote_abstain() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cast_vote(&voter, &proposal_id, &VoteChoice::Abstain, &200i128);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.votes_abstain, 200);
}

#[test]
fn test_vote_tally_accumulates() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);
    let voter_a = Address::generate(&env);
    let voter_b = Address::generate(&env);
    let voter_c = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cast_vote(&voter_a, &proposal_id, &VoteChoice::For, &300i128);
    client.cast_vote(&voter_b, &proposal_id, &VoteChoice::For, &200i128);
    client.cast_vote(&voter_c, &proposal_id, &VoteChoice::Against, &100i128);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.votes_for, 500);
    assert_eq!(proposal.votes_against, 100);
}

#[test]
#[should_panic(expected = "already voted")]
fn test_double_vote_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cast_vote(&voter, &proposal_id, &VoteChoice::For, &1_000i128);
    client.cast_vote(&voter, &proposal_id, &VoteChoice::Against, &1_000i128); // panic
}

#[test]
#[should_panic(expected = "invalid voting power")]
fn test_vote_zero_power_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cast_vote(&voter, &proposal_id, &VoteChoice::For, &0i128);
}

#[test]
#[should_panic(expected = "voting period ended")]
fn test_vote_after_period_ends() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    // start=0, period=100 → end=100; advance past it
    env.ledger().with_mut(|li| {
        li.sequence_number = 200;
    });

    client.cast_vote(&voter, &proposal_id, &VoteChoice::For, &1_000i128);
}

#[test]
#[should_panic(expected = "proposal not active")]
fn test_vote_on_cancelled_proposal() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cancel_proposal(&proposer, &proposal_id);
    client.cast_vote(&voter, &proposal_id, &VoteChoice::For, &1_000i128);
}

// ─── finalize_proposal ────────────────────────────────────────────────────────

#[test]
fn test_finalize_proposal_passed() {
    let env = Env::default();
    env.mock_all_auths();

    // total supply = 1_000; quorum_bps=1_000 (10%) → need ≥100 votes
    let (client, _, _) = setup_with_mock_token(&env, 1_000);

    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    // 200 for votes  (200*10_000 >= 1_000*1_000 → quorum met; 100% >= 51% → Passed)
    client.cast_vote(&voter, &proposal_id, &VoteChoice::For, &200i128);

    env.ledger().with_mut(|li| {
        li.sequence_number = 200;
    });

    client.finalize_proposal(&proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(matches!(proposal.status, ProposalStatus::Passed));
}

#[test]
fn test_finalize_proposal_rejected_no_quorum() {
    let env = Env::default();
    env.mock_all_auths();

    // total supply = 10_000; quorum needs ≥1_000 votes; cast only 50
    let (client, _, _) = setup_with_mock_token(&env, 10_000);

    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cast_vote(&voter, &proposal_id, &VoteChoice::For, &50i128);

    env.ledger().with_mut(|li| {
        li.sequence_number = 200;
    });

    client.finalize_proposal(&proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(matches!(proposal.status, ProposalStatus::Rejected));
}

#[test]
fn test_finalize_proposal_rejected_not_enough_for_votes() {
    let env = Env::default();
    env.mock_all_auths();

    // Need custom pass_threshold of 60%, so build inline
    let admin = Address::generate(&env);
    // total supply = 1_000; pass_threshold = 60%; for=40%, against=60% → Rejected
    let token_addr = deploy_mock_gov_token(&env, 1_000);
    let contract_id = env.register_contract(None, GovernanceDaoContract);
    let client = GovernanceDaoContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &100u32, &1_000u32, &60u32, &0i128);

    let proposer = Address::generate(&env);
    let voter_for = Address::generate(&env);
    let voter_against = Address::generate(&env);
    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cast_vote(&voter_for, &proposal_id, &VoteChoice::For, &400i128);
    client.cast_vote(&voter_against, &proposal_id, &VoteChoice::Against, &600i128);

    env.ledger().with_mut(|li| {
        li.sequence_number = 200;
    });

    client.finalize_proposal(&proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(matches!(proposal.status, ProposalStatus::Rejected));
}

#[test]
#[should_panic(expected = "voting period not ended")]
fn test_finalize_still_active() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    // sequence_number=0, end_ledger=100 → still active
    client.finalize_proposal(&proposal_id);
}

// ─── execute_proposal ────────────────────────────────────────────────────────

#[test]
fn test_execute_proposal() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, _) = setup_with_mock_token(&env, 1_000);

    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cast_vote(&voter, &proposal_id, &VoteChoice::For, &200i128);

    env.ledger().with_mut(|li| {
        li.sequence_number = 200;
    });

    client.finalize_proposal(&proposal_id);

    let p = client.get_proposal(&proposal_id).unwrap();
    assert!(matches!(p.status, ProposalStatus::Passed));

    client.execute_proposal(&admin, &proposal_id);

    let p = client.get_proposal(&proposal_id).unwrap();
    assert!(matches!(p.status, ProposalStatus::Executed));
    assert!(p.executed_at.is_some());
}

#[test]
#[should_panic(expected = "proposal not passed")]
fn test_execute_active_proposal_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, _) = setup(&env);
    let proposer = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.execute_proposal(&admin, &proposal_id);
}

#[test]
#[should_panic(expected = "proposal not passed")]
fn test_execute_proposal_twice_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, _) = setup_with_mock_token(&env, 1_000);

    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cast_vote(&voter, &proposal_id, &VoteChoice::For, &200i128);

    env.ledger().with_mut(|li| {
        li.sequence_number = 200;
    });

    client.finalize_proposal(&proposal_id);

    // First execution succeeds
    client.execute_proposal(&admin, &proposal_id);
    let p = client.get_proposal(&proposal_id).unwrap();
    assert!(matches!(p.status, ProposalStatus::Executed));

    // Second execution must be rejected — status is no longer Passed
    client.execute_proposal(&admin, &proposal_id);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_execute_proposal_by_stranger_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, _) = setup_with_mock_token(&env, 1_000);

    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let stranger = Address::generate(&env);
    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cast_vote(&voter, &proposal_id, &VoteChoice::For, &200i128);

    env.ledger().with_mut(|li| {
        li.sequence_number = 200;
    });

    client.finalize_proposal(&proposal_id);
    client.execute_proposal(&stranger, &proposal_id);
}

// ─── finalize_proposal — basis-point precision (#480) ────────────────────────

#[test]
fn test_finalize_proposal_passes_at_51_pct_exact() {
    let env = Env::default();
    env.mock_all_auths();

    // 510_000 For, 490_000 Against out of 1_000_000 total = exactly 51.0%
    // Old integer division: (510_000 * 100) / 1_000_000 = 51 → passes (fine here)
    // Basis-point check: (510_000 * 10_000) / 1_000_000 = 5100 >= 5100 → Passed
    let (client, _, _) = setup_with_mock_token(&env, 2_000_000);

    let proposer = Address::generate(&env);
    let voter_for = Address::generate(&env);
    let voter_against = Address::generate(&env);
    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cast_vote(&voter_for, &proposal_id, &VoteChoice::For, &510_000i128);
    client.cast_vote(
        &voter_against,
        &proposal_id,
        &VoteChoice::Against,
        &490_000i128,
    );

    env.ledger().with_mut(|li| {
        li.sequence_number = 200;
    });

    client.finalize_proposal(&proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(
        matches!(proposal.status, ProposalStatus::Passed),
        "exactly 51% For should pass a 51% threshold"
    );
}

#[test]
fn test_finalize_proposal_passes_at_51_1_pct() {
    let env = Env::default();
    env.mock_all_auths();

    // 511_000 For, 489_000 Against out of 1_000_000 total = 51.1%
    // Old integer division: (511_000 * 100) / 1_000_000 = 51 → passes
    // Basis-point check: (511_000 * 10_000) / 1_000_000 = 5110 >= 5100 → Passed
    // (Both agree here; the precision fix matters most near the boundary)
    let (client, _, _) = setup_with_mock_token(&env, 2_000_000);

    let proposer = Address::generate(&env);
    let voter_for = Address::generate(&env);
    let voter_against = Address::generate(&env);
    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cast_vote(&voter_for, &proposal_id, &VoteChoice::For, &511_000i128);
    client.cast_vote(
        &voter_against,
        &proposal_id,
        &VoteChoice::Against,
        &489_000i128,
    );

    env.ledger().with_mut(|li| {
        li.sequence_number = 200;
    });

    client.finalize_proposal(&proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(matches!(proposal.status, ProposalStatus::Passed));
}

#[test]
fn test_finalize_proposal_rejected_at_50_99_pct_with_bps() {
    let env = Env::default();
    env.mock_all_auths();

    // Use 10_000 total votes: 5_099 For, 4_901 Against = 50.99%
    // Old integer division: (5_099 * 100) / 10_000 = 50 → Rejected (wrong)
    // Basis-point check: (5_099 * 10_000) / 10_000 = 5099 < 5100 → Rejected (correct)
    // This confirms the bps fix correctly rejects sub-51% proposals
    let (client, _, _) = setup_with_mock_token(&env, 20_000);

    let proposer = Address::generate(&env);
    let voter_for = Address::generate(&env);
    let voter_against = Address::generate(&env);
    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cast_vote(&voter_for, &proposal_id, &VoteChoice::For, &5_099i128);
    client.cast_vote(
        &voter_against,
        &proposal_id,
        &VoteChoice::Against,
        &4_901i128,
    );

    env.ledger().with_mut(|li| {
        li.sequence_number = 200;
    });

    client.finalize_proposal(&proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(matches!(proposal.status, ProposalStatus::Rejected));
}

#[test]
fn test_finalize_proposal_rejected_at_exactly_50_pct() {
    let env = Env::default();
    env.mock_all_auths();

    // Exactly 50% For → must Reject (threshold is 51%)
    let (client, _, _) = setup_with_mock_token(&env, 2_000_000);

    let proposer = Address::generate(&env);
    let voter_for = Address::generate(&env);
    let voter_against = Address::generate(&env);
    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cast_vote(&voter_for, &proposal_id, &VoteChoice::For, &500_000i128);
    client.cast_vote(
        &voter_against,
        &proposal_id,
        &VoteChoice::Against,
        &500_000i128,
    );

    env.ledger().with_mut(|li| {
        li.sequence_number = 200;
    });

    client.finalize_proposal(&proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(matches!(proposal.status, ProposalStatus::Rejected));
}

// ─── execute_proposal — target_contract event (#479) ─────────────────────────

#[test]
fn test_execute_proposal_emits_executed_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, _) = setup_with_mock_token(&env, 1_000);

    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let target = env.register_contract(None, GovernanceExecutionTarget);
    let proposal_id = client.create_proposal(
        &proposer,
        &make_title(&env),
        &make_desc(&env),
        &Some(target.clone()),
    );

    client.cast_vote(&voter, &proposal_id, &VoteChoice::For, &200i128);

    env.ledger().with_mut(|li| {
        li.sequence_number = 200;
    });

    client.finalize_proposal(&proposal_id);
    client.execute_proposal(&admin, &proposal_id);

    let p = client.get_proposal(&proposal_id).unwrap();
    assert!(matches!(p.status, ProposalStatus::Executed));
    // target_contract is preserved on the proposal
    assert_eq!(p.target_contract, Some(target));
}

#[test]
fn test_execute_proposal_invokes_target_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, _) = setup_with_mock_token(&env, 1_000);
    let target = env.register_contract(None, GovernanceExecutionTarget);
    let target_client = GovernanceExecutionTargetClient::new(&env, &target);

    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let proposal_id = client.create_proposal(
        &proposer,
        &make_title(&env),
        &make_desc(&env),
        &Some(target.clone()),
    );

    client.cast_vote(&voter, &proposal_id, &VoteChoice::For, &200i128);

    env.ledger().with_mut(|li| {
        li.sequence_number = 200;
    });

    client.finalize_proposal(&proposal_id);
    client.execute_proposal(&admin, &proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(matches!(proposal.status, ProposalStatus::Executed));
    assert_eq!(target_client.last_executed(), Some(proposal_id));
}

#[test]
fn test_cancel_proposal_by_proposer() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cancel_proposal(&proposer, &proposal_id);

    let p = client.get_proposal(&proposal_id).unwrap();
    assert!(matches!(p.status, ProposalStatus::Cancelled));
}

#[test]
fn test_cancel_proposal_by_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, _) = setup(&env);
    let proposer = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cancel_proposal(&admin, &proposal_id);

    let p = client.get_proposal(&proposal_id).unwrap();
    assert!(matches!(p.status, ProposalStatus::Cancelled));
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_cancel_proposal_by_stranger_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);
    let stranger = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cancel_proposal(&stranger, &proposal_id);
}

#[test]
#[should_panic(expected = "can only cancel active proposals")]
fn test_cancel_passed_proposal_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, _) = setup_with_mock_token(&env, 1_000);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cast_vote(&voter, &proposal_id, &VoteChoice::For, &200i128);

    env.ledger().with_mut(|li| {
        li.sequence_number = 200;
    });

    client.finalize_proposal(&proposal_id);

    let p = client.get_proposal(&proposal_id).unwrap();
    assert!(matches!(p.status, ProposalStatus::Passed));

    // Must reject — proposal already Passed
    client.cancel_proposal(&admin, &proposal_id);
}

#[test]
#[should_panic(expected = "can only cancel active proposals")]
fn test_cancel_executed_proposal_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, _) = setup_with_mock_token(&env, 1_000);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    client.cast_vote(&voter, &proposal_id, &VoteChoice::For, &200i128);

    env.ledger().with_mut(|li| {
        li.sequence_number = 200;
    });

    client.finalize_proposal(&proposal_id);
    client.execute_proposal(&admin, &proposal_id);

    let p = client.get_proposal(&proposal_id).unwrap();
    assert!(matches!(p.status, ProposalStatus::Executed));

    // Must reject — proposal already Executed
    client.cancel_proposal(&admin, &proposal_id);
}

#[test]
#[should_panic(expected = "can only cancel active proposals")]
fn test_cancel_already_cancelled_proposal_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    // First cancel succeeds (proposal is Active)
    client.cancel_proposal(&proposer, &proposal_id);

    let p = client.get_proposal(&proposal_id).unwrap();
    assert!(matches!(p.status, ProposalStatus::Cancelled));

    // Second cancel must reject — already Cancelled
    client.cancel_proposal(&proposer, &proposal_id);
}

// ─── read-only helpers ────────────────────────────────────────────────────────

#[test]
fn test_has_voted_false_before_vote() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    assert!(!client.has_voted(&proposal_id, &voter));
}

#[test]
fn test_get_vote_none_before_voting() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    assert!(client.get_vote(&proposal_id, &voter).is_none());
}

#[test]
fn test_get_proposal_nonexistent_returns_none() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);

    assert!(client.get_proposal(&999u64).is_none());
}

#[test]
fn test_get_proposal_count_initial_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);

    assert_eq!(client.get_proposal_count(), 0);
}

// ─── TTL double-vote regression (#477) ───────────────────────────────────────

#[test]
#[should_panic(expected = "already voted")]
fn test_double_vote_after_ttl_window_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    // Use a long voting period (300 ledgers) so the proposal stays active
    // well past the old 15-day / 259_200-ledger TTL window.
    let admin = Address::generate(&env);
    let token_addr = deploy_mock_gov_token(&env, 10_000_000);
    let contract_id = env.register_contract(None, GovernanceDaoContract);
    let client = GovernanceDaoContractClient::new(&env, &contract_id);
    client.initialize(
        &admin,
        &token_addr,
        &300_000u32, // voting period: 300_000 ledgers (~35 days)
        &1_000u32,
        &51u32,
        &0i128,
    );

    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    // Voter A casts their vote at ledger 0
    client.cast_vote(&voter, &proposal_id, &VoteChoice::For, &1_000i128);
    assert!(client.has_voted(&proposal_id, &voter));

    // Advance the ledger past the OLD persistent bump amount (259_200 ≈ 15 days).
    // We step in increments smaller than INSTANCE_BUMP_AMOUNT (86_400) and make
    // a read-only call between each jump to keep the contract instance alive.
    for seq in [80_000u32, 160_000, 240_000, 260_000] {
        env.ledger().with_mut(|li| {
            li.sequence_number = seq;
        });
        // Read-only call bumps the instance TTL so the contract stays alive.
        let _ = client.get_proposal(&proposal_id);
    }

    // Voter A tries to vote again — must be rejected
    client.cast_vote(&voter, &proposal_id, &VoteChoice::For, &1_000i128);
}

// ─── proposer minimum token enforcement (#76) ────────────────────────────────

/// Deploy MockGovToken with balance support and initialize the DAO with a
/// non-zero proposer_min. Returns (client, admin, token_addr).
fn setup_with_token_gate(
    env: &Env,
    total_supply: i128,
    proposer_min: i128,
) -> (GovernanceDaoContractClient<'_>, Address, Address) {
    let admin = Address::generate(env);
    let token_addr = deploy_mock_gov_token(env, total_supply);

    let contract_id = env.register_contract(None, GovernanceDaoContract);
    let client = GovernanceDaoContractClient::new(env, &contract_id);
    client.initialize(
        &admin,
        &token_addr,
        &100u32,
        &1_000u32,
        &51u32,
        &proposer_min,
    );

    (client, admin, token_addr)
}

#[test]
fn test_create_proposal_with_sufficient_tokens() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, token_addr) = setup_with_token_gate(&env, 10_000, 100);
    let proposer = Address::generate(&env);

    // Give proposer enough tokens
    MockGovTokenClient::new(&env, &token_addr).set_balance(&proposer, &500);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    assert_eq!(proposal_id, 1);
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.proposer, proposer);
}

#[test]
#[should_panic(expected = "insufficient tokens to create proposal")]
fn test_create_proposal_insufficient_tokens() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, token_addr) = setup_with_token_gate(&env, 10_000, 100);
    let proposer = Address::generate(&env);

    // Give proposer fewer tokens than required
    MockGovTokenClient::new(&env, &token_addr).set_balance(&proposer, &50);

    // Should panic
    client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);
}

#[test]
#[should_panic(expected = "insufficient tokens to create proposal")]
fn test_create_proposal_zero_balance_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, token_addr) = setup_with_token_gate(&env, 10_000, 100);
    let proposer = Address::generate(&env);

    // Proposer has no tokens at all (set explicitly)
    MockGovTokenClient::new(&env, &token_addr).set_balance(&proposer, &0);
    client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);
}

#[test]
fn test_create_proposal_zero_min_allows_anyone() {
    let env = Env::default();
    env.mock_all_auths();

    // proposer_min = 0 → token check is skipped entirely
    let (client, _, _) = setup_with_token_gate(&env, 10_000, 0);
    let proposer = Address::generate(&env);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    assert_eq!(proposal_id, 1);
}

#[test]
fn test_create_proposal_exact_minimum_accepted() {
    let env = Env::default();
    env.mock_all_auths();

    let min = 250i128;
    let (client, _, token_addr) = setup_with_token_gate(&env, 10_000, min);
    let proposer = Address::generate(&env);

    // Proposer holds exactly the minimum
    MockGovTokenClient::new(&env, &token_addr).set_balance(&proposer, &min);

    let proposal_id = client.create_proposal(&proposer, &make_title(&env), &make_desc(&env), &None);

    assert_eq!(proposal_id, 1);
}
