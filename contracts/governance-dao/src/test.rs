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
        env.storage().instance().get::<u32, i128>(&0u32).unwrap_or(0)
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
/// Uses a plain address as the governance token for tests that don't finalize.
fn setup(env: &Env) -> (GovernanceDaoContractClient, Address, Address, Address) {
    let admin = Address::generate(env);
    // For simple tests (no finalize) any address works as the governance token.
    let token_addr = Address::generate(env);

    let contract_id = env.register_contract(None, GovernanceDaoContract);
    let client = GovernanceDaoContractClient::new(env, &contract_id);
    client.initialize(&admin, &token_addr, &100u32, &1_000u32, &51u32, &1i128);

    (client, admin, Address::generate(env), token_addr)
}

/// Deploy governance-dao initialized with a real MockGovToken (needed for finalize tests).
fn setup_with_mock_token(
    env: &Env,
    total_supply: i128,
) -> (GovernanceDaoContractClient, Address, Address) {
    let admin = Address::generate(env);
    let token_addr = deploy_mock_gov_token(env, total_supply);

    let contract_id = env.register_contract(None, GovernanceDaoContract);
    let client = GovernanceDaoContractClient::new(env, &contract_id);
    client.initialize(&admin, &token_addr, &100u32, &1_000u32, &51u32, &1i128);

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

    let proposal_id = client.create_proposal(
        &proposer,
        &make_title(&env),
        &make_desc(&env),
        &None,
    );

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

    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

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

    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

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

    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

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

    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

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

    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

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

    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

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

    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

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

    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

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
    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

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
    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

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
    client.initialize(&admin, &token_addr, &100u32, &1_000u32, &60u32, &1i128);

    let proposer = Address::generate(&env);
    let voter_for = Address::generate(&env);
    let voter_against = Address::generate(&env);
    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

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

    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

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
    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

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

    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

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
    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

    client.cast_vote(&voter, &proposal_id, &VoteChoice::For, &200i128);

    env.ledger().with_mut(|li| {
        li.sequence_number = 200;
    });

    client.finalize_proposal(&proposal_id);
    client.execute_proposal(&stranger, &proposal_id);
}

// ─── cancel_proposal ─────────────────────────────────────────────────────────

#[test]
fn test_cancel_proposal_by_proposer() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);

    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

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

    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

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

    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

    client.cancel_proposal(&stranger, &proposal_id);
}

// ─── read-only helpers ────────────────────────────────────────────────────────

#[test]
fn test_has_voted_false_before_vote() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

    assert!(!client.has_voted(&proposal_id, &voter));
}

#[test]
fn test_get_vote_none_before_voting() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    let proposal_id = client.create_proposal(
        &proposer, &make_title(&env), &make_desc(&env), &None,
    );

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
