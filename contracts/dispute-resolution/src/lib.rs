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
#[derive(Clone, PartialEq)]
pub enum AppealStatus {
    Pending,
    UnderReview,
    Upheld,
    Overturned,
    Dismissed,
}

#[contracttype]
#[derive(Clone)]
pub struct DisputeAppeal {
    pub appeal_id: u64,
    pub dispute_id: u64,
    pub appellant: Address,
    pub reason: String,
    pub evidence_hash: String,
    pub status: AppealStatus,
    pub filed_at: u64,
    pub resolved_at: Option<u64>,
    pub new_arbitrator: Option<Address>,
    pub original_outcome: DisputeOutcome,
    pub final_outcome: DisputeOutcome,
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
    AppealCounter,
    FilingFee,
    AppealFee,
    TokenAddress,
    Dispute(u64),
    Appeal(u64),
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
    pub fn initialize(env: Env, admin: Address, token: Address, filing_fee: i128, appeal_fee: i128) {
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
            .set(&DataKey::AppealFee, &appeal_fee);
        env.storage()
            .instance()
            .set(&DataKey::DisputeCounter, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::AppealCounter, &0u64);
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

        // Collect filing fee into escrow
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
            let token_client = token::Client::new(&env, &token_addr);
            token_client.transfer(&claimant, &env.current_contract_address(), &fee);
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

        dispute.outcome = outcome.clone();
        dispute.resolution_notes = notes;
        dispute.status = DisputeStatus::Resolved;
        dispute.resolved_at = Some(env.ledger().timestamp());

        // Distribute filing fee based on outcome
        let fee: i128 = env
            .storage()
            .instance()
            .get(&DataKey::FilingFee)
            .unwrap_or(0);
        if fee > 0 {
            let token_client = token::Client::new(&env, &dispute.token);
            match outcome {
                DisputeOutcome::Claimant => {
                    // Refund filing fee to claimant
                    token_client.transfer(
                        &env.current_contract_address(),
                        &dispute.claimant,
                        &fee,
                    );
                }
                _ => {
                    // Send filing fee to admin (treasury)
                    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
                    token_client.transfer(
                        &env.current_contract_address(),
                        &admin,
                        &fee,
                    );
                }
            }
        }

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

    pub fn appeal_dispute(
        env: Env,
        appellant: Address,
        dispute_id: u64,
        reason: String,
        evidence_hash: String,
    ) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        appellant.require_auth();

        let dispute: Dispute = env
            .storage()
            .persistent()
            .get(&DataKey::Dispute(dispute_id))
            .expect("dispute not found");

        if dispute.status != DisputeStatus::Resolved {
            panic!("can only appeal resolved disputes");
        }

        if appellant != dispute.claimant && appellant != dispute.respondent {
            panic!("only claimant or respondent can appeal");
        }

        // Collect appeal fee into escrow
        let appeal_fee: i128 = env
            .storage()
            .instance()
            .get(&DataKey::AppealFee)
            .unwrap_or(0);
        if appeal_fee > 0 {
            let token_addr: Address = env
                .storage()
                .instance()
                .get(&DataKey::TokenAddress)
                .unwrap();
            let token_client = token::Client::new(&env, &token_addr);
            token_client.transfer(&appellant, &env.current_contract_address(), &appeal_fee);
        }

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AppealCounter)
            .unwrap_or(0);
        let appeal_id = counter + 1;

        let appeal = DisputeAppeal {
            appeal_id,
            dispute_id,
            appellant: appellant.clone(),
            reason,
            evidence_hash,
            status: AppealStatus::Pending,
            filed_at: env.ledger().timestamp(),
            resolved_at: None,
            new_arbitrator: None,
            original_outcome: dispute.outcome.clone(),
            final_outcome: DisputeOutcome::Pending,
        };

        let _ttl_key = DataKey::Appeal(appeal_id);
        env.storage().persistent().set(&_ttl_key, &appeal);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::AppealCounter, &appeal_id);

        env.events().publish(
            (symbol_short!("dispute"), symbol_short!("appeal")),
            (appeal_id, dispute_id, appellant),
        );

        appeal_id
    }

    pub fn resolve_appeal(
        env: Env,
        admin: Address,
        appeal_id: u64,
        new_arbitrator: Address,
        final_outcome: DisputeOutcome,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let mut appeal: DisputeAppeal = env
            .storage()
            .persistent()
            .get(&DataKey::Appeal(appeal_id))
            .expect("appeal not found");

        if appeal.status != AppealStatus::Pending {
            panic!("appeal not pending");
        }

        appeal.status = AppealStatus::Upheld;
        appeal.new_arbitrator = Some(new_arbitrator);
        appeal.final_outcome = final_outcome.clone();
        appeal.resolved_at = Some(env.ledger().timestamp());

        // Update the original dispute with new outcome
        let mut dispute: Dispute = env
            .storage()
            .persistent()
            .get(&DataKey::Dispute(appeal.dispute_id))
            .expect("dispute not found");
        dispute.outcome = final_outcome;
        dispute.arbitrator = Some(new_arbitrator);

        let dispute_ttl_key = DataKey::Dispute(appeal.dispute_id);
        env.storage().persistent().set(&dispute_ttl_key, &dispute);
        env.storage().persistent().extend_ttl(
            &dispute_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let appeal_ttl_key = DataKey::Appeal(appeal_id);
        env.storage().persistent().set(&appeal_ttl_key, &appeal);
        env.storage().persistent().extend_ttl(
            &appeal_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("dispute"), symbol_short!("appeal_resolved")),
            appeal_id,
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

    pub fn get_appeal(env: Env, appeal_id: u64) -> Option<DisputeAppeal> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Appeal(appeal_id))
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
