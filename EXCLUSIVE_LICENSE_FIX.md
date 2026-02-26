# Exclusive License Duplicate Content Fix

## Problem

The creative-marketplace contract had a critical vulnerability where creators could defeat exclusive license guarantees by re-listing the same content after selling an exclusive license.

### Attack Scenario
1. Creator lists creative with `content_hash = "Qm..."` as `Exclusive`
2. Buyer purchases exclusive license
3. Listing status → `Sold`
4. Creator calls `create_listing()` again with the same `content_hash` → succeeds
5. Buyer's "exclusive" license is now worthless

## Solution

Added content ownership tracking to prevent duplicate exclusive content listings.

### Changes Made

#### 1. Enhanced DataKey Enum
```rust
pub enum DataKey {
    // ... existing variants
    ContentOwner(String),  // content_hash -> tracks exclusive licenses
}
```

#### 2. Duplicate Check in create_listing()
```rust
// Check for duplicate content hash with exclusive license
if env
    .storage()
    .persistent()
    .has(&DataKey::ContentOwner(content_hash.clone()))
{
    panic!("content already listed - check for exclusive licenses");
}
```

#### 3. Set Content Owner for Exclusive Listings
```rust
// If this is an exclusive license, mark content as owned
if matches!(license_type, LicenseType::Exclusive) {
    let content_key = DataKey::ContentOwner(content_hash);
    env.storage().persistent().set(&content_key, &listing_id);
    env.storage().persistent().extend_ttl(
        &content_key,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}
```

#### 4. Clear Content Owner on Removal
```rust
// If this was an exclusive license, clear the content owner
if matches!(listing.license_type, LicenseType::Exclusive) {
    let content_key = DataKey::ContentOwner(listing.content_hash.clone());
    if env.storage().persistent().has(&content_key) {
        env.storage().persistent().remove(&content_key);
    }
}
```

## Behavior

### Exclusive Licenses
- ✅ First exclusive listing with content hash → succeeds
- ❌ Second exclusive listing with same content hash → panics with error
- ✅ After removing first listing → new listing with same hash succeeds

### Non-Exclusive Licenses
- ✅ Multiple listings with same content hash → all succeed
- No restrictions on duplicate content for OneTime, Recurring, or OpenSource licenses

## Tests Added

1. **test_duplicate_exclusive_content_blocked**: Verifies duplicate exclusive content is rejected
2. **test_non_exclusive_allows_duplicate_content**: Confirms non-exclusive licenses allow duplicates
3. **test_remove_exclusive_listing_allows_recreation**: Tests that removing an exclusive listing clears the content lock
4. **test_exclusive_license_marks_sold**: Validates exclusive purchases mark listing as Sold

## Test Results

```
running 11 tests
test test::test_get_listing_nonexistent ... ok
test test::test_create_listing ... ok
test test::test_initialize ... ok
test test::test_initialize_twice - should panic ... ok
test test::test_has_license_false ... ok
test test::test_non_exclusive_allows_duplicate_content ... ok
test test::test_purchase_license ... ok
test test::test_exclusive_license_marks_sold ... ok
test test::test_duplicate_exclusive_content_blocked - should panic ... ok
test test::test_remove_exclusive_listing_allows_recreation ... ok
test test::test_remove_listing ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured
```

## Files Modified

- `contracts/creative-marketplace/src/lib.rs`: Core implementation
- `contracts/creative-marketplace/src/test.rs`: Test suite with 4 new tests
- Test snapshots: Updated for new behavior

## Security Impact

This fix closes a critical vulnerability that would have allowed creators to violate exclusive license agreements, potentially leading to:
- Loss of buyer trust
- Legal disputes
- Platform reputation damage
- Financial losses for exclusive license buyers

## Deployment Notes

When deploying this fix:
1. Existing exclusive listings will need migration if content ownership tracking is required retroactively
2. No breaking changes to the contract interface
3. Backward compatible with existing non-exclusive listings
