use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

// A custom Amount type since we're doing financial transactions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Amount(i64);

impl Amount {
    pub fn new(whole_units: u64) -> Self {
        Self((whole_units * 10000) as i64)
    }
}

impl From<u64> for Amount {
    fn from(value: u64) -> Self {
        Self((value * 10000) as i64)
    }
}

impl From<f64> for Amount {
    fn from(value: f64) -> Self {
        Self((value * 10000.0) as i64)
    }
}

impl From<Amount> for f64 {
    fn from(amount: Amount) -> Self {
        amount.0 as f64 / 10000.0
    }
}

impl Add for Amount {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl AddAssign for Amount {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl Sub for Amount {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl SubAssign for Amount {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl Neg for Amount {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repeated_addition_no_drift() {
        let mut total = Amount::from(0.0);
        let increment = Amount::from(0.1);

        for _ in 0..10 {
            total += increment;
        }

        assert_eq!(total, Amount::from(1.0));
    }

    #[test]
    fn test_subtraction_then_addition_identity() {
        let start = Amount::from(100.0);
        let subtract = Amount::from(0.1);
        let result = start - subtract + subtract;

        assert_eq!(result, start);
    }

    #[test]
    fn test_precise_decimal_representation() {
        let a = Amount::from(0.1);
        let b = Amount::from(0.2);
        let sum = a + b;

        assert_eq!(sum, Amount::from(0.3));
    }

    #[test]
    fn test_large_number_small_increment() {
        let large = Amount::from(1000000.0);
        let small = Amount::from(0.01);
        let result = large + small;

        assert_eq!(result, Amount::from(1000000.01));
    }

    #[test]
    fn test_many_small_additions() {
        let mut total = Amount::from(0.0);

        for _ in 0..100 {
            total += Amount::from(0.01);
        }

        assert_eq!(total, Amount::from(1.0));
    }

    #[test]
    fn test_subtraction_precision() {
        let a = Amount::from(1.0);
        let b = Amount::from(0.9999);
        let result = a - b;

        assert_eq!(result, Amount::from(0.0001));
    }

    #[test]
    fn test_negation_and_addition() {
        let amount = Amount::from(42.5);
        let result = amount + (-amount);

        assert_eq!(result, Amount::from(0.0));
    }

    #[test]
    fn test_comparison_with_close_values() {
        let a = Amount::from(0.0001);
        let b = Amount::from(0.0002);

        assert!(a < b);
        assert!(b > a);
        assert_ne!(a, b);
    }

    #[test]
    fn test_four_decimal_precision() {
        let a = Amount::from(1.2345);
        let b = Amount::from(1.2346);

        assert_ne!(a, b);
    }

    #[test]
    fn test_decimal_truncation() {
        let a = Amount::from(1.99999);
        let b = Amount::from(0.00001);

        assert_eq!(a + b, a);
    }
}
