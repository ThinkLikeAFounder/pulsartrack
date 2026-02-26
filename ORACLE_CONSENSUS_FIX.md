# Performance Oracle Consensus Averaging Fix

## Problem

The performance-oracle contract claimed to compute "simple average consensus" but actually used only the last attester's values, giving them 100% influence over the consensus result.

### False Consensus Implementation

The `_try_build_consensus()` function was documented as "Build simple average consensus" but did not compute any averages:

```rust
fn _try_build_consensus(
    env: &Env,
    campaign_id: u64,
    impressions: u64,      // <-- Last attester's value
    clicks: u64,           // <-- Last attester's value
    fraud_rate: u32,       // <-- Last attester's value
    quality_score: u32,    // <-- Last attester's value
    total_attesters: u32,
) {
    // ...
    let consensus = OracleConsensus {
        avg_impressions: impressions,    // NOT an average!
        avg_clicks: clicks,              // NOT an average!
        avg_fraud_rate: fraud_rate,      // NOT an average!
        avg_quality_score: quality_score, // NOT an average!
        // ...
    };
}
```

### Issues This Created

1. **Unfair Influence**: The last attester to submit had 100% control over the "consensus"
2. **Misleading Field Names**: `OracleConsensus` fields named `avg_*` contained single attester values
3. **Broken Trust Model**: Multiple attesters were required but only one's data was used
4. **Attack Vector**: Malicious attester could wait to be last and submit fraudulent data
5. **False Documentation**: Code comments claimed averaging but implementation didn't match

### Attack Scenario

```
1. Attester A submits: 1000 impressions, 5% fraud rate
2. Attester B submits: 1100 impressions, 6% fraud rate  
3. Malicious Attester C waits and submits: 10000 impressions, 50% fraud rate
4. Consensus result: 10000 impressions, 50% fraud rate (only C's values!)
5. Attesters A and B's data completely ignored
```

## Solution

Implemented true consensus averaging by storing all attestations and computing actual averages when building consensus.

### Changes Made

#### 1. Added Attester Index Tracking

```rust
pub enum DataKey {
    // ... existing variants
    CampaignAttesterIndex(u64, u32), // campaign_id, index -> Address
}
```

This allows us to iterate through all attesters for a campaign.

#### 2. Store Attester Addresses During Submission

```rust
pub fn submit_attestation(...) {
    // ... existing code
    
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
    
    // ... rest of code
}
```

#### 3. Rewrote Consensus Calculation to Compute Real Averages

```rust
fn _try_build_consensus(env: &Env, campaign_id: u64, total_attesters: u32) {
    let min_attesters: u32 = env
        .storage()
        .instance()
        .get(&DataKey::MinAttesters)
        .unwrap_or(3);

    if total_attesters < min_attesters {
        return;
    }

    // Compute actual averages by reading all attestations
    let mut sum_impressions: u64 = 0;
    let mut sum_clicks: u64 = 0;
    let mut sum_fraud_rate: u64 = 0;
    let mut sum_quality_score: u64 = 0;

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

        sum_impressions += attestation.impressions_verified;
        sum_clicks += attestation.clicks_verified;
        sum_fraud_rate += attestation.fraud_rate as u64;
        sum_quality_score += attestation.quality_score as u64;
    }

    // Calculate averages
    let avg_impressions = sum_impressions / (total_attesters as u64);
    let avg_clicks = sum_clicks / (total_attesters as u64);
    let avg_fraud_rate = (sum_fraud_rate / (total_attesters as u64)) as u32;
    let avg_quality_score = (sum_quality_score / (total_attesters as u64)) as u32;

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

    // ... store consensus
}
```

#### 4. Updated Function Signature

Changed from:
```rust
fn _try_build_consensus(
    env: &Env,
    campaign_id: u64,
    impressions: u64,      // Individual attester values
    clicks: u64,
    fraud_rate: u32,
    quality_score: u32,
    total_attesters: u32,
)
```

To:
```rust
fn _try_build_consensus(
    env: &Env,
    campaign_id: u64,
    total_attesters: u32,  // Only need count, will read all attestations
)
```

## Benefits

### 1. True Consensus
All attesters now contribute equally to the consensus result:
```
Attester A: 1000 impressions
Attester B: 1100 impressions  
Attester C: 1200 impressions
Consensus: 1100 impressions (actual average)
```

### 2. Fair Influence
No single attester can dominate the consensus:
```
Attester A: 1000 impressions, 5% fraud
Attester B: 1100 impressions, 6% fraud
Malicious C: 10000 impressions, 50% fraud
Consensus: 4033 impressions, 20.3% fraud (C's influence is diluted)
```

### 3. Accurate Field Names
`OracleConsensus.avg_*` fields now actually contain averages.

### 4. Attack Resistance
Malicious attesters cannot override consensus by submitting last.

### 5. Transparent Calculation
The averaging logic is explicit and verifiable.

## Tests Added

### 1. test_consensus_with_actual_averaging
Vhttps://github.com/Manuelshub/pulsartrack/pull/new/fix/oracle-consensus-averagingerifies that consensus correctly averages two attesters' values:
```rust
// Attester 1: 1000 impressions, 100 clicks, 1000 fraud_rate, 80 quality
// Attester 2: 2000 impressions, 200 clicks, 2000 fraud_rate, 90 quality
// Expected: 1500 impressions, 150 clicks, 1500 fraud_rate, 85 quality
```

### 2. test_consensus_with_three_attesters
Tests averaging with three attesters:
```rust
// Attester 1: 900 impressions, 90 clicks, 500 fraud_rate, 70 quality
// Attester 2: 1200 impressions, 120 clicks, 1000 fraud_rate, 80 quality
// Attester 3: 1500 impressions, 150 clicks, 1500 fraud_rate, 90 quality
// Expected: 1200 impressions, 120 clicks, 1000 fraud_rate, 80 quality
```

### 3. test_last_attester_does_not_override_consensus
Proves that the last attester cannot override consensus:
```rust
// First two attesters: ~1000 impressions
// Third attester: 9000 impressions (outlier)
// Consensus: 3700 impressions (includes all three, not just last)
```

### 4. test_no_consensus_with_insufficient_attesters
Validates that consensus requires minimum attesters:
```rust
// Only 1 attester (need min 2)
// Result: No consensus created
```

## Test Results

```
running 11 tests
test test::test_authorize_attester ... ok
test test::test_initialize_non_admin_fails - should panic ... ok
test test::test_get_consensus_nonexistent ... ok
test test::test_initialize_twice - should panic ... ok
test test::test_initialize ... ok
test test::test_get_attestation_count ... ok
test test::test_consensus_with_actual_averaging ... ok
test test::test_no_consensus_with_insufficient_attesters ... ok
test test::test_submit_attestation ... ok
test test::test_last_attester_does_not_override_consensus ... ok
test test::test_consensus_with_three_attesters ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured
```

## Performance Considerations

### Storage Cost
- Additional storage: One `Address` per attester per campaign
- Storage key: `CampaignAttesterIndex(campaign_id, index)`
- Cost scales linearly with number of attesters

### Computation Cost
- Consensus calculation now iterates through all attestations
- Time complexity: O(n) where n = number of attesters
- Acceptable for reasonable attester counts (typically 3-10)

### Gas Optimization
For very large attester sets, consider:
1. Incremental averaging (update running average on each attestation)
2. Median calculation instead of mean (more outlier-resistant)
3. Weighted averaging based on attester reputation

## Migration Notes

### Breaking Change
This is a breaking change to the internal consensus calculation logic.

### Deployment Strategy
1. **New Deployments**: Simply deploy the updated contract
2. **Existing Deployments**: 
   - Existing consensus values were computed incorrectly
   - Consider re-computing consensus for active campaigns
   - New attestations will use correct averaging

### Data Migration
If you have existing campaigns with consensus:
1. The old consensus values are invalid (used only last attester's data)
2. You may want to clear old consensus and require re-attestation
3. Or accept that historical data used the flawed calculation

## Alternative Approaches Considered

### 1. Median Instead of Mean
- **Pros**: More resistant to outliers
- **Cons**: More complex to compute on-chain, requires sorting
- **Decision**: Mean is simpler and sufficient with attester authorization

### 2. Weighted Averaging
- **Pros**: Could weight by attester reputation
- **Cons**: Adds complexity, requires reputation system
- **Decision**: Simple averaging is fair when all attesters are authorized

### 3. Incremental Averaging
- **Pros**: No need to iterate through all attestations
- **Cons**: Requires storing running sums, more complex state management
- **Decision**: Current approach is clearer and easier to verify

## Security Impact

This fix closes a critical vulnerability in the oracle consensus mechanism:

### Before
- Single attester could manipulate consensus by submitting last
- Multi-attester requirement was security theater
- Consensus values were unreliable

### After
- All attesters contribute equally to consensus
- Malicious attester influence is diluted by honest attesters
- Consensus values are trustworthy averages

## Files Modified

- `contracts/performance-oracle/src/lib.rs`: Core implementation
- `contracts/performance-oracle/src/test.rs`: Added 4 new comprehensive tests
- Test snapshots: Updated for new behavior

## Conclusion

This fix transforms the performance-oracle from a broken consensus mechanism (last-attester-wins) to a true multi-attester averaging system. The `avg_*` fields in `OracleConsensus` now actually contain averages, making the oracle trustworthy for campaign performance validation.
