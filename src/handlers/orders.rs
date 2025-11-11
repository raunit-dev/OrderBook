use actix_web::{delete, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::messages::{OrderBookCommand, OrderBookResponse};
use crate::state::AppState;
use crate::types::{OrderSide, Price, Quantity};
use crate::utils::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct LimitOrderRequest {
    pub side: String,     // "buy" or "sell"
    pub price: f64,
    pub quantity: f64,
}

#[derive(Debug, Deserialize)]
pub struct MarketOrderRequest {
    pub side: String,     // "buy" or "sell"
    pub quantity: f64,
}

#[derive(Debug, Deserialize)]
pub struct CancelOrderRequest {
    pub order_id: String,
}

#[post("/limit")]
pub async fn create_limit_order(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<LimitOrderRequest>,
) -> Result<impl Responder, ApiError> {
    // Extract user_id from request extensions (added by JWT middleware)
    let user_id = req.extensions().get::<Uuid>().copied()
        .ok_or_else(|| ApiError::Unauthorized("Not authenticated".to_string()))?;

    // Parse side
    let side = match body.side.to_lowercase().as_str() {
        "buy" => OrderSide::Buy,
        "sell" => OrderSide::Sell,
        _ => return Err(ApiError::BadRequest("Invalid side, use 'buy' or 'sell'".to_string())),
    };

    // Create oneshot channel for response
    let (response_tx, response_rx) = oneshot::channel();

    // Send command to orderbook engine
    state.orderbook_tx.send(OrderBookCommand::PlaceLimitOrder {
        user_id,
        side,
        price: Price::from_f64(body.price),
        quantity: Quantity::from_f64(body.quantity),
        response_tx,
    })
    .await
    .map_err(|_| ApiError::InternalError("Failed to send command to orderbook".to_string()))?;

    // Wait for response
    let response = response_rx.await
        .map_err(|_| ApiError::InternalError("Failed to receive response from orderbook".to_string()))?;

    // Handle response
    match response {
        OrderBookResponse::OrderPlaced { order_id, trades, status } => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "order_id": order_id.to_string(),
                "status": status,
                "trades_count": trades.len(),
                "trades": trades,
            })))
        }
        OrderBookResponse::Error { message } => {
            Err(ApiError::BadRequest(message))
        }
        _ => Err(ApiError::InternalError("Unexpected response from orderbook".to_string())),
    }
}

#[post("/market")]
pub async fn create_market_order(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<MarketOrderRequest>,
) -> Result<impl Responder, ApiError> {
    // Extract user_id from JWT
    let user_id = req.extensions().get::<Uuid>().copied()
        .ok_or_else(|| ApiError::Unauthorized("Not authenticated".to_string()))?;

    // Parse side
    let side = match body.side.to_lowercase().as_str() {
        "buy" => OrderSide::Buy,
        "sell" => OrderSide::Sell,
        _ => return Err(ApiError::BadRequest("Invalid side, use 'buy' or 'sell'".to_string())),
    };

    // Create oneshot channel
    let (response_tx, response_rx) = oneshot::channel();

    // Send command
    state.orderbook_tx.send(OrderBookCommand::PlaceMarketOrder {
        user_id,
        side,
        quantity: Quantity::from_f64(body.quantity),
        response_tx,
    })
    .await
    .map_err(|_| ApiError::InternalError("Failed to send command to orderbook".to_string()))?;

    // Wait for response
    let response = response_rx.await
        .map_err(|_| ApiError::InternalError("Failed to receive response from orderbook".to_string()))?;

    // Handle response
    match response {
        OrderBookResponse::OrderPlaced { order_id, trades, status } => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "order_id": order_id.to_string(),
                "status": status,
                "trades_count": trades.len(),
                "trades": trades,
            })))
        }
        OrderBookResponse::Error { message } => {
            Err(ApiError::BadRequest(message))
        }
        _ => Err(ApiError::InternalError("Unexpected response from orderbook".to_string())),
    }
}

#[delete("/cancel")]
pub async fn cancel_order(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<CancelOrderRequest>,
) -> Result<impl Responder, ApiError> {
    // Extract user_id from JWT
    let user_id = req.extensions().get::<Uuid>().copied()
        .ok_or_else(|| ApiError::Unauthorized("Not authenticated".to_string()))?;

    // Parse order_id
    let order_id = Uuid::parse_str(&body.order_id)
        .map_err(|_| ApiError::BadRequest("Invalid order_id format".to_string()))?;

    // Create oneshot channel
    let (response_tx, response_rx) = oneshot::channel();

    // Send command
    state.orderbook_tx.send(OrderBookCommand::CancelOrder {
        user_id,
        order_id,
        response_tx,
    })
    .await
    .map_err(|_| ApiError::InternalError("Failed to send command to orderbook".to_string()))?;

    // Wait for response
    let response = response_rx.await
        .map_err(|_| ApiError::InternalError("Failed to receive response from orderbook".to_string()))?;

    // Handle response
    match response {
        OrderBookResponse::OrderCancelled { order_id, success } => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "order_id": order_id.to_string(),
                "cancelled": success,
            })))
        }
        OrderBookResponse::Error { message } => {
            Err(ApiError::BadRequest(message))
        }
        _ => Err(ApiError::InternalError("Unexpected response from orderbook".to_string())),
    }
}
