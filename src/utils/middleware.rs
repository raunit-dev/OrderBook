use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use uuid::Uuid;

use crate::utils::auth::validate_token;
use crate::utils::error::ApiError;

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();

    match validate_token(token) {
        Ok(claims) => {
            // Parse user_id from claims
            match Uuid::parse_str(&claims.sub) {
                Ok(user_id) => {
                    // Store user_id in request extensions for later use
                    req.extensions_mut().insert(user_id);
                    Ok(req)
                }
                Err(_) => Err((
                    ApiError::Unauthorized("Invalid user ID in token".to_string()).into(),
                    req,
                )),
            }
        }
        Err(_) => Err((
            ApiError::Unauthorized("Invalid or expired token".to_string()).into(),
            req,
        )),
    }
}
