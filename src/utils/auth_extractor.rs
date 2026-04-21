use actix_web::{dev::Payload, FromRequest, HttpMessage, HttpRequest};
use std::future::{ready, Ready};
use uuid::Uuid;

use crate::utils::error::ApiError;

/// Typed handler parameter that resolves the authenticated user from request extensions.
/// The JWT middleware inserts the user id as a `Uuid`; this extractor pulls it out.
#[derive(Debug, Clone, Copy)]
pub struct AuthUser {
    pub id: Uuid,
}

impl FromRequest for AuthUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let id = req.extensions().get::<Uuid>().copied();
        ready(match id {
            Some(id) => Ok(AuthUser { id }),
            None => Err(ApiError::Unauthorized("not authenticated".to_string())),
        })
    }
}
