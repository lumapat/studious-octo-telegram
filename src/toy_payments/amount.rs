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
