use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;

use crate::orderbook::OrderBookError;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    Unauthorized(String),

    #[error("{0}")]
    Forbidden(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    InternalError(String),
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        HttpResponse::build(status).json(ErrorResponse { error: message })
    }
}

/// Map engine errors to the right HTTP status.
impl From<OrderBookError> for ApiError {
    fn from(err: OrderBookError) -> Self {
        match err {
            // Client-fixable inputs: bad currency, not enough funds, missing price, no liquidity.
            OrderBookError::InsufficientBalance { .. }
            | OrderBookError::UnknownCurrency(_)
            | OrderBookError::MissingPrice
            | OrderBookError::InsufficientLiquidity => ApiError::BadRequest(err.to_string()),

            // Nothing exists to act on.
            OrderBookError::UserNotFound(_) | OrderBookError::OrderNotFound(_) => {
                ApiError::NotFound(err.to_string())
            }
        }
    }
}
