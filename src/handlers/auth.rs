use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::types::User;
use crate::utils::auth::{generate_token, hash_password, verify_password};
use crate::utils::error::ApiError;

// Simple in-memory user store (in production, use a database)
pub struct UserStore {
    pub users: Mutex<HashMap<String, User>>, // username -> User
}

impl UserStore {
    pub fn new() -> Self {
        UserStore {
            users: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SigninRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
}

#[post("/signup")]
pub async fn signup(
    user_store: web::Data<UserStore>,
    req: web::Json<SignupRequest>,
) -> Result<impl Responder, ApiError> {
    // Validate input
    if req.username.is_empty() || req.email.is_empty() || req.password.is_empty() {
        return Err(ApiError::BadRequest(
            "Username, email, and password are required".to_string(),
        ));
    }

    if req.password.len() < 6 {
        return Err(ApiError::BadRequest(
            "Password must be at least 6 characters".to_string(),
        ));
    }

    // Hash password
    let password_hash = hash_password(&req.password)
        .map_err(|e| ApiError::InternalError(e))?;

    // Create user
    let user = User::new(req.username.clone(), req.email.clone(), password_hash);
    let user_id = user.id;
    let username = user.username.clone();

    // Store user
    let mut users = user_store.users.lock().unwrap();

    // Check if username already exists
    if users.contains_key(&req.username) {
        return Err(ApiError::BadRequest("Username already exists".to_string()));
    }

    users.insert(req.username.clone(), user);
    drop(users);

    // Generate token
    let token = generate_token(user_id, username.clone())
        .map_err(|e| ApiError::InternalError(e))?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user_id: user_id.to_string(),
        username,
    }))
}

#[post("/signin")]
pub async fn signin(
    user_store: web::Data<UserStore>,
    req: web::Json<SigninRequest>,
) -> Result<impl Responder, ApiError> {
    // Validate input
    if req.username.is_empty() || req.password.is_empty() {
        return Err(ApiError::BadRequest(
            "Username and password are required".to_string(),
        ));
    }

    // Get user
    let users = user_store.users.lock().unwrap();
    let user = users
        .get(&req.username)
        .ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?
        .clone();
    drop(users);

    // Verify password
    let valid = verify_password(&req.password, &user.password_hash)
        .map_err(|e| ApiError::InternalError(e))?;

    if !valid {
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }

    // Generate token
    let token = generate_token(user.id, user.username.clone())
        .map_err(|e| ApiError::InternalError(e))?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user_id: user.id.to_string(),
        username: user.username,
    }))
}
