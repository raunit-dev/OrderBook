use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

impl User {
    pub fn new(username: String, email: String, password_hash: String) -> Self {
        User {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBalance {
    pub user_id: Uuid,
    pub balances: HashMap<String, f64>, // currency -> amount (e.g., "USD", "BTC")
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

    pub fn subtract_balance(&mut self, currency: &str, amount: f64) -> Result<(), String> {
        let balance = self.balances.get_mut(currency).ok_or("Currency not found")?;

        if *balance < amount {
            return Err("Insufficient balance".to_string());
        }

        *balance -= amount;
        Ok(())
    }

    pub fn get_balance(&self, currency: &str) -> f64 {
        *self.balances.get(currency).unwrap_or(&0.0)
    }
}
