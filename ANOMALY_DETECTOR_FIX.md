# Anomaly Detector Baseline Validation Fix

## Problem

The `report_anomaly()` function in `contracts/anomaly-detector/src/lib.rs` was accepting arbitrary anomaly reports from the oracle without validating them against stored baseline data. The `set_baseline()` and `report_anomaly()` functions were completely disconnected, making the spike thresholds serve no purpose.

## Solution

Modified `report_anomaly()` to validate reported metrics against stored baselines:

### Changes Made

1. **Added validation parameters** to `report_anomaly()`:
   - `current_impressions_per_hour: u64`
   - `current_clicks_per_hour: u64`

2. **Implemented baseline validation logic**:
   - Retrieves the stored baseline for the campaign
   - Calculates threshold values based on `spike_threshold_pct`
   - Validates that at least one metric (impressions or clicks) exceeds the threshold
   - Panics with "metrics do not exceed baseline thresholds" if validation fails

3. **Graceful handling when no baseline exists**:
   - If no baseline is set for a campaign, the anomaly report is accepted without validation
   - This allows flexibility for new campaigns or manual anomaly reporting

### Validation Logic

```rust
// Calculate threshold (e.g., 300% = 3.0x baseline)
let impressions_threshold = baseline.avg_impressions_per_hour * spike_threshold_pct / 100;
let clicks_threshold = baseline.avg_clicks_per_hour * spike_threshold_pct / 100;

// At least one metric must exceed threshold
if current_impressions <= impressions_threshold && current_clicks <= clicks_threshold {
    panic!("metrics do not exceed baseline thresholds");
}
```

### Test Coverage

Added comprehensive tests:

- `test_report_anomaly` - Updated to set baseline and provide valid metrics
- `test_report_anomaly_below_threshold` - Verifies rejection when metrics don't exceed threshold
- `test_report_anomaly_no_baseline` - Confirms reports work without baseline
- `test_report_anomaly_clicks_exceed_threshold` - Tests partial threshold exceedance

All 12 tests pass successfully.

## Impact

- Prevents oracle from reporting false anomalies
- Ensures baseline thresholds are actually enforced
- Maintains backward compatibility for campaigns without baselines
- Provides clear error messages when validation fails
