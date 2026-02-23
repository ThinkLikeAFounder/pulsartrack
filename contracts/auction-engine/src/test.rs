#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, String,
};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn deploy_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone()).address()
}

fn mint(env: &Env, token_addr: &Address, to: &Address, amount: i128) {
    let sac = StellarAssetClient::new(env, token_addr);
    sac.mint(to, &amount);
}

fn setup(env: &Env) -> (AuctionEngineContractClient, Address, Address, Address) {
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_addr = deploy_token(env, &token_admin);

    let contract_id = env.register_contract(None, AuctionEngineContract);
    let client = AuctionEngineContractClient::new(env, &contract_id);
    client.initialize(&admin, &token_addr);

    (client, admin, token_admin, token_addr)
}

fn slot(env: &Env) -> String {
    String::from_str(env, "banner-300x250")
}

// ─── initialize ──────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let token = deploy_token(&env, &admin);

    let contract_id = env.register_contract(None, AuctionEngineContract);
    let client = AuctionEngineContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let token = deploy_token(&env, &admin);

    let contract_id = env.register_contract(None, AuctionEngineContract);
    let client = AuctionEngineContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token);
    client.initialize(&admin, &token);
}

#[test]
#[should_panic]
fn test_initialize_non_admin_fails() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let token = deploy_token(&env, &admin);

    let contract_id = env.register_contract(None, AuctionEngineContract);
    let client = AuctionEngineContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token);
}

// ─── create_auction ───────────────────────────────────────────────────────────

#[test]
fn test_create_auction() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let publisher = Address::generate(&env);

    let auction_id = client.create_auction(
        &publisher,
        &slot(&env),
        &1_000i128,   // floor
        &5_000i128,   // reserve
        &3600u64,     // 1 hour
    );

    assert_eq!(auction_id, 1);

    let auction = client.get_auction(&auction_id).unwrap();
    assert_eq!(auction.publisher, publisher);
    assert_eq!(auction.floor_price, 1_000);
    assert_eq!(auction.reserve_price, 5_000);
    assert!(matches!(auction.status, AuctionStatus::Open));
    assert_eq!(auction.bid_count, 0);
}

// ─── place_bid ────────────────────────────────────────────────────────────────

#[test]
fn test_place_bid() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let publisher = Address::generate(&env);
    let bidder = Address::generate(&env);

    let auction_id = client.create_auction(
        &publisher, &slot(&env), &1_000i128, &5_000i128, &3600u64,
    );

    client.place_bid(&bidder, &auction_id, &2_000i128, &42u64);

    let auction = client.get_auction(&auction_id).unwrap();
    assert_eq!(auction.bid_count, 1);
    assert_eq!(auction.winning_bid, Some(2_000));
    assert_eq!(auction.winner, Some(bidder.clone()));

    assert_eq!(client.get_bid_count(&auction_id), 1);
    assert_eq!(client.get_highest_bid(&auction_id), Some(2_000));
}

#[test]
fn test_multiple_bids_highest_wins() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let publisher = Address::generate(&env);
    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);

    let auction_id = client.create_auction(
        &publisher, &slot(&env), &1_000i128, &5_000i128, &3600u64,
    );

    client.place_bid(&bidder1, &auction_id, &2_000i128, &1u64);
    client.place_bid(&bidder2, &auction_id, &4_000i128, &2u64);

    let auction = client.get_auction(&auction_id).unwrap();
    assert_eq!(auction.winner, Some(bidder2));
    assert_eq!(auction.winning_bid, Some(4_000));
    assert_eq!(auction.bid_count, 2);
    assert_eq!(client.get_highest_bid(&auction_id), Some(4_000));
}

#[test]
fn test_bid_stored_by_index() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let publisher = Address::generate(&env);
    let bidder = Address::generate(&env);

    let auction_id = client.create_auction(
        &publisher, &slot(&env), &500i128, &2_000i128, &3600u64,
    );

    client.place_bid(&bidder, &auction_id, &1_000i128, &99u64);

    let bid = client.get_bid(&auction_id, &0u32).unwrap();
    assert_eq!(bid.bidder, bidder);
    assert_eq!(bid.amount, 1_000);
    assert_eq!(bid.campaign_id, 99);
}

// ─── bid error paths ─────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "bid below floor price")]
fn test_bid_below_floor_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let publisher = Address::generate(&env);
    let bidder = Address::generate(&env);

    let auction_id = client.create_auction(
        &publisher, &slot(&env), &1_000i128, &5_000i128, &3600u64,
    );

    client.place_bid(&bidder, &auction_id, &500i128, &1u64); // below 1_000
}

#[test]
#[should_panic(expected = "bid too low")]
fn test_bid_not_higher_than_current_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let publisher = Address::generate(&env);
    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);

    let auction_id = client.create_auction(
        &publisher, &slot(&env), &1_000i128, &5_000i128, &3600u64,
    );

    client.place_bid(&bidder1, &auction_id, &3_000i128, &1u64);
    client.place_bid(&bidder2, &auction_id, &2_000i128, &2u64); // lower than current best
}

#[test]
#[should_panic(expected = "auction ended")]
fn test_bid_after_auction_ended() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let publisher = Address::generate(&env);
    let bidder = Address::generate(&env);

    let auction_id = client.create_auction(
        &publisher, &slot(&env), &1_000i128, &5_000i128, &100u64, // 100 second duration
    );

    // advance past end_time
    env.ledger().with_mut(|li| {
        li.timestamp = 200;
    });

    client.place_bid(&bidder, &auction_id, &2_000i128, &1u64);
}

// ─── settle_auction ──────────────────────────────────────────────────────────

#[test]
fn test_settle_auction_with_winner() {
    let env = Env::default();
    // winner is a non-root auth signer when settle_auction is called by publisher
    env.mock_all_auths_allowing_non_root_auth();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);

    let contract_id = env.register_contract(None, AuctionEngineContract);
    let client = AuctionEngineContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr);

    let publisher = Address::generate(&env);
    let bidder = Address::generate(&env);
    mint(&env, &token_addr, &bidder, 100_000);

    let auction_id = client.create_auction(
        &publisher, &slot(&env), &1_000i128, &2_000i128, &100u64,
    );

    client.place_bid(&bidder, &auction_id, &3_000i128, &1u64); // above reserve

    // advance past end_time
    env.ledger().with_mut(|li| {
        li.timestamp = 200;
    });

    client.settle_auction(&publisher, &auction_id);

    let auction = client.get_auction(&auction_id).unwrap();
    assert!(matches!(auction.status, AuctionStatus::Settled));

    let tc = TokenClient::new(&env, &token_addr);
    assert_eq!(tc.balance(&publisher), 3_000);
    assert_eq!(tc.balance(&bidder), 97_000);
}

#[test]
fn test_settle_auction_below_reserve_cancelled() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);

    let contract_id = env.register_contract(None, AuctionEngineContract);
    let client = AuctionEngineContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr);

    let publisher = Address::generate(&env);
    let bidder = Address::generate(&env);
    mint(&env, &token_addr, &bidder, 100_000);

    let auction_id = client.create_auction(
        &publisher, &slot(&env),
        &1_000i128,   // floor
        &10_000i128,  // reserve (high)
        &100u64,
    );

    client.place_bid(&bidder, &auction_id, &1_500i128, &1u64); // above floor, below reserve

    env.ledger().with_mut(|li| {
        li.timestamp = 200;
    });

    client.settle_auction(&publisher, &auction_id);

    let auction = client.get_auction(&auction_id).unwrap();
    assert!(matches!(auction.status, AuctionStatus::Cancelled));

    // no transfer should have happened
    let tc = TokenClient::new(&env, &token_addr);
    assert_eq!(tc.balance(&bidder), 100_000);
    assert_eq!(tc.balance(&publisher), 0);
}

#[test]
fn test_settle_auction_no_bids_cancelled() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let publisher = Address::generate(&env);

    let auction_id = client.create_auction(
        &publisher, &slot(&env), &1_000i128, &5_000i128, &100u64,
    );

    env.ledger().with_mut(|li| {
        li.timestamp = 200;
    });

    client.settle_auction(&publisher, &auction_id);

    let auction = client.get_auction(&auction_id).unwrap();
    assert!(matches!(auction.status, AuctionStatus::Cancelled));
}

#[test]
#[should_panic(expected = "auction still running")]
fn test_settle_auction_still_running() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let publisher = Address::generate(&env);
    let bidder = Address::generate(&env);

    let auction_id = client.create_auction(
        &publisher, &slot(&env), &1_000i128, &5_000i128, &3600u64,
    );

    client.place_bid(&bidder, &auction_id, &2_000i128, &1u64);
    // time has NOT advanced → still running
    client.settle_auction(&publisher, &auction_id);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_settle_auction_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);
    let publisher = Address::generate(&env);
    let stranger = Address::generate(&env);

    let auction_id = client.create_auction(
        &publisher, &slot(&env), &1_000i128, &5_000i128, &100u64,
    );

    env.ledger().with_mut(|li| {
        li.timestamp = 200;
    });

    client.settle_auction(&stranger, &auction_id); // not publisher or admin
}

// ─── admin can force-settle before end_time ──────────────────────────────────

#[test]
fn test_admin_can_settle_before_end_time() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = deploy_token(&env, &token_admin);

    let contract_id = env.register_contract(None, AuctionEngineContract);
    let client = AuctionEngineContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr);

    let publisher = Address::generate(&env);
    let bidder = Address::generate(&env);
    mint(&env, &token_addr, &bidder, 100_000);

    let auction_id = client.create_auction(
        &publisher, &slot(&env), &1_000i128, &2_000i128, &9999u64,
    );

    client.place_bid(&bidder, &auction_id, &5_000i128, &1u64);

    // admin can settle even though auction is still running
    client.settle_auction(&admin, &auction_id);

    let auction = client.get_auction(&auction_id).unwrap();
    assert!(matches!(auction.status, AuctionStatus::Settled));
}

// ─── non-existent auction ────────────────────────────────────────────────────

#[test]
fn test_get_auction_nonexistent_returns_none() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, _) = setup(&env);

    assert!(client.get_auction(&999u64).is_none());
    assert_eq!(client.get_bid_count(&999u64), 0);
    assert!(client.get_highest_bid(&999u64).is_none());
}
