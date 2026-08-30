//! PulsarTrack - Anomaly Detector (Soroban)
//! On-chain anomaly detection for ad campaign traffic on Stellar.

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String};

#[contracttype]
#[derive(Clone)]
pub enum AnomalyType {
    SuddenTrafficSpike,
    UnusualGeoPattern,
    BotLikePattern,
    ClickFarming,
    InvalidTraffic,
    SuspiciousPublisher,
}

#[contracttype]
#[derive(Clone)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[contracttype]
#[derive(Clone)]
pub struct AnomalyReport {
    pub report_id: u64,
    pub campaign_id: u64,
    pub publisher: Option<Address>,
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub description: String,
    pub metrics_snapshot: String, // JSON snapshot of metrics at detection time
    pub auto_action_taken: bool,
    pub reported_at: u64,
    pub resolved: bool,
    pub resolved_at: Option<u64>,
}

/// Parameters for reporting an anomaly — groups the per-report fields to keep
/// `report_anomaly` within clippy's argument-count limit (#751).
#[contracttype]
#[derive(Clone)]
pub struct AnomalyParams {
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub description: String,
    pub metrics_snapshot: String,
    pub auto_action: bool,
    pub current_impressions_per_hour: u64,
    pub current_clicks_per_hour: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct TrafficBaseline {
    pub campaign_id: u64,
    pub avg_impressions_per_hour: u64,
    pub avg_clicks_per_hour: u64,
    pub spike_threshold_pct: u32, // % increase to trigger alert
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    OracleAddress,
    ReportCounter,
    SpikeThreshold,
    Report(u64),
    Baseline(u64), // campaign_id
    FlaggedPublisher(Address),
}

const INSTANCE_LIFETIME_THRESHOLD: u32 = 120_960;
const INSTANCE_BUMP_AMOUNT: u32 = 1_051_200;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 34_560;
const PERSISTENT_BUMP_AMOUNT: u32 = 259_200;
const MAX_BASELINE_MULTIPLIER: u64 = 3; // Max 3x increase per update

#[contract]
pub struct AnomalyDetectorContract;

#[contractimpl]
impl AnomalyDetectorContract {
    pub fn initialize(env: Env, admin: Address, oracle: Address) {
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
            .set(&DataKey::OracleAddress, &oracle);
        env.storage().instance().set(&DataKey::ReportCounter, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::SpikeThreshold, &300u32); // 300% = 3x normal
    }

    pub fn set_baseline(
        env: Env,
        oracle: Address,
        campaign_id: u64,
        avg_impressions: u64,
        avg_clicks: u64,
        spike_threshold: u32,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        oracle.require_auth();
        let stored_oracle: Address = env
            .storage()
            .instance()
            .get(&DataKey::OracleAddress)
            .unwrap();
        if oracle != stored_oracle {
            panic!("unauthorized");
        }

        if avg_impressions == 0 || avg_clicks == 0 {
            panic!("baseline averages must be positive");
        }
        if spike_threshold < 100 {
            panic!("spike_threshold must be at least 100");
        }

        let baseline = TrafficBaseline {
            campaign_id,
            avg_impressions_per_hour: avg_impressions,
            avg_clicks_per_hour: avg_clicks,
            spike_threshold_pct: spike_threshold,
            last_updated: env.ledger().timestamp(),
        };

        let _ttl_key = DataKey::Baseline(campaign_id);
        env.storage().persistent().set(&_ttl_key, &baseline);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn update_baseline(
        env: Env,
        admin: Address,
        campaign_id: u64,
        new_impressions: u64,
        new_clicks: u64,
    ) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let _ttl_key = DataKey::Baseline(campaign_id);
        let current: Option<TrafficBaseline> = env.storage().persistent().get(&_ttl_key);

        if let Some(baseline) = current {
            let max_impressions = baseline
                .avg_impressions_per_hour
                .saturating_mul(MAX_BASELINE_MULTIPLIER);
            let max_clicks = baseline
                .avg_clicks_per_hour
                .saturating_mul(MAX_BASELINE_MULTIPLIER);

            if new_impressions > max_impressions {
                panic!("baseline change too large");
            }
            if new_clicks > max_clicks {
                panic!("baseline change too large");
            }

            env.events().publish(
                (symbol_short!("baseline"), symbol_short!("updated")),
                (
                    campaign_id,
                    baseline.avg_impressions_per_hour,
                    new_impressions,
                ),
            );
        }

        let new_baseline = TrafficBaseline {
            campaign_id,
            avg_impressions_per_hour: new_impressions,
            avg_clicks_per_hour: new_clicks,
            spike_threshold_pct: 300,
            last_updated: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&_ttl_key, &new_baseline);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    /// Report an anomaly for a campaign. Related fields are bundled in `params`
    /// to keep the argument count within clippy's `too_many_arguments` limit (#751).
    pub fn report_anomaly(
        env: Env,
        oracle: Address,
        campaign_id: u64,
        publisher: Option<Address>,
        params: AnomalyParams,
    ) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        oracle.require_auth();
        let stored_oracle: Address = env
            .storage()
            .instance()
            .get(&DataKey::OracleAddress)
            .unwrap();
        if oracle != stored_oracle {
            panic!("unauthorized");
        }

        // Validate against baseline if it exists
        let baseline: Option<TrafficBaseline> = env
            .storage()
            .persistent()
            .get(&DataKey::Baseline(campaign_id));

        if let Some(b) = baseline {
            let threshold_multiplier = b.spike_threshold_pct as u64;

            let impressions_threshold = b
                .avg_impressions_per_hour
                .saturating_mul(threshold_multiplier)
                .saturating_div(100);
            let clicks_threshold = b
                .avg_clicks_per_hour
                .saturating_mul(threshold_multiplier)
                .saturating_div(100);

            let impressions_exceeded =
                params.current_impressions_per_hour > impressions_threshold;
            let clicks_exceeded = params.current_clicks_per_hour > clicks_threshold;

            if !impressions_exceeded && !clicks_exceeded {
                panic!("metrics do not exceed baseline thresholds");
            }
        }

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ReportCounter)
            .unwrap_or(0);
        let report_id = counter + 1;

        // Auto-flag critical publisher anomalies
        if let Some(ref pub_addr) = publisher {
            if let AnomalySeverity::Critical = params.severity {
                let _ttl_key = DataKey::FlaggedPublisher(pub_addr.clone());
                env.storage().persistent().set(&_ttl_key, &true);
                env.storage().persistent().extend_ttl(
                    &_ttl_key,
                    PERSISTENT_LIFETIME_THRESHOLD,
                    PERSISTENT_BUMP_AMOUNT,
                );
            }
        }

        let report = AnomalyReport {
            report_id,
            campaign_id,
            publisher,
            anomaly_type: params.anomaly_type,
            severity: params.severity,
            description: params.description,
            metrics_snapshot: params.metrics_snapshot,
            auto_action_taken: params.auto_action,
            reported_at: env.ledger().timestamp(),
            resolved: false,
            resolved_at: None,
        };

        let _ttl_key = DataKey::Report(report_id);
        env.storage().persistent().set(&_ttl_key, &report);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage()
            .instance()
            .set(&DataKey::ReportCounter, &report_id);

        env.events().publish(
            (symbol_short!("anomaly"), symbol_short!("detected")),
            (report_id, campaign_id),
        );

        report_id
    }

    pub fn resolve_anomaly(env: Env, admin: Address, report_id: u64) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("unauthorized");
        }

        let mut report: AnomalyReport = env
            .storage()
            .persistent()
            .get(&DataKey::Report(report_id))
            .expect("report not found");

        if report.resolved {
            panic!("anomaly report is already resolved");
        }

        report.resolved = true;
        report.resolved_at = Some(env.ledger().timestamp());

        // Clear the persistent publisher flag so the publisher is no longer blocked
        if let Some(ref pub_addr) = report.publisher {
            env.storage()
                .persistent()
                .remove(&DataKey::FlaggedPublisher(pub_addr.clone()));
        }

        let _ttl_key = DataKey::Report(report_id);
        env.storage().persistent().set(&_ttl_key, &report);
        env.storage().persistent().extend_ttl(
            &_ttl_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn get_report(env: Env, report_id: u64) -> Option<AnomalyReport> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage().persistent().get(&DataKey::Report(report_id))
    }

    pub fn get_baseline(env: Env, campaign_id: u64) -> Option<TrafficBaseline> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::Baseline(campaign_id))
    }

    pub fn is_publisher_flagged(env: Env, publisher: Address) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .persistent()
            .get(&DataKey::FlaggedPublisher(publisher))
            .unwrap_or(false)
    }

    pub fn get_report_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        env.storage()
            .instance()
            .get(&DataKey::ReportCounter)
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
