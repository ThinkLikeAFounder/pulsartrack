//! PulsarTrack - Dispute Resolution (Soroban)
//! On-chain dispute resolution for PulsarTrack ecosystem participants on Stellar.

#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, IntoVal, String,
    Symbol, Vec as SdkVec,
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
    EscrowContract,
    Dispute(u64),
    DisputeEscrow(u64),
    ArbitratorApproved(Address),
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;

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

    pub fn revoke_arbitrator(env: Env, admin: Address, arbitrator: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        env.storage()
            .persistent()
            .remove(&DataKey::ArbitratorApproved(arbitrator));
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
        if claim_amount <= 0 {
            panic!("invalid claim amount");
        }
        if claimant == respondent {
            panic!("claimant and respondent cannot be the same address");
        }

        let fee: i128 = env
            .storage()
            .instance()
            .get(&DataKey::FilingFee)
            .unwrap_or(0);
        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::DisputeCounter)
            .unwrap_or(0);
        let dispute_id = counter + 1;

        let dispute = Dispute {
            dispute_id,
            claimant: claimant.clone(),
            respondent,
            campaign_id,
            claim_amount,
            token: token_addr.clone(),
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

        let token_client = token::Client::new(&env, &token_addr);
        if fee > 0 {
            token_client.transfer(&claimant, &env.current_contract_address(), &fee);
        }
        // Lock claim funds in this contract until dispute settlement.
        token_client.transfer(&claimant, &env.current_contract_address(), &claim_amount);

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

        if dispute.status == DisputeStatus::Resolved || dispute.status == DisputeStatus::Closed {
            panic!("cannot assign arbitrator to a finalized dispute");
        }

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

        match dispute.status {
            DisputeStatus::Resolved | DisputeStatus::Closed | DisputeStatus::Appealed => {
                panic!("dispute cannot be resolved in its current status");
            }
            _ => {}
        }

        // Reject Pending as a resolution outcome — it is only valid as an initial
        // filing state (#747). Keep the arm in the match below as unreachable!()
        // so Rust's exhaustiveness checker is satisfied.
        if outcome == DisputeOutcome::Pending {
            panic!("Pending is not a valid resolution outcome");
        }

        let (claimant_amount, respondent_amount) = match &outcome {
            DisputeOutcome::Claimant => (dispute.claim_amount, 0),
            DisputeOutcome::Respondent => (0, dispute.claim_amount),
            DisputeOutcome::Split => {
                let claimant_part = dispute.claim_amount / 2;
                (claimant_part, dispute.claim_amount - claimant_part)
            }
            DisputeOutcome::NoAction => (0, 0),
            DisputeOutcome::Pending => unreachable!("rejected above"),
        };

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

        let used_escrow = if claimant_amount > 0 || respondent_amount > 0 {
            Self::try_settle_linked_escrow(
                &env,
                dispute_id,
                &dispute,
                claimant_amount,
                respondent_amount,
            )
        } else {
            false
        };

        if !used_escrow {
            let token_client = token::Client::new(&env, &dispute.token);
            if claimant_amount > 0 {
                token_client.transfer(
                    &env.current_contract_address(),
                    &dispute.claimant,
                    &claimant_amount,
                );
            }
            if respondent_amount > 0 {
                token_client.transfer(
                    &env.current_contract_address(),
                    &dispute.respondent,
                    &respondent_amount,
                );
            }
        }

        // Return the locked claim funds to the claimant whenever the payout path
        // above did not disburse them (#817). file_dispute locks claim_amount in
        // this contract, so an outcome that pays out nothing from it -- NoAction --
        // must hand it back, as must the escrow path, which settles the payout from
        // escrow balances and leaves the locked claim funds untouched here.
        let refund_claim = dispute.outcome == DisputeOutcome::NoAction || used_escrow;
        if refund_claim && dispute.claim_amount > 0 {
            let token_client = token::Client::new(&env, &dispute.token);
            token_client.transfer(
                &env.current_contract_address(),
                &dispute.claimant,
                &dispute.claim_amount,
            );
        }

        let fee: i128 = env
            .storage()
            .instance()
            .get(&DataKey::FilingFee)
            .unwrap_or(0);
        let fee_to_claimant = matches!(
            dispute.outcome,
            DisputeOutcome::Claimant | DisputeOutcome::Split | DisputeOutcome::NoAction
        );
        if fee > 0 && fee_to_claimant {
            let token_client = token::Client::new(&env, &dispute.token);
            token_client.transfer(&env.current_contract_address(), &dispute.claimant, &fee);
        }

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

    pub fn set_escrow_contract(env: Env, admin: Address, escrow_contract: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        env.storage()
            .instance()
            .set(&DataKey::EscrowContract, &escrow_contract);
    }

    pub fn link_dispute_escrow(env: Env, admin: Address, dispute_id: u64, escrow_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        if !env
            .storage()
            .persistent()
            .has(&DataKey::Dispute(dispute_id))
        {
            panic!("dispute not found");
        }
        let _ttl_key = DataKey::DisputeEscrow(dispute_id);
        env.storage().persistent().set(&_ttl_key, &escrow_id);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
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

    fn try_settle_linked_escrow(
        env: &Env,
        dispute_id: u64,
        dispute: &Dispute,
        claimant_amount: i128,
        respondent_amount: i128,
    ) -> bool {
        let escrow_contract: Address =
            if let Some(addr) = env.storage().instance().get(&DataKey::EscrowContract) {
                addr
            } else {
                return false;
            };
        let escrow_id: u64 = if let Some(id) = env
            .storage()
            .persistent()
            .get(&DataKey::DisputeEscrow(dispute_id))
        {
            id
        } else {
            return false;
        };

        env.invoke_contract::<()>(
            &escrow_contract,
            &Symbol::new(env, "settle_dispute"),
            SdkVec::from_array(
                env,
                [
                    env.current_contract_address().into_val(env),
                    escrow_id.into_val(env),
                    dispute.claimant.clone().into_val(env),
                    dispute.respondent.clone().into_val(env),
                    claimant_amount.into_val(env),
                    respondent_amount.into_val(env),
                ],
            ),
        );
        true
    }
}

mod test;
