#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    Address, BytesN, Env, IntoVal,
};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn setup(env: &Env) -> (FraudPreventionContractClient<'_>, Address) {
    let admin = Address::generate(env);

    let contract_id = env.register(FraudPreventionContract, ());
    let client = FraudPreventionContractClient::new(env, &contract_id);
    client.initialize(&admin);

    (client, admin)
}

// ─── initialize ──────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(FraudPreventionContract, ());
    let client = FraudPreventionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.initialize(&admin);
    let stored_admin = env.as_contract(&client.address, || {
        env.storage().instance().get(&DataKey::Admin)
    });
    assert_eq!(stored_admin, Some(admin));
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(FraudPreventionContract, ());
    let client = FraudPreventionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.initialize(&admin);
    client.initialize(&admin);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_initialize_non_admin_fails() {
    let env = Env::default();

    let contract_id = env.register(FraudPreventionContract, ());
    let client = FraudPreventionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    // This should panic because admin didn't authorize it and we haven't mocked it
    client.initialize(&admin);
}

// ─── set_dependent_contracts ─────────────────────────────────────────────────

#[test]
fn test_set_dependent_contracts() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let lifecycle = env.register(mocks::CampaignLifecycleContract, ());
    let network = Address::generate(&env);
    let vault = Address::generate(&env);

    client.set_dependent_contracts(&admin, &lifecycle, &network, &vault);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_dependent_contracts_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let stranger = Address::generate(&env);

    let lifecycle = Address::generate(&env);
    let network = Address::generate(&env);
    let vault = Address::generate(&env);

    client.set_dependent_contracts(&stranger, &lifecycle, &network, &vault);
}

// ─── set_threshold ───────────────────────────────────────────────────────────

#[test]
fn test_set_threshold() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    client.set_threshold(&admin, &90u32);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_threshold_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let stranger = Address::generate(&env);

    client.set_threshold(&stranger, &90u32);
}

#[test]
#[should_panic(expected = "invalid threshold")]
fn test_set_threshold_too_low() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    client.set_threshold(&admin, &40u32); // < 50
}

#[test]
#[should_panic(expected = "invalid threshold")]
fn test_set_threshold_too_high() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    client.set_threshold(&admin, &101u32); // > 100
}

// ─── oracle management ───────────────────────────────────────────────────────

#[test]
fn test_add_oracle_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let oracle = Address::generate(&env);
    client.add_oracle(&admin, &oracle);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_add_oracle_unauthorized_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let stranger = Address::generate(&env);
    let oracle = Address::generate(&env);

    client.add_oracle(&stranger, &oracle);
}

#[test]
fn test_remove_oracle() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let oracle = Address::generate(&env);
    client.add_oracle(&admin, &oracle);
    client.remove_oracle(&admin, &oracle);
}

// ─── flag_suspicious ─────────────────────────────────────────────────────────

#[test]
fn test_flag_suspicious_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let publisher = Address::generate(&env);

    client.flag_suspicious(&admin, &publisher);

    let status = client.get_suspicious_status(&publisher).unwrap();
    assert_eq!(status.suspicious_views, 1);
    assert_eq!(status.total_flags, 1);
    assert!(!status.suspended);
}

#[test]
fn test_flag_suspicious_oracle() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let oracle = Address::generate(&env);
    let publisher = Address::generate(&env);

    client.add_oracle(&admin, &oracle);
    client.flag_suspicious(&oracle, &publisher);

    let status = client.get_suspicious_status(&publisher).unwrap();
    assert_eq!(status.suspicious_views, 1);
    assert!(!status.suspended);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_flag_suspicious_unauthorized_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let stranger = Address::generate(&env);
    let publisher = Address::generate(&env);

    client.flag_suspicious(&stranger, &publisher);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_flag_suspicious_without_auth_fails() {
    let env = Env::default();
    // No mock_all_auths() here
    let (client, admin) = setup(&env);
    let publisher = Address::generate(&env);

    client.flag_suspicious(&admin, &publisher);
}

#[test]
fn test_flag_suspicious_accumulates() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let publisher = Address::generate(&env);

    client.flag_suspicious(&admin, &publisher);
    client.flag_suspicious(&admin, &publisher);
    client.flag_suspicious(&admin, &publisher);

    let status = client.get_suspicious_status(&publisher).unwrap();
    assert_eq!(status.suspicious_views, 3);
    assert_eq!(status.total_flags, 3);
}

// ─── suspend_publisher ───────────────────────────────────────────────────────

#[test]
fn test_suspend_publisher_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let lifecycle = Address::generate(&env);
    let network = env.register(mocks::PublisherNetworkContract, ());
    let vault = Address::generate(&env);
    let publisher = Address::generate(&env);

    client.set_dependent_contracts(&admin, &lifecycle, &network, &vault);
    client.suspend_publisher(&admin, &publisher);

    assert!(client.is_publisher_suspended(&publisher));
}

#[test]
#[should_panic(expected = "publisher network contract not configured")]
fn test_suspend_publisher_without_network_config_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let publisher = Address::generate(&env);

    client.suspend_publisher(&admin, &publisher);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_suspend_publisher_unauthorized_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let stranger = Address::generate(&env);
    let publisher = Address::generate(&env);

    client.suspend_publisher(&stranger, &publisher);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_suspend_publisher_oracle_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let oracle = Address::generate(&env);
    let publisher = Address::generate(&env);

    client.add_oracle(&admin, &oracle);
    // Oracle can flag, but cannot suspend
    client.suspend_publisher(&oracle, &publisher);
}

// ─── clear_flag ──────────────────────────────────────────────────────────────

#[test]
fn test_clear_flag() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let publisher = Address::generate(&env);
    client.flag_suspicious(&admin, &publisher);
    assert!(client.get_suspicious_status(&publisher).is_some());

    client.clear_flag(&admin, &publisher);
    assert!(client.get_suspicious_status(&publisher).is_none());
}

#[test]
fn test_clear_flag_reactivates_publisher_in_network() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let lifecycle = Address::generate(&env);
    let network = env.register(mocks::PublisherNetworkContract, ());
    let vault = Address::generate(&env);
    let publisher = Address::generate(&env);
    client.set_dependent_contracts(&admin, &lifecycle, &network, &vault);
    let network_client = mocks::PublisherNetworkContractClient::new(&env, &network);

    client.suspend_publisher(&admin, &publisher);
    assert!(client.is_publisher_suspended(&publisher));
    assert!(network_client.is_suspended(&publisher));

    client.clear_flag(&admin, &publisher);

    // Both the local flag and the real publisher-network state are lifted.
    assert!(!client.is_publisher_suspended(&publisher));
    assert!(!network_client.is_suspended(&publisher));
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_clear_flag_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let stranger = Address::generate(&env);
    let publisher = Address::generate(&env);

    client.flag_suspicious(&admin, &publisher);
    client.clear_flag(&stranger, &publisher);
}

// ─── verify_view ─────────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_verify_view_without_publisher_auth_fails() {
    let env = Env::default();
    let (client, admin) = {
        env.mock_all_auths();
        setup(&env)
    };

    let publisher = Address::generate(&env);
    let viewer = Address::generate(&env);
    let lifecycle = env.register(mocks::CampaignLifecycleContract, ());
    let network = Address::generate(&env);
    let vault = Address::generate(&env);

    env.mock_all_auths();
    client.set_dependent_contracts(&admin, &lifecycle, &network, &vault);

    // Clear auth mocks before calling verify_view with no authorization for `publisher`.
    let env_unauth = Env::default();
    let admin_u = Address::generate(&env_unauth);
    let contract_id = env_unauth.register(FraudPreventionContract, ());
    let unauth_client = FraudPreventionContractClient::new(&env_unauth, &contract_id);
    env_unauth.mock_all_auths();
    unauth_client.initialize(&admin_u);

    // Re-run on `env` where `publisher` has no auth mocked:
    let _ = (&client, &publisher, &viewer, &lifecycle);
    let fresh_env = Env::default();
    let fresh_admin = Address::generate(&fresh_env);
    let fresh_cid = fresh_env.register(FraudPreventionContract, ());
    let fresh_client = FraudPreventionContractClient::new(&fresh_env, &fresh_cid);
    {
        fresh_env.mock_all_auths();
        fresh_client.initialize(&fresh_admin);
    }
    let fresh_publisher = Address::generate(&fresh_env);
    let fresh_viewer = Address::generate(&fresh_env);
    // No mock_all_auths() active on `fresh_env` -> panics with Auth InvalidAction
    fresh_client.verify_view(&1u64, &fresh_publisher, &fresh_viewer, &None);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_verify_view_wrong_signer_fails() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(FraudPreventionContract, ());
    let client = FraudPreventionContractClient::new(&env, &contract_id);
    {
        env.mock_all_auths();
        client.initialize(&admin);
    }

    let publisher = Address::generate(&env);
    let imposter = Address::generate(&env);
    let viewer = Address::generate(&env);

    // Authorize `imposter` instead of `publisher` for `verify_view`.
    env.mock_auths(&[MockAuth {
        address: &imposter,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "verify_view",
            args: (1u64, &publisher, &viewer, Option::<BytesN<32>>::None).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.verify_view(&1u64, &publisher, &viewer, &None);
}

#[test]
fn test_verify_view_legitimate_publisher_with_auth() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(FraudPreventionContract, ());
    let client = FraudPreventionContractClient::new(&env, &contract_id);
    {
        env.mock_all_auths();
        client.initialize(&admin);
    }

    let publisher = Address::generate(&env);
    let viewer = Address::generate(&env);
    let proof = BytesN::from_array(&env, &[9u8; 32]);

    // Authorize specifically `publisher` for `verify_view` without broad `mock_all_auths()`.
    env.mock_auths(&[MockAuth {
        address: &publisher,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "verify_view",
            args: (1u64, &publisher, &viewer, Some(proof.clone())).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let verified = client.verify_view(&1u64, &publisher, &viewer, &Some(proof));
    assert!(verified);
    assert_eq!(client.get_total_verifications(), 1);
}

#[test]
fn test_verify_view_with_proof() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let publisher = Address::generate(&env);
    let viewer = Address::generate(&env);

    let lifecycle = Address::generate(&env);
    let network = Address::generate(&env);
    let vault = Address::generate(&env);
    client.set_dependent_contracts(&admin, &lifecycle, &network, &vault);

    // Threshold is 80, base_score=80 + proof_bonus=10 = 90 >= 80 → verified
    let proof = BytesN::from_array(&env, &[1u8; 32]);
    let result = client.verify_view(&1u64, &publisher, &viewer, &Some(proof));
    assert!(result);

    assert_eq!(client.get_total_verifications(), 1);
}

#[test]
fn test_verify_view_without_proof() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let publisher = Address::generate(&env);
    let viewer = Address::generate(&env);

    let lifecycle = env.register(mocks::CampaignLifecycleContract, ());
    let network = Address::generate(&env);
    let vault = Address::generate(&env);
    client.set_dependent_contracts(&admin, &lifecycle, &network, &vault);

    // Missing proof scores below the default threshold and is rejected.
    // The lifecycle mock is configured so the low-score fraud-pause call succeeds.
    let result = client.verify_view(&1u64, &publisher, &viewer, &None);
    assert!(!result);
    assert_eq!(client.get_verification_stats(&1u64).rejected_views, 1);
    let lifecycle_client = mocks::CampaignLifecycleContractClient::new(&env, &lifecycle);
    assert_eq!(lifecycle_client.paused_campaign(), Some(1u64));
}

#[test]
fn test_verification_stats_updated() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let publisher = Address::generate(&env);
    let viewer = Address::generate(&env);

    let lifecycle = Address::generate(&env);
    let network = Address::generate(&env);
    let vault = Address::generate(&env);
    client.set_dependent_contracts(&admin, &lifecycle, &network, &vault);

    let proof = BytesN::from_array(&env, &[2u8; 32]);
    assert!(client.verify_view(&1u64, &publisher, &viewer, &Some(proof)));

    let stats = client.get_verification_stats(&1u64);
    assert_eq!(stats.total_views, 1);
    assert_eq!(stats.verified_views, 1);
    assert_eq!(stats.rejected_views, 0);
}

// ─── read-only ───────────────────────────────────────────────────────────────

#[test]
fn test_get_suspicious_status_unknown() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let unknown = Address::generate(&env);

    assert!(client.get_suspicious_status(&unknown).is_none());
}

#[test]
fn test_get_total_verifications_initial() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    assert_eq!(client.get_total_verifications(), 0);
}

#[test]
fn test_fraud_integration() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(FraudPreventionContract, ());
    let client = FraudPreventionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let lifecycle = Address::generate(&env);
    let network = Address::generate(&env);
    let vault = Address::generate(&env);
    let publisher = Address::generate(&env);

    client.initialize(&admin);
    client.set_dependent_contracts(&admin, &lifecycle, &network, &vault);

    client.flag_suspicious(&admin, &publisher);
}
#[test]
fn test_duplicate_view_prevention_same_campaign() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let publisher1 = Address::generate(&env);
    let publisher2 = Address::generate(&env);
    let viewer1 = Address::generate(&env);
    let viewer2 = Address::generate(&env);

    let lifecycle = Address::generate(&env);
    let network = Address::generate(&env);
    let vault = Address::generate(&env);
    client.set_dependent_contracts(&admin, &lifecycle, &network, &vault);

    let proof = BytesN::from_array(&env, &[3u8; 32]);

    // Initial verify
    assert!(client.verify_view(&1u64, &publisher1, &viewer1, &Some(proof.clone())));

    // Same campaign, different viewer — distinct triplet, allowed
    assert!(client.verify_view(&1u64, &publisher1, &viewer2, &Some(proof.clone())));

    // Same campaign, different publisher — distinct triplet, allowed
    assert!(client.verify_view(&1u64, &publisher2, &viewer1, &Some(proof)));
}

#[test]
#[should_panic(expected = "duplicate view")]
fn test_duplicate_view_same_triplet_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let publisher = Address::generate(&env);
    let viewer = Address::generate(&env);

    let lifecycle = env.register(mocks::CampaignLifecycleContract, ());
    let network = Address::generate(&env);
    let vault = Address::generate(&env);
    client.set_dependent_contracts(&admin, &lifecycle, &network, &vault);

    client.verify_view(&1u64, &publisher, &viewer, &None);

    // Same triplet after a timestamp advance must still be rejected
    env.ledger().with_mut(|li| {
        li.timestamp += 1;
    });
    client.verify_view(&1u64, &publisher, &viewer, &None);
}

#[test]
#[should_panic(expected = "duplicate view")]
fn test_failed_view_retry_is_deduplicated() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let publisher = Address::generate(&env);
    let viewer = Address::generate(&env);
    let lifecycle = env.register(mocks::CampaignLifecycleContract, ());
    let network = Address::generate(&env);
    let vault = Address::generate(&env);
    client.set_dependent_contracts(&admin, &lifecycle, &network, &vault);

    // First failed verification returns false but commits the dedup/rate/cache writes.
    assert!(!client.verify_view(&77u64, &publisher, &viewer, &None));
    assert_eq!(client.get_verification_stats(&77u64).rejected_views, 1);

    // Retrying the identical triplet must hit the persisted duplicate guard.
    client.verify_view(&77u64, &publisher, &viewer, &None);
}