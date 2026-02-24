//! PulsarTrack - Dispute Resolution (Soroban)
//! On-chain dispute resolution for PulsarTrack ecosystem participants on Stellar.

#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, String,
};

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum DisputeStatus {
    Filed,
    UnderReview,
    AwaitingEvidence,
    Deliberating,
    Resolved,
    Appealed,
    Closed,
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum DisputeOutcome {
    Pending,
    Claimant,
    Respondent,
    Split,
    NoAction,
}

#[contracttype]
#[derive(Clone)]
pub struct Dispute {
    pub dispute_id: u64,
    pub claimant: Address,
    pub respondent: Address,
    pub campaign_id: u64,
    pub claim_amount: i128,
    pub token: Address,
    pub description: String,
    pub evidence_hash: String, // IPFS hash of evidence
    pub status: DisputeStatus,
    pub outcome: DisputeOutcome,
    pub resolution_notes: String,
    pub filed_at: u64,
    pub resolved_at: Option<u64>,
    pub arbitrator: Option<Address>,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    ArbitratorPool,
    DisputeCounter,
    FilingFee,
    TokenAddress,
    Dispute(u64),
    ArbitratorApproved(Address),
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 34_560;
const PERSISTENT_BUMP_AMOUNT: u32 = 259_200;

#[contract]
pub struct DisputeResolutionContract;

#[contractimpl]
impl DisputeResolutionContract {
    pub fn initialize(env: Env, admin: Address, token: Address, filing_fee: i128) {
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
            .set(&DataKey::FilingFee, &filing_fee);
        env.storage()
            .instance()
            .set(&DataKey::DisputeCounter, &0u64);
    }

    pub fn authorize_arbitrator(env: Env, admin: Address, arbitrator: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        let _ttl_key = DataKey::ArbitratorApproved(arbitrator);
        env.storage().persistent().set(&_ttl_key, &true);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn file_dispute(
        env: Env,
        claimant: Address,
        respondent: Address,
        campaign_id: u64,
        claim_amount: i128,
        description: String,
        evidence_hash: String,
    ) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        claimant.require_auth();

        // Collect filing fee
        let fee: i128 = env
            .storage()
            .instance()
            .get(&DataKey::FilingFee)
            .unwrap_or(0);
        if fee > 0 {
            let token_addr: Address = env
                .storage()
                .instance()
                .get(&DataKey::TokenAddress)
                .unwrap();
            let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
            let token_client = token::Client::new(&env, &token_addr);
            token_client.transfer(&claimant, &admin, &fee);
        }

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::DisputeCounter)
            .unwrap_or(0);
        let dispute_id = counter + 1;

        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let dispute = Dispute {
            dispute_id,
            claimant: claimant.clone(),
            respondent,
            campaign_id,
            claim_amount,
            token: token_addr,
            description,
            evidence_hash,
            status: DisputeStatus::Filed,
            outcome: DisputeOutcome::Pending,
            resolution_notes: String::from_str(&env, ""),
            filed_at: env.ledger().timestamp(),
            resolved_at: None,
            arbitrator: None,
        };

        let _ttl_key = DataKey::Dispute(dispute_id);
        env.storage().persistent().set(&_ttl_key, &dispute);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::DisputeCounter, &dispute_id);

        env.events().publish(
            (symbol_short!("dispute"), symbol_short!("filed")),
            (dispute_id, claimant),
        );

        dispute_id
    }

    pub fn assign_arbitrator(env: Env, admin: Address, dispute_id: u64, arbitrator: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let is_authorized: bool = env
            .storage()
            .persistent()
            .get(&DataKey::ArbitratorApproved(arbitrator.clone()))
            .unwrap_or(false);

        if !is_authorized {
            panic!("arbitrator not authorized");
        }

        let mut dispute: Dispute = env
            .storage()
            .persistent()
            .get(&DataKey::Dispute(dispute_id))
            .expect("dispute not found");

        dispute.arbitrator = Some(arbitrator);
        dispute.status = DisputeStatus::UnderReview;
        let _ttl_key = DataKey::Dispute(dispute_id);
        env.storage().persistent().set(&_ttl_key, &dispute);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn resolve_dispute(
        env: Env,
        arbitrator: Address,
        dispute_id: u64,
        outcome: DisputeOutcome,
        notes: String,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        arbitrator.require_auth();

        let mut dispute: Dispute = env
            .storage()
            .persistent()
            .get(&DataKey::Dispute(dispute_id))
            .expect("dispute not found");

        if let Some(ref assigned) = dispute.arbitrator {
            if *assigned != arbitrator {
                panic!("not assigned arbitrator");
            }
        } else {
            panic!("not assigned arbitrator");
        }

        dispute.outcome = outcome;
        dispute.resolution_notes = notes;
        dispute.status = DisputeStatus::Resolved;
        dispute.resolved_at = Some(env.ledger().timestamp());

        let _ttl_key = DataKey::Dispute(dispute_id);
        env.storage().persistent().set(&_ttl_key, &dispute);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("dispute"), symbol_short!("resolved")),
            dispute_id,
        );
    }

    pub fn get_dispute(env: Env, dispute_id: u64) -> Option<Dispute> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Dispute(dispute_id))
    }

    pub fn get_dispute_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .instance()
            .get(&DataKey::DisputeCounter)
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
