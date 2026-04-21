use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Quantity(u64);

impl Quantity {
    const MULTIPLIER: u64 = 100_000_000; // 10^8

    pub fn new(value: u64) -> Self {
        Quantity(value)
    }

    pub fn from_f64(value: f64) -> Self {
        let fixed_point = (value * Self::MULTIPLIER as f64).round() as u64;
        Quantity(fixed_point)
    }

    pub fn to_f64(self) -> f64 {
        self.0 as f64 / Self::MULTIPLIER as f64
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl Add for Quantity {
    type Output = Quantity;
    fn add(self, other: Quantity) -> Quantity {
        Quantity(self.0 + other.0)
    }
}

impl Sub for Quantity {
    type Output = Quantity;
    fn sub(self, other: Quantity) -> Quantity {
        Quantity(self.0 - other.0)
    }
}

impl AddAssign for Quantity {
    fn add_assign(&mut self, other: Quantity) {
        self.0 += other.0;
    }
}

impl SubAssign for Quantity {
    fn sub_assign(&mut self, other: Quantity) {
        self.0 -= other.0;
    }
}

impl std::fmt::Display for Quantity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.8}", self.to_f64())
    }
}
