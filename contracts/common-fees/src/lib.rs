#![no_std]

/// Shared fee-calculation helpers using checked arithmetic.
///
/// All functions panic with a clear message on overflow rather than wrapping.

/// Calculate a fee in basis points: `amount * bps / 10_000`.
///
/// Uses `checked_mul` and `checked_div` to prevent overflow on the
/// multiplication step. Panics on overflow with a descriptive message.
pub fn calculate_fee_bps(amount: i128, bps: u32) -> i128 {
    let bps_i128 = bps as i128;
    amount
        .checked_mul(bps_i128)
        .expect("fee calculation overflow: amount * bps exceeds i128")
        .checked_div(10_000)
        .expect("fee calculation overflow: division by 10_000 failed")
}

/// Calculate the net amount after deducting a fee in basis points.
///
/// Returns `(fee, net_amount)`. Panics on overflow.
pub fn split_fee_bps(amount: i128, bps: u32) -> (i128, i128) {
    let fee = calculate_fee_bps(amount, bps);
    let net = amount.checked_sub(fee).expect("fee exceeds amount");
    (fee, net)
}

/// Checked addition for accumulators. Panics on overflow.
pub fn checked_add(a: i128, b: i128) -> i128 {
    a.checked_add(b).expect("accumulator overflow")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_fee_bps_normal() {
        assert_eq!(calculate_fee_bps(10_000, 250), 250);
        assert_eq!(calculate_fee_bps(1_000, 100), 10);
        assert_eq!(calculate_fee_bps(100, 500), 50);
    }

    #[test]
    fn test_calculate_fee_bps_zero() {
        assert_eq!(calculate_fee_bps(0, 250), 0);
        assert_eq!(calculate_fee_bps(10_000, 0), 0);
    }

    #[test]
    fn test_split_fee_bps() {
        let (fee, net) = split_fee_bps(10_000, 250);
        assert_eq!(fee, 250);
        assert_eq!(net, 9_750);
    }

    #[test]
    fn test_checked_add_normal() {
        assert_eq!(checked_add(100, 200), 300);
    }

    #[test]
    #[should_panic(expected = "accumulator overflow")]
    fn test_checked_add_overflow() {
        checked_add(i128::MAX, 1);
    }

    #[test]
    fn test_near_overflow_fee() {
        // Large amount that would overflow with unchecked mul
        let amount = i128::MAX / 2;
        let fee = calculate_fee_bps(amount, 1);
        assert!(fee > 0);
    }

    #[test]
    #[should_panic(expected = "fee calculation overflow")]
    fn test_fee_bps_overflow() {
        calculate_fee_bps(i128::MAX, 10_000);
    }
}
