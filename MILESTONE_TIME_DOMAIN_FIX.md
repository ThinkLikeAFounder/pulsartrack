# Milestone Time Domain Consistency Fix

## Problem

The milestone-tracker contract had inconsistent time domains across its fields, creating confusing semantics and making it difficult for frontend consumers to compare and display time-related data.

### Time Domain Mixing

The `Milestone` struct mixed two incompatible time domains:

```rust
pub struct Milestone {
    // ... other fields
    pub deadline_ledger: u32,     // Ledger sequence number
    pub achieved_at: Option<u64>, // Unix timestamp
    pub created_at: u64,          // Unix timestamp
}
```

### Inconsistent Comparisons

In `update_progress()`, the deadline check used ledger sequence:
```rust
} else if env.ledger().sequence() > milestone.deadline_ledger {
    milestone.status = MilestoneStatus::Missed;
}
```

But achievement recording used Unix timestamp:
```rust
milestone.achieved_at = Some(env.ledger().timestamp());
```

### Issues This Created

1. **Frontend Confusion**: Developers couldn't easily compare `deadline_ledger` with `achieved_at` or `created_at`
2. **Display Inconsistency**: Converting ledger sequences to human-readable dates requires additional blockchain queries
3. **Semantic Mismatch**: A milestone's deadline was in one time domain while its achievement time was in another
4. **API Complexity**: Consumers needed to handle two different time representations for the same logical concept

## Solution

Standardized all time-related fields to use Unix timestamps (u64) for consistency and human readability.

### Changes Made

#### 1. Updated Milestone Struct
```rust
pub struct Milestone {
    pub milestone_id: u64,
    pub campaign_id: u64,
    pub description: String,
    pub target_metric: String,
    pub target_value: u64,
    pub current_value: u64,
    pub reward_amount: i128,
    pub status: MilestoneStatus,
    pub deadline: u64,           // Unix timestamp (changed from deadline_ledger: u32)
    pub achieved_at: Option<u64>, // Unix timestamp
    pub created_at: u64,          // Unix timestamp
}
```

#### 2. Updated create_milestone() Function Signature
```rust
pub fn create_milestone(
    env: Env,
    advertiser: Address,
    campaign_id: u64,
    description: String,
    target_metric: String,
    target_value: u64,
    reward_amount: i128,
    deadline: u64,  // Changed from deadline_ledger: u32
) -> u64
```

#### 3. Updated Deadline Check in update_progress()
```rust
} else if env.ledger().timestamp() > milestone.deadline {
    milestone.status = MilestoneStatus::Missed;
}
```

Now both the deadline check and achievement recording use the same time domain (Unix timestamps).

## Benefits

### 1. Consistent Time Representation
All time fields now use Unix timestamps:
- `created_at`: When the milestone was created
- `deadline`: When the milestone must be achieved by
- `achieved_at`: When the milestone was actually achieved

### 2. Easy Comparisons
Frontend code can now directly compare time values:
```javascript
const timeRemaining = milestone.deadline - Date.now() / 1000;
const timeToAchieve = milestone.achieved_at - milestone.created_at;
const achievedBeforeDeadline = milestone.achieved_at <= milestone.deadline;
```

### 3. Human-Readable Display
Unix timestamps can be directly converted to human-readable dates without blockchain queries:
```javascript
const deadlineDate = new Date(milestone.deadline * 1000);
const achievedDate = new Date(milestone.achieved_at * 1000);
```

### 4. Simplified API
Consumers only need to understand one time representation instead of two.

## Tests Added

### 1. test_milestone_missed_after_deadline
Verifies that milestones are correctly marked as "Missed" when the deadline passes:
```rust
// Set deadline to current timestamp (already expired)
let deadline = env.ledger().timestamp();
let id = c.create_milestone(..., deadline);

// Advance time by 1 second
env.ledger().with_mut(|li| {
    li.timestamp = li.timestamp + 1;
});

// Update progress but don't reach target
c.update_progress(&oracle, &id, &500u64);
let m = c.get_milestone(&id).unwrap();

// Should be marked as Missed because deadline passed
assert!(matches!(m.status, MilestoneStatus::Missed));
```

### 2. test_time_domain_consistency
Validates that all time fields are in the same domain and logically consistent:
```rust
let current_time = env.ledger().timestamp();
let deadline = current_time + 86_400; // 1 day from now

let id = c.create_milestone(..., deadline);
c.update_progress(&oracle, &id, &1000u64);
let m = c.get_milestone(&id).unwrap();

// All time fields should be in the same domain (Unix timestamps)
assert!(m.created_at > 0);
assert_eq!(m.deadline, deadline);
assert!(m.achieved_at.is_some());

let achieved_time = m.achieved_at.unwrap();

// achieved_at should be >= created_at
assert!(achieved_time >= m.created_at);

// achieved_at should be <= deadline (achieved before deadline)
assert!(achieved_time <= m.deadline);
```

## Test Results

```
running 10 tests
test test::test_initialize ... ok
test test::test_initialize_twice - should panic ... ok
test test::test_get_milestone_nonexistent ... ok
test test::test_resolve_dispute ... ok
test test::test_time_domain_consistency ... ok
test test::test_create_milestone ... ok
test test::test_milestone_missed_after_deadline ... ok
test test::test_dispute_milestone ... ok
test test::test_update_progress ... ok
test test::test_update_progress_achieves ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured
```

## Migration Notes

### Breaking Change
This is a breaking change to the contract interface. When deploying this update:

1. **Function Signature Change**: `create_milestone()` now accepts `deadline: u64` instead of `deadline_ledger: u32`
2. **Struct Field Change**: `Milestone.deadline_ledger` is now `Milestone.deadline`
3. **Value Interpretation**: Callers must now provide Unix timestamps instead of ledger sequence numbers

### Migration Strategy

For existing deployments:

1. **New Deployments**: Simply deploy the updated contract
2. **Existing Data**: If you have existing milestones with `deadline_ledger` values, you'll need to:
   - Deploy a new contract instance with the updated code
   - Migrate existing milestone data by converting ledger sequences to approximate timestamps
   - Update frontend code to use Unix timestamps

### Conversion Reference

If you need to convert existing ledger sequences to timestamps:
```rust
// Approximate conversion (Stellar ledger closes every ~5 seconds)
let approximate_timestamp = current_timestamp + ((deadline_ledger - current_ledger) * 5);
```

Note: This is an approximation. For precise conversions, query historical ledger data.

## Files Modified

- `contracts/milestone-tracker/src/lib.rs`: Core implementation changes
- `contracts/milestone-tracker/src/test.rs`: Updated tests and added new test cases
- Test snapshots: Updated for new behavior

## Alternative Considered

We considered using ledger sequences for all time fields instead of Unix timestamps. However, Unix timestamps were chosen because:

1. **Human Readability**: Easier for developers and users to understand
2. **Frontend Compatibility**: JavaScript Date objects work natively with Unix timestamps
3. **Industry Standard**: Most APIs and databases use Unix timestamps
4. **No External Dependencies**: Don't require blockchain queries to convert to human-readable dates

## Conclusion

This fix eliminates the confusing time domain mixing in the milestone-tracker contract, making it easier for developers to work with milestone deadlines and achievement times. All time-related fields now use Unix timestamps consistently, improving code clarity and reducing potential bugs in frontend implementations.
