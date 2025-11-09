use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Price in fixed-point representation (6 decimal places)
/// Example: 1.234567 BTC = 1_234_567
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Price(u64);

impl Price {
    const DECIMAL_PLACES: u32 = 6;
    const MULTIPLIER: u64 = 1_000_000; // 10^6

    pub fn new(value: u64) -> Self {
        Price(value)
    }

    pub fn from_f64(value: f64) -> Self {
        let fixed_point = (value * Self::MULTIPLIER as f64).round() as u64;
        Price(fixed_point)
    }

    pub fn to_f64(&self) -> f64 {
        self.0 as f64 / Self::MULTIPLIER as f64
    }

    pub fn raw(&self) -> u64 {
        self.0
    }
}

//Trait purpose: Enables <, >, <=, and >= comparisons.
impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Price {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl std::fmt::Display for Price {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.6}", self.to_f64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_creation() {
        let price = Price::from_f64(123.456789);
        assert_eq!(price.raw(), 123_456_789);
    }

    #[test]
    fn test_price_creation_reverse() {
        //Remove this test later
        let price = Price::to_f64(123_456_789);
        assert_eq!(price.raw(), 123.456789);
    }

    #[test]
    fn test_price_ordering() {
        let p1 = Price::from_f64(100.0);
        let p2 = Price::from_f64(200.0);
        assert!(p1 < p2);
    }
}
