use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;
use crate::utils::auth::{generate_token, hash_password, verify_password};
use crate::utils::error::ApiError;

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
    state: web::Data<AppState>,
    req: web::Json<SignupRequest>,
) -> Result<impl Responder, ApiError> {
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

    let password_hash = hash_password(&req.password).map_err(ApiError::InternalError)?;
    let user_id = Uuid::new_v4();

    let result = sqlx::query(
        "INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(&req.username)
    .bind(&req.email)
    .bind(&password_hash)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {}
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            return Err(ApiError::BadRequest(
                "Username or email already exists".to_string(),
            ));
        }
        Err(e) => {
            return Err(ApiError::InternalError(format!("Database error: {}", e)));
        }
    }

    let token =
        generate_token(user_id, req.username.clone()).map_err(ApiError::InternalError)?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user_id: user_id.to_string(),
        username: req.username.clone(),
    }))
}

#[post("/signin")]
pub async fn signin(
    state: web::Data<AppState>,
    req: web::Json<SigninRequest>,
) -> Result<impl Responder, ApiError> {
    if req.username.is_empty() || req.password.is_empty() {
        return Err(ApiError::BadRequest(
            "Username and password are required".to_string(),
        ));
    }

    let row: Option<(Uuid, String, String)> = sqlx::query_as(
        "SELECT id, username, password_hash FROM users WHERE username = $1",
    )
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;

    let (user_id, username, password_hash) =
        row.ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?;

    let valid =
        verify_password(&req.password, &password_hash).map_err(ApiError::InternalError)?;
    if !valid {
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }

    let token = generate_token(user_id, username.clone()).map_err(ApiError::InternalError)?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user_id: user_id.to_string(),
        username,
    }))
}
