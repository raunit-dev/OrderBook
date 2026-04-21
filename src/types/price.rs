use crate::types::Quantity;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Price(u64);

impl Price {
    pub const MULTIPLIER: u64 = 1_000_000;

    pub fn from_f64(value: f64) -> Self {
        let fixed_point = (value * Self::MULTIPLIER as f64).round() as u64;
        Price(fixed_point)
    }

    pub fn to_f64(self) -> f64 {
        self.0 as f64 / Self::MULTIPLIER as f64
    }

    pub fn raw(self) -> u64 {
        self.0
    }

    /// USD value in millionths (same scale as Price) of `quantity` BTC at this price.
    /// Computed in u128 to avoid overflow; caller may lossily truncate to u64.
    pub fn times_quantity(self, quantity: Quantity) -> u64 {
        let product = (self.raw() as u128) * (quantity.raw() as u128);
        (product / Quantity::MULTIPLIER as u128) as u64
    }
}

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
