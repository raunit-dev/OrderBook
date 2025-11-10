use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Quantity(u64);

impl Quantity {
    // const DECIMAL_PLACES: u32 = 8;
    const MULTIPLIER: u64 = 100_000_000; // 10^8

    pub fn new(value: u64) -> Self {
        Quantity(value)
    }

    pub fn from_f64(value: f64) -> Self {
        let fixed_point = (value * Self::MULTIPLIER as f64).round() as u64;
        Quantity(fixed_point)
    }

    pub fn to_f64(&self) -> f64 {
        self.0 as f64 / Self::MULTIPLIER as f64
    }

    pub fn raw(&self) -> u64 {
        self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn checked_sub(&self, other: Quantity) -> Option<Quantity> {
        self.0.checked_sub(other.0).map(Quantity)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantity_operations() {
        let q1 = Quantity::from_f64(1.5);
        let q2 = Quantity::from_f64(0.5);
        let result = q1 - q2;
        assert_eq!(result.to_f64(), 1.0);
    }

    #[test]
    fn test_quantity_is_zero() {
        let q = Quantity::new(0);
        assert!(q.is_zero());
    }
}
