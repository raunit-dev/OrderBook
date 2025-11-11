use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const JWT_SECRET: &[u8] = b"your-secret-key-change-in-production";
const TOKEN_EXPIRATION_HOURS: i64 = 24;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // Subject (user_id)
    pub username: String, // Username
    pub exp: usize,       // Expiration time
}

/// Generate JWT token for a user
pub fn generate_token(user_id: Uuid, username: String) -> Result<String, String> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(TOKEN_EXPIRATION_HOURS))
        .ok_or("Failed to calculate expiration time")?
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        username,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .map_err(|e| format!("Failed to generate token: {}", e))
}

/// Validate JWT token and extract claims
pub fn validate_token(token: &str) -> Result<Claims, String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| format!("Invalid token: {}", e))
}

/// Hash password using bcrypt
pub fn hash_password(password: &str) -> Result<String, String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| format!("Failed to hash password: {}", e))
}

/// Verify password against hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    bcrypt::verify(password, hash).map_err(|e| format!("Failed to verify password: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "my_secure_password";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_token_generation() {
        let user_id = Uuid::new_v4();
        let token = generate_token(user_id, "testuser".to_string()).unwrap();
        let claims = validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.username, "testuser");
    }
}
