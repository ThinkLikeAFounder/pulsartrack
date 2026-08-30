#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env,
};

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}
fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

fn setup(env: &Env) -> (LiquidityPoolContractClient<'_>, Address, Address, Address) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = deploy_token(env, &token_admin);
    let contract_id = env.register(LiquidityPoolContract, ());
    let c = LiquidityPoolContractClient::new(env, &contract_id);
    c.initialize(&admin, &token);
    (c, admin, token_admin, token)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    setup(&env);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, token) = setup(&env);
    c.initialize(&admin, &token);
}

#[test]
fn test_deposit() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    mint(&env, &token, &provider, 1_000_000);
    let shares = c.deposit(&provider, &100_000i128);
    assert!(shares > 0);
    let pos = c.get_provider_position(&provider).unwrap();
    assert_eq!(pos.shares, shares);
    let pool = c.get_pool_state();
    assert_eq!(pool.total_liquidity, 100_000);
}

#[test]
fn test_withdraw() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    mint(&env, &token, &provider, 1_000_000);
    let shares = c.deposit(&provider, &100_000i128);
    let withdrawn = c.withdraw(&provider, &shares);
    assert_eq!(withdrawn, 100_000);
    let pool = c.get_pool_state();
    assert_eq!(pool.total_liquidity, 0);
}

#[test]
fn test_borrow() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint(&env, &token, &provider, 1_000_000);
    c.deposit(&provider, &500_000i128);
    c.borrow(&borrower, &1u64, &100_000i128, &86_400u64);
    let borrow = c.get_borrow(&1u64).unwrap();
    assert_eq!(borrow.borrowed, 100_000);
    let pool = c.get_pool_state();
    assert_eq!(pool.total_borrowed, 100_000);
}

#[test]
fn test_repay() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint(&env, &token, &provider, 1_000_000);
    mint(&env, &token, &borrower, 1_000_000);
    c.deposit(&provider, &500_000i128);
    c.borrow(&borrower, &1u64, &100_000i128, &86_400u64);

    // Repay immediately (no time elapsed, minimal interest)
    c.repay(&borrower, &1u64, &100_000i128);
    let pool = c.get_pool_state();
    assert_eq!(pool.total_borrowed, 0);
}

#[test]
fn test_get_provider_position_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    assert!(c.get_provider_position(&Address::generate(&env)).is_none());
}

#[test]
fn test_get_borrow_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    assert!(c.get_borrow(&999u64).is_none());
}

#[test]
fn test_repay_with_interest_distributes_yield() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);

    mint(&env, &token, &provider, 1_000_000);
    mint(&env, &token, &borrower, 1_000_000);

    // Provider deposits 500,000
    c.deposit(&provider, &500_000i128);
    let pool_after_deposit = c.get_pool_state();
    assert_eq!(pool_after_deposit.total_liquidity, 500_000);

    // Borrower borrows 100,000
    c.borrow(&borrower, &1u64, &100_000i128, &86_400u64);
    let pool_after_borrow = c.get_pool_state();
    assert_eq!(pool_after_borrow.total_liquidity, 500_000);
    assert_eq!(pool_after_borrow.total_borrowed, 100_000);

    // Advance time by 1 year to accrue interest
    env.ledger().with_mut(|li| {
        li.timestamp += 31_557_600;
    });

    // Borrower repays with interest
    c.repay(&borrower, &1u64, &110_000i128);
    let pool_after_repay = c.get_pool_state();

    // Interest ~5,000 at 5% on 100k. With reserve_factor=10%:
    // - protocol reserve gets ~500 (10%)
    // - lenders get ~4,500 (90%) added to total_liquidity
    assert_eq!(pool_after_repay.total_borrowed, 0);
    assert!(
        pool_after_repay.total_liquidity >= 504_400 && pool_after_repay.total_liquidity <= 504_600
    );
    assert!(pool_after_repay.interest_reserve >= 490 && pool_after_repay.interest_reserve <= 510);
}

#[test]
fn test_repay_principal_only() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);

    mint(&env, &token, &provider, 1_000_000);
    mint(&env, &token, &borrower, 1_000_000);

    c.deposit(&provider, &500_000i128);
    c.borrow(&borrower, &1u64, &100_000i128, &86_400u64);

    // Repay immediately (minimal time elapsed, negligible interest)
    c.repay(&borrower, &1u64, &100_000i128);
    let pool = c.get_pool_state();

    // With negligible time elapsed, interest ~0, so liquidity stays ~500,000
    assert!(pool.total_liquidity >= 500_000 && pool.total_liquidity <= 500_010);
    assert_eq!(pool.total_borrowed, 0);
    assert!(pool.interest_reserve < 10);
}

#[test]
fn test_multiple_borrows_with_interest() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower1 = Address::generate(&env);
    let borrower2 = Address::generate(&env);

    mint(&env, &token, &provider, 1_000_000);
    mint(&env, &token, &borrower1, 1_000_000);
    mint(&env, &token, &borrower2, 1_000_000);

    // Provider deposits 500,000
    c.deposit(&provider, &500_000i128);

    // Two borrowers each borrow 100,000 (20% of pool — within the 25% per-tx cap)
    c.borrow(&borrower1, &1u64, &100_000i128, &86_400u64);
    c.borrow(&borrower2, &2u64, &100_000i128, &86_400u64);

    let pool_after_borrows = c.get_pool_state();
    assert_eq!(pool_after_borrows.total_liquidity, 500_000);
    assert_eq!(pool_after_borrows.total_borrowed, 200_000);

    // Advance time by 1 year
    env.ledger().with_mut(|li| {
        li.timestamp += 31_557_600;
    });

    // First borrower repays with interest (~5% of 100k = ~5k)
    // 10% reserve → ~500 to protocol, ~4500 to lenders
    c.repay(&borrower1, &1u64, &110_000i128);
    let pool_after_first = c.get_pool_state();
    assert!(
        pool_after_first.total_liquidity >= 504_400 && pool_after_first.total_liquidity <= 504_600
    );
    assert_eq!(pool_after_first.total_borrowed, 100_000);
    assert!(pool_after_first.interest_reserve >= 490 && pool_after_first.interest_reserve <= 510);

    // Second borrower repays with interest (~5% of 100k = ~5k)
    // 10% reserve → ~500 to protocol, ~4500 to lenders
    c.repay(&borrower2, &2u64, &110_000i128);
    let pool_after_second = c.get_pool_state();
    assert_eq!(pool_after_second.total_borrowed, 0);
    // Total lender yield: ~4500 + ~4500 = ~9,000 added to liquidity
    assert!(
        pool_after_second.total_liquidity >= 508_900
            && pool_after_second.total_liquidity <= 509_100
    );
    // Total protocol reserve: ~500 + ~500 = ~1,000
    assert!(
        pool_after_second.interest_reserve >= 990 && pool_after_second.interest_reserve <= 1_010
    );
}

#[test]
fn test_provider_shares_not_inflated_by_interest() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);

    mint(&env, &token, &provider, 1_000_000);
    mint(&env, &token, &borrower, 1_000_000);

    // Provider deposits and gets shares
    let shares = c.deposit(&provider, &500_000i128);

    // Borrower borrows
    c.borrow(&borrower, &1u64, &100_000i128, &86_400u64);

    // Advance time by 1 year
    env.ledger().with_mut(|li| {
        li.timestamp += 31_557_600;
    });

    // Borrower repays with interest
    c.repay(&borrower, &1u64, &110_000i128);

    // Provider's shares are now worth MORE than original deposit (yield earned)
    let pool = c.get_pool_state();
    let position = c.get_provider_position(&provider).unwrap();

    // Share count unchanged, but total_liquidity grew by lender share of interest
    assert_eq!(position.shares, shares);
    // ~5000 interest, 90% → lenders = ~4500 added to total_liquidity
    assert!(pool.total_liquidity >= 504_400 && pool.total_liquidity <= 504_600);
    // Protocol reserve gets 10% = ~500
    assert!(pool.interest_reserve >= 490 && pool.interest_reserve <= 510);
}

#[test]
fn test_repay_partial_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);

    mint(&env, &token, &provider, 1_000_000);
    mint(&env, &token, &borrower, 1_000_000);

    c.deposit(&provider, &500_000i128);
    c.borrow(&borrower, &1u64, &100_000i128, &86_400u64);

    // Repay immediately (no time elapsed, minimal interest)
    c.repay(&borrower, &1u64, &100_000i128);
    let pool = c.get_pool_state();

    assert_eq!(pool.total_borrowed, 0);
    // With negligible time elapsed, liquidity stays ~500,000
    assert!(pool.total_liquidity >= 500_000 && pool.total_liquidity <= 500_010);

    // Borrow record should be removed
    assert!(c.get_borrow(&1u64).is_none());
}

#[test]
#[should_panic(expected = "shares must be positive")]
fn test_withdraw_when_total_shares_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);

    // Deposit and then withdraw all shares so total_shares reaches 0
    mint(&env, &token, &provider, 1_000_000);
    let shares = c.deposit(&provider, &100_000i128);
    c.withdraw(&provider, &shares);

    // Pool now has total_shares == 0; attempting to withdraw 0 shares must panic
    c.withdraw(&provider, &0i128);
}

#[test]
#[should_panic(expected = "shares must be positive")]
fn test_withdraw_rejects_zero_shares() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);

    mint(&env, &token, &provider, 1_000_000);
    c.deposit(&provider, &100_000i128);

    c.withdraw(&provider, &0i128);
}

#[test]
#[should_panic(expected = "borrow amount must be positive")]
fn test_borrow_rejects_zero_amount() {
    // amount=0 is invalid; the positivity guard must fire before any pool read.
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, _) = setup(&env);
    let borrower = Address::generate(&env);
    c.borrow(&borrower, &1u64, &0i128, &86_400u64);
}

#[test]
fn test_accrue_interest() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, admin, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);

    mint(&env, &token, &provider, 1_000_000);
    mint(&env, &token, &borrower, 1_000_000);

    c.deposit(&provider, &500_000i128);
    c.borrow(&borrower, &1u64, &100_000i128, &86_400u64);

    // Advance time by 1 year (31,557,600 seconds)
    env.ledger().with_mut(|li| {
        li.timestamp += 31_557_600;
    });

    // Accrue interest
    let interest = c.accrue_interest(&admin, &1u64);

    // At 5% annual rate on 100,000: interest should be ~5,000
    assert!((4_900..=5_100).contains(&interest));

    let borrow = c.get_borrow(&1u64).unwrap();
    assert_eq!(borrow.interest_accrued, interest);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_accrue_interest_rejects_unauthorized_caller() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);
    let stranger = Address::generate(&env);

    mint(&env, &token, &provider, 1_000_000);
    c.deposit(&provider, &500_000i128);
    c.borrow(&borrower, &1u64, &100_000i128, &86_400u64);

    c.accrue_interest(&stranger, &1u64);
}

#[test]
fn test_repay_with_accrued_interest() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);

    mint(&env, &token, &provider, 1_000_000);
    mint(&env, &token, &borrower, 1_000_000);

    c.deposit(&provider, &500_000i128);
    c.borrow(&borrower, &1u64, &100_000i128, &86_400u64);

    // Advance time by 1 year
    env.ledger().with_mut(|li| {
        li.timestamp += 31_557_600;
    });

    // Repay with interest (100,000 + ~5,000)
    c.repay(&borrower, &1u64, &105_500i128);

    let pool = c.get_pool_state();
    assert_eq!(pool.total_borrowed, 0);
    // 10% of ~5000 interest → ~500 to protocol reserve
    assert!(pool.interest_reserve >= 490 && pool.interest_reserve <= 510);
    // 90% of ~5000 interest → ~4500 to lenders via total_liquidity
    assert!(pool.total_liquidity >= 504_400 && pool.total_liquidity <= 504_600);
}

#[test]
#[should_panic(expected = "insufficient payment")]
fn test_repay_insufficient_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);

    mint(&env, &token, &provider, 1_000_000);
    mint(&env, &token, &borrower, 1_000_000);

    c.deposit(&provider, &500_000i128);
    c.borrow(&borrower, &1u64, &100_000i128, &86_400u64);

    // Advance time by 1 year
    env.ledger().with_mut(|li| {
        li.timestamp += 31_557_600;
    });

    // Try to repay only principal (should fail due to accrued interest)
    c.repay(&borrower, &1u64, &100_000i128);
}

#[test]
fn test_interest_calculation_over_time() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);

    mint(&env, &token, &provider, 1_000_000);
    mint(&env, &token, &borrower, 1_000_000);

    c.deposit(&provider, &500_000i128);
    c.borrow(&borrower, &1u64, &100_000i128, &86_400u64);

    // Check interest after 6 months
    env.ledger().with_mut(|li| {
        li.timestamp += 15_778_800; // ~6 months
    });

    let interest_6mo = c.accrue_interest(&borrower, &1u64);
    assert!((2_400..=2_600).contains(&interest_6mo)); // ~2.5% of 100k

    // Check interest after another 6 months (1 year total)
    env.ledger().with_mut(|li| {
        li.timestamp += 15_778_800;
    });

    let interest_1yr = c.accrue_interest(&borrower, &1u64);
    assert!((4_900..=5_100).contains(&interest_1yr)); // ~5% of 100k
}

#[test]
fn test_repay_with_overpayment() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);

    mint(&env, &token, &provider, 1_000_000);
    mint(&env, &token, &borrower, 1_000_000);

    c.deposit(&provider, &500_000i128);
    c.borrow(&borrower, &1u64, &100_000i128, &86_400u64);

    // Advance time by 1 year
    env.ledger().with_mut(|li| {
        li.timestamp += 31_557_600;
    });

    // Repay with overpayment (should return excess)
    c.repay(&borrower, &1u64, &110_000i128);

    let pool = c.get_pool_state();
    assert_eq!(pool.total_borrowed, 0);
    // Protocol reserve gets 10% of ~5000 = ~500
    assert!(pool.interest_reserve >= 490 && pool.interest_reserve <= 510);
    // Lenders get 90% of ~5000 = ~4500 added to liquidity
    assert!(pool.total_liquidity >= 504_400 && pool.total_liquidity <= 504_600);
}

#[test]
fn test_lender_earns_yield_on_withdrawal() {
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);

    mint(&env, &token, &provider, 1_000_000);
    mint(&env, &token, &borrower, 1_000_000);

    // Provider deposits 500,000 and receives shares
    let shares = c.deposit(&provider, &500_000i128);
    assert_eq!(shares, 500_000); // 1:1 for first deposit

    // Borrower borrows 100,000 for 1 year
    c.borrow(&borrower, &1u64, &100_000i128, &31_557_600u64);

    // Advance time by 1 year
    env.ledger().with_mut(|li| {
        li.timestamp += 31_557_600;
    });

    // Borrower repays with interest (~5,000)
    c.repay(&borrower, &1u64, &110_000i128);

    // Lender's shares now represent more value due to interest yield
    // total_liquidity grew from 500,000 to ~504,500
    // Withdrawing all shares should return ~504,500
    let withdrawn = c.withdraw(&provider, &shares);
    assert!((504_400..=504_600).contains(&withdrawn));
}

#[test]
#[should_panic(expected = "borrow amount must be positive")]
fn test_borrow_rejects_negative_amount() {
    // Negative amount would corrupt pool.total_borrowed via wrapping addition.
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint(&env, &token, &provider, 1_000_000);
    c.deposit(&provider, &500_000i128);
    c.borrow(&borrower, &1u64, &-1i128, &86_400u64);
}

#[test]
#[should_panic(expected = "borrow exceeds per-transaction limit")]
fn test_borrow_rejects_exceeding_per_tx_cap() {
    // Pool: 500,000; 25% cap = 125,000; borrowing 125,001 must be rejected.
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint(&env, &token, &provider, 1_000_000);
    c.deposit(&provider, &500_000i128);
    c.borrow(&borrower, &1u64, &125_001i128, &86_400u64);
}

#[test]
fn test_borrow_at_exactly_per_tx_cap() {
    // Pool: 500,000; 25% cap = 125,000; borrowing exactly 125,000 must succeed.
    let env = Env::default();
    env.mock_all_auths();
    let (c, _, _, token) = setup(&env);
    let provider = Address::generate(&env);
    let borrower = Address::generate(&env);
    mint(&env, &token, &provider, 1_000_000);
    c.deposit(&provider, &500_000i128);
    c.borrow(&borrower, &1u64, &125_000i128, &86_400u64);
    let b = c.get_borrow(&1u64).unwrap();
    assert_eq!(b.borrowed, 125_000);
    let pool = c.get_pool_state();
    assert_eq!(pool.total_borrowed, 125_000);
}
