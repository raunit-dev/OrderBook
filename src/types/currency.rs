use serde::{Deserialize, Serialize};
use std::fmt;

/// Fiat/crypto currencies supported by the engine.
/// Stored internally as a typed enum; serializes as its uppercase ticker ("USD", "BTC").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    Usd,
    Btc,
}

impl Currency {
    /// Scale factor converting a human-facing amount into the currency's integer minor units.
    /// - USD uses 1e6 (matches `Price`'s precision): $100.00 → 100_000_000
    /// - BTC uses 1e8 (satoshis):                   0.5 BTC → 50_000_000
    pub const fn multiplier(self) -> u64 {
        match self {
            Currency::Usd => 1_000_000,
            Currency::Btc => 100_000_000,
        }
    }

    /// Convert a decimal amount (e.g. from a JSON body) into integer minor units.
    pub fn from_f64(self, value: f64) -> u64 {
        (value * self.multiplier() as f64).round() as u64
    }

    /// Convert integer minor units back to a decimal amount for display.
    pub fn to_f64(self, raw: u64) -> f64 {
        raw as f64 / self.multiplier() as f64
    }

    /// Parse a string ticker ("USD", "BTC", case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_uppercase().as_str() {
            "USD" => Some(Currency::Usd),
            "BTC" => Some(Currency::Btc),
            _ => None,
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::Usd => write!(f, "USD"),
            Currency::Btc => write!(f, "BTC"),
        }
    }
}
