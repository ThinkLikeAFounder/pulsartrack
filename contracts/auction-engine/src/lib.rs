//! PulsarTrack - Auction Engine (Soroban)
//! Real-time bidding (RTB) auction system for ad impressions on Stellar.
//!
//! Events:
//! - ("auction", "created"): [auction_id: u64, publisher: Address]
//! - ("bid", "placed"): [auction_id: u64, bidder: Address, amount: i128]
//! - ("auction", "settle"): [auction_id: u64, winner: Option<Address>, amount: Option<i128>]

#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, String,
};

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum AuctionStatus {
    Open,
    Closed,
    Settled,
    Cancelled,
}

#[contracttype]
#[derive(Clone)]
pub struct Auction {
    pub auction_id: u64,
    pub publisher: Address,
    pub impression_slot: String,
    pub floor_price: i128,
    pub reserve_price: i128,
    pub start_time: u64,
    pub end_time: u64,
    pub status: AuctionStatus,
    pub winning_bid: Option<i128>,
    pub winner: Option<Address>,
    pub bid_count: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct Bid {
    pub bidder: Address,
    pub amount: i128,
    pub campaign_id: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    TokenAddress,
    AuctionCounter,
    Auction(u64),
    BidCount(u64),
    Bid(u64, u32), // auction_id, bid_index
    HighestBid(u64),
    BidderBid(u64, Address), // auction_id, bidder
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 17_280;
const PERSISTENT_BUMP_AMOUNT: u32 = 86_400;

#[contract]
pub struct AuctionEngineContract;

#[contractimpl]
impl AuctionEngineContract {
    pub fn initialize(env: Env, admin: Address, token: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TokenAddress, &token);
        env.storage()
            .instance()
            .set(&DataKey::AuctionCounter, &0u64);
    }

    pub fn create_auction(
        env: Env,
        publisher: Address,
        impression_slot: String,
        floor_price: i128,
        reserve_price: i128,
        duration_secs: u64,
    ) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        publisher.require_auth();

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AuctionCounter)
            .unwrap_or(0);
        let auction_id = counter + 1;

        let now = env.ledger().timestamp();
        let auction = Auction {
            auction_id,
            publisher: publisher.clone(),
            impression_slot,
            floor_price,
            reserve_price,
            start_time: now,
            end_time: now + duration_secs,
            status: AuctionStatus::Open,
            winning_bid: None,
            winner: None,
            bid_count: 0,
        };

        let _ttl_key = DataKey::Auction(auction_id);
        env.storage().persistent().set(&_ttl_key, &auction);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::AuctionCounter, &auction_id);

        env.events().publish(
            (symbol_short!("auction"), symbol_short!("created")),
            (auction_id, publisher),
        );

        auction_id
    }

    pub fn place_bid(env: Env, bidder: Address, auction_id: u64, amount: i128, campaign_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        bidder.require_auth();

        let mut auction: Auction = env
            .storage()
            .persistent()
            .get(&DataKey::Auction(auction_id))
            .expect("auction not found");

        if auction.status != AuctionStatus::Open {
            panic!("auction not open");
        }

        let now = env.ledger().timestamp();
        if now > auction.end_time {
            panic!("auction ended");
        }

        if amount < auction.floor_price {
            panic!("bid below floor price");
        }

        // Check if higher than current best
        let current_high: Option<i128> = env
            .storage()
            .persistent()
            .get(&DataKey::HighestBid(auction_id));
        if let Some(high) = current_high {
            if amount <= high {
                panic!("bid too low");
            }
        }

        let bid = Bid {
            bidder: bidder.clone(),
            amount,
            campaign_id,
            timestamp: now,
        };

        let bid_count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::BidCount(auction_id))
            .unwrap_or(0);
        let _ttl_key = DataKey::Bid(auction_id, bid_count);
        env.storage().persistent().set(&_ttl_key, &bid);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        let _ttl_key = DataKey::BidCount(auction_id);
        env.storage().persistent().set(&_ttl_key, &(bid_count + 1));
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        let _ttl_key = DataKey::BidderBid(auction_id, bidder.clone());
        env.storage().persistent().set(&_ttl_key, &amount);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        let _ttl_key = DataKey::HighestBid(auction_id);
        env.storage().persistent().set(&_ttl_key, &amount);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        auction.bid_count += 1;
        auction.winning_bid = Some(amount);
        auction.winner = Some(bidder.clone());
        let _ttl_key = DataKey::Auction(auction_id);
        env.storage().persistent().set(&_ttl_key, &auction);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("bid"), symbol_short!("placed")),
            (auction_id, bidder, amount),
        );
    }

    pub fn settle_auction(env: Env, caller: Address, auction_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let mut auction: Auction = env
            .storage()
            .persistent()
            .get(&DataKey::Auction(auction_id))
            .expect("auction not found");

        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if caller != auction.publisher && caller != admin {
            panic!("unauthorized");
        }

        let now = env.ledger().timestamp();
        if now < auction.end_time && caller != admin {
            panic!("auction still running");
        }

        auction.status = if auction.winning_bid.is_some() {
            let winning = auction.winning_bid.unwrap();
            if winning >= auction.reserve_price {
                // Transfer payment from winner to publisher
                let token_addr: Address = env
                    .storage()
                    .instance()
                    .get(&DataKey::TokenAddress)
                    .unwrap();
                let token_client = token::Client::new(&env, &token_addr);
                if let Some(winner) = auction.winner.clone() {
                    token_client.transfer(&winner, &auction.publisher, &winning);
                }
                AuctionStatus::Settled
            } else {
                AuctionStatus::Cancelled
            }
        } else {
            AuctionStatus::Cancelled
        };

        let _ttl_key = DataKey::Auction(auction_id);
        env.storage().persistent().set(&_ttl_key, &auction);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("auction"), symbol_short!("settle")),
            (auction_id, auction.winner, auction.winning_bid),
        );
    }

    pub fn get_auction(env: Env, auction_id: u64) -> Option<Auction> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Auction(auction_id))
    }

    pub fn get_bid(env: Env, auction_id: u64, index: u32) -> Option<Bid> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Bid(auction_id, index))
    }

    pub fn get_bid_count(env: Env, auction_id: u64) -> u32 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::BidCount(auction_id))
            .unwrap_or(0)
    }

    pub fn get_highest_bid(env: Env, auction_id: u64) -> Option<i128> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::HighestBid(auction_id))
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
