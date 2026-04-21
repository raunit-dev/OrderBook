use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::orderbook::OrderBookError;
use crate::types::Currency;

/// User balance across supported currencies.
/// Amounts are stored as integer minor units (USD in millionths, BTC in satoshis) —
/// never f64 — to avoid float drift on the hot path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBalance {
    pub user_id: Uuid,
    pub balances: HashMap<Currency, u64>,
}

impl UserBalance {
    pub fn new(user_id: Uuid) -> Self {
        let mut balances = HashMap::new();
        balances.insert(Currency::Usd, 0);
        balances.insert(Currency::Btc, 0);
        UserBalance { user_id, balances }
    }

    pub fn add(&mut self, currency: Currency, amount: u64) {
        *self.balances.entry(currency).or_insert(0) += amount;
    }

    pub fn subtract(&mut self, currency: Currency, amount: u64) -> Result<(), OrderBookError> {
        let balance = self.balances.entry(currency).or_insert(0);
        if *balance < amount {
            return Err(OrderBookError::InsufficientBalance { currency });
        }
        *balance -= amount;
        Ok(())
    }

    pub fn get(&self, currency: Currency) -> u64 {
        *self.balances.get(&currency).unwrap_or(&0)
    }
}
