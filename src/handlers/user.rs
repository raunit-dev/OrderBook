use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::messages::{OrderBookCommand, OrderBookResponse};
use crate::state::AppState;
use crate::utils::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct OnrampRequest {
    pub currency: String, // "USD" or "BTC"
    pub amount: f64,
}

#[get("/balance")]
pub async fn get_balance(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    // Extract user_id from JWT
    let user_id = req.extensions().get::<Uuid>().copied()
        .ok_or_else(|| ApiError::Unauthorized("Not authenticated".to_string()))?;

    // Create oneshot channel
    let (response_tx, response_rx) = oneshot::channel();

    // Send command
    state.orderbook_tx.send(OrderBookCommand::GetUserBalance {
        user_id,
        response_tx,
    })
    .await
    .map_err(|_| ApiError::InternalError("Failed to send command to orderbook".to_string()))?;

    // Wait for response
    let response = response_rx.await
        .map_err(|_| ApiError::InternalError("Failed to receive response from orderbook".to_string()))?;

    // Handle response
    match response {
        OrderBookResponse::UserBalance { balance } => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "user_id": balance.user_id.to_string(),
                "balances": balance.balances,
            })))
        }
        OrderBookResponse::Error { message } => {
            Err(ApiError::NotFound(message))
        }
        _ => Err(ApiError::InternalError("Unexpected response from orderbook".to_string())),
    }
}

#[post("/onramp")]
pub async fn onramp(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<OnrampRequest>,
) -> Result<impl Responder, ApiError> {
    // Extract user_id from JWT
    let user_id = req.extensions().get::<Uuid>().copied()
        .ok_or_else(|| ApiError::Unauthorized("Not authenticated".to_string()))?;

    // Validate currency
    if body.currency != "USD" && body.currency != "BTC" {
        return Err(ApiError::BadRequest("Currency must be 'USD' or 'BTC'".to_string()));
    }

    // Validate amount
    if body.amount <= 0.0 {
        return Err(ApiError::BadRequest("Amount must be positive".to_string()));
    }

    // Create oneshot channel
    let (response_tx, response_rx) = oneshot::channel();

    // Send command
    state.orderbook_tx.send(OrderBookCommand::AddFunds {
        user_id,
        currency: body.currency.clone(),
        amount: body.amount,
        response_tx,
    })
    .await
    .map_err(|_| ApiError::InternalError("Failed to send command to orderbook".to_string()))?;

    // Wait for response
    let response = response_rx.await
        .map_err(|_| ApiError::InternalError("Failed to receive response from orderbook".to_string()))?;

    // Handle response
    match response {
        OrderBookResponse::FundsAdded { user_id, currency, new_balance } => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "user_id": user_id.to_string(),
                "currency": currency,
                "new_balance": new_balance,
            })))
        }
        _ => Err(ApiError::InternalError("Unexpected response from orderbook".to_string())),
    }
}
