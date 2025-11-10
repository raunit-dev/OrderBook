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

    pub fn subtract_balance(&mut self, currency: &str, amount: f64) -> Result<(), String> {
        let balance = self
            .balances
            .get_mut(currency)
            .ok_or("Currency not found")?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let username = "raunit".to_string();
        let email = "raunit@example.com".to_string();
        let password_hash = "hashed_password_123".to_string();

        let user = User::new(username.clone(), email.clone(), password_hash.clone());
        assert_eq!(user.username, username);
        assert_eq!(user.email, email);
        assert_eq!(user.password_hash, password_hash);

        assert!(!user.id.is_nil());
    }

    #[test]
    fn test_unique_user_ids() {
        let user1 = User::new(
            "vidhi".to_string(),
            "vidhi@example.com".to_string(),
            "vidhi1".to_string(),
        );
        let user2 = User::new(
            "rishi".to_string(),
            "rishi@example.com".to_string(),
            "rishi2".to_string(),
        );

        assert_ne!(user1.id, user2.id);
    }

    #[test]
    fn test_user_serialization_and_deserialization() {
        let user = User::new(
            "sona".to_string(),
            "sona@example.com".to_string(),
            "raunit45".to_string(),
        );

        let json = serde_json::to_string(&user).unwrap();
        let deserialized: User = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.username, user.username);
        assert_eq!(deserialized.email, user.email);
        assert_eq!(deserialized.password_hash, user.password_hash);
        assert_eq!(deserialized.id, user.id);
    }
}
