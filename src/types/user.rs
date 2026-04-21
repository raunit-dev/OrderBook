use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::orderbook::OrderBookError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBalance {
    pub user_id: Uuid,
    pub balances: HashMap<String, f64>,
}

impl UserBalance {
    pub fn new(user_id: Uuid) -> Self {
        let mut balances = HashMap::new();
        balances.insert("USD".to_string(), 0.0);
        balances.insert("BTC".to_string(), 0.0);
        UserBalance { user_id, balances }
    }

    pub fn add_balance(&mut self, currency: &str, amount: f64) {
        *self.balances.entry(currency.to_string()).or_insert(0.0) += amount;
    }

    pub fn subtract_balance(
        &mut self,
        currency: &str,
        amount: f64,
    ) -> Result<(), OrderBookError> {
        let balance = self
            .balances
            .get_mut(currency)
            .ok_or_else(|| OrderBookError::UnknownCurrency(currency.to_string()))?;
        if *balance < amount {
            return Err(OrderBookError::InsufficientBalance {
                currency: currency.to_string(),
            });
        }
        *balance -= amount;
        Ok(())
    }

    pub fn get_balance(&self, currency: &str) -> f64 {
        *self.balances.get(currency).unwrap_or(&0.0)
    }
}
