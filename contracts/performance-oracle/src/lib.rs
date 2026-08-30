//! PulsarTrack - Performance Oracle (Soroban)
//! Validates and attests to campaign performance metrics on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env};

#[contracttype]
#[derive(Clone)]
pub struct PerformanceAttestation {
    pub campaign_id: u64,
    pub attester: Address,
    pub impressions_verified: u64,
    pub clicks_verified: u64,
    pub fraud_rate: u32,       // basis points
    pub quality_score: u32,    // 0-100
    pub data_hash: BytesN<32>, // hash of raw performance data
    pub attested_at: u64,
    pub ledger_sequence: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct OracleConsensus {
    pub campaign_id: u64,
    pub total_attesters: u32,
    pub avg_impressions: u64,
    pub avg_clicks: u64,
    pub avg_fraud_rate: u32,
    pub avg_quality_score: u32,
    pub consensus_reached: bool,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    MinAttesters,
    ConsensusThresholdPct,
    Attester(Address),
    Attestation(u64, Address),       // campaign_id, attester
    AttestationCount(u64),           // campaign_id
    Consensus(u64),                  // campaign_id
    ConsensusFinalized(u64),         // campaign_id
    CampaignAttesterIndex(u64, u32), // campaign_id, index -> Address
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 17_280;
const INSTANCE_BUMP_AMOUNT: u32 = 86_400;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 120_960;
const PERSISTENT_BUMP_AMOUNT: u32 = 1_051_200;
const MAX_CONSENSUS_ATTESTERS: u32 = 50;
const MAX_DEVIATION_BPS: u32 = 5000; // 50% maximum deviation (5000 basis points = 50%)

#[contract]
pub struct PerformanceOracleContract;

#[contractimpl]
impl PerformanceOracleContract {
    pub fn initialize(env: Env, admin: Address, min_attesters: u32) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::MinAttesters, &min_attesters);
        env.storage()
            .instance()
            .set(&DataKey::ConsensusThresholdPct, &67u32); // 2/3 majority
    }

    pub fn authorize_attester(env: Env, admin: Address, attester: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }
        let _ttl_key = DataKey::Attester(attester);
        env.storage().persistent().set(&_ttl_key, &true);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn revoke_attester(env: Env, admin: Address, attester: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let _ttl_key = DataKey::Attester(attester);
        env.storage().persistent().remove(&_ttl_key);
    }

    pub fn submit_attestation(
        env: Env,
        attester: Address,
        campaign_id: u64,
        impressions: u64,
        clicks: u64,
        fraud_rate: u32,
        quality_score: u32,
        data_hash: BytesN<32>,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        attester.require_auth();

        let is_auth: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Attester(attester.clone()))
            .unwrap_or(false);

        if !is_auth {
            panic!("not authorized attester");
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::ConsensusFinalized(campaign_id))
        {
            panic!("consensus already finalized");
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::Attestation(campaign_id, attester.clone()))
        {
            panic!("already attested");
        }

        if quality_score > 100 {
            panic!("quality_score must be 0-100");
        }
        if fraud_rate > 10_000 {
            panic!("fraud_rate must be 0-10000 basis points");
        }
        if clicks > impressions {
            panic!("clicks cannot exceed impressions");
        }

        // Enforce hard cap on attesters to prevent exceeding Soroban CPU limits
        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::AttestationCount(campaign_id))
            .unwrap_or(0);

        if count >= MAX_CONSENSUS_ATTESTERS {
            panic!("attester limit reached");
        }

        let attestation = PerformanceAttestation {
            campaign_id,
            attester: attester.clone(),
            impressions_verified: impressions,
            clicks_verified: clicks,
            fraud_rate,
            quality_score,
            data_hash,
            attested_at: env.ledger().timestamp(),
            ledger_sequence: env.ledger().sequence(),
        };

        let _ttl_key = DataKey::Attestation(campaign_id, attester.clone());
        env.storage().persistent().set(&_ttl_key, &attestation);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::AttestationCount(campaign_id))
            .unwrap_or(0);

        // Store attester address in indexed list for consensus calculation
        let _ttl_key = DataKey::CampaignAttesterIndex(campaign_id, count);
        env.storage().persistent().set(&_ttl_key, &attester);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let _ttl_key = DataKey::AttestationCount(campaign_id);
        env.storage().persistent().set(&_ttl_key, &(count + 1));
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        env.events().publish(
            (symbol_short!("oracle"), symbol_short!("attested")),
            campaign_id,
        );
    }

    pub fn get_attestation(
        env: Env,
        campaign_id: u64,
        attester: Address,
    ) -> Option<PerformanceAttestation> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Attestation(campaign_id, attester))
    }

    pub fn get_consensus(env: Env, campaign_id: u64) -> Option<OracleConsensus> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Consensus(campaign_id))
    }

    pub fn get_attestation_count(env: Env, campaign_id: u64) -> u32 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::AttestationCount(campaign_id))
            .unwrap_or(0)
    }

    pub fn finalize_consensus(env: Env, admin: Address, campaign_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::ConsensusFinalized(campaign_id))
        {
            panic!("consensus already finalized");
        }

        if !Self::_try_build_consensus(&env, campaign_id) {
            panic!("consensus not ready");
        }

        let _ttl_key = DataKey::ConsensusFinalized(campaign_id);
        env.storage().persistent().set(&_ttl_key, &true);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    fn _try_build_consensus(env: &Env, campaign_id: u64) -> bool {
        let min_attesters: u32 = env
            .storage()
            .instance()
            .get(&DataKey::MinAttesters)
            .unwrap_or(3);

        if env
            .storage()
            .persistent()
            .has(&DataKey::ConsensusFinalized(campaign_id))
        {
            return false;
        }

        let total_attesters: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::AttestationCount(campaign_id))
            .unwrap_or(0);

        if total_attesters == 0 || total_attesters < min_attesters {
            return false;
        }

        // Compute actual averages by reading all attestations
        let mut sum_impressions: u64 = 0;
        let mut sum_clicks: u64 = 0;
        let mut sum_fraud_rate: u64 = 0;
        let mut sum_quality_score: u64 = 0;

        // Track min/max for variance check
        let mut min_fraud_rate: u32 = u32::MAX;
        let mut max_fraud_rate: u32 = 0;
        let mut min_quality_score: u32 = u32::MAX;
        let mut max_quality_score: u32 = 0;

        for i in 0..total_attesters {
            let attester: Address = env
                .storage()
                .persistent()
                .get(&DataKey::CampaignAttesterIndex(campaign_id, i))
                .expect("attester index not found");

            let attestation: PerformanceAttestation = env
                .storage()
                .persistent()
                .get(&DataKey::Attestation(campaign_id, attester))
                .expect("attestation not found");

            sum_impressions = sum_impressions.saturating_add(attestation.impressions_verified);
            sum_clicks = sum_clicks.saturating_add(attestation.clicks_verified);
            sum_fraud_rate = sum_fraud_rate.saturating_add(attestation.fraud_rate as u64);
            sum_quality_score = sum_quality_score.saturating_add(attestation.quality_score as u64);

            // Track min/max for variance check
            if attestation.fraud_rate < min_fraud_rate {
                min_fraud_rate = attestation.fraud_rate;
            }
            if attestation.fraud_rate > max_fraud_rate {
                max_fraud_rate = attestation.fraud_rate;
            }
            if attestation.quality_score < min_quality_score {
                min_quality_score = attestation.quality_score;
            }
            if attestation.quality_score > max_quality_score {
                max_quality_score = attestation.quality_score;
            }
        }

        // Calculate averages
        let total_attesters_u64 = total_attesters as u64;
        let avg_impressions = sum_impressions / total_attesters_u64;
        let avg_clicks = sum_clicks / total_attesters_u64;
        let avg_fraud_rate = (sum_fraud_rate / total_attesters_u64).min(u32::MAX as u64) as u32;
        let avg_quality_score =
            (sum_quality_score / total_attesters_u64).min(u32::MAX as u64) as u32;

        // Variance check: reject if values are too divergent
        // Using median-absolute-deviation style check: max - min > tolerance
        let fraud_deviation_bps = if avg_fraud_rate > 0 {
            ((max_fraud_rate - min_fraud_rate) as u64 * 10000 / avg_fraud_rate as u64) as u32
        } else {
            0
        };

        let quality_deviation_bps = if avg_quality_score > 0 {
            ((max_quality_score - min_quality_score) as u64 * 10000 / avg_quality_score as u64)
                as u32
        } else {
            0
        };

        if fraud_deviation_bps > MAX_DEVIATION_BPS {
            // Log the variance issue but don't panic - just don't build consensus
            env.events().publish(
                (symbol_short!("oracle"), symbol_short!("variance")),
                (campaign_id, fraud_deviation_bps),
            );
            return false;
        }

        if quality_deviation_bps > MAX_DEVIATION_BPS {
            env.events().publish(
                (symbol_short!("oracle"), symbol_short!("variance")),
                (campaign_id, quality_deviation_bps),
            );
            return false;
        }

        let consensus = OracleConsensus {
            campaign_id,
            total_attesters,
            avg_impressions,
            avg_clicks,
            avg_fraud_rate,
            avg_quality_score,
            consensus_reached: true,
            last_updated: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::Consensus(campaign_id);
        env.storage().persistent().set(&_ttl_key, &consensus);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        true
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
