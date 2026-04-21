use actix_web::{delete, post, web, HttpResponse, Responder};
use serde::Deserialize;
use uuid::Uuid;

use crate::messages::{OrderBookCommand, OrderBookResponse};
use crate::state::AppState;
use crate::types::{OrderSide, Price, Quantity};
use crate::utils::error::ApiError;
use crate::utils::AuthUser;

#[derive(Debug, Deserialize)]
pub struct LimitOrderRequest {
    pub side: String,
    pub price: f64,
    pub quantity: f64,
}

#[derive(Debug, Deserialize)]
pub struct MarketOrderRequest {
    pub side: String,
    pub quantity: f64,
}

#[derive(Debug, Deserialize)]
pub struct CancelOrderRequest {
    pub order_id: String,
}

fn parse_side(s: &str) -> Result<OrderSide, ApiError> {
    match s.to_lowercase().as_str() {
        "buy" => Ok(OrderSide::Buy),
        "sell" => Ok(OrderSide::Sell),
        _ => Err(ApiError::BadRequest(
            "invalid side, use 'buy' or 'sell'".to_string(),
        )),
    }
}

#[post("/limit")]
pub async fn create_limit_order(
    user: AuthUser,
    state: web::Data<AppState>,
    body: web::Json<LimitOrderRequest>,
) -> Result<impl Responder, ApiError> {
    let side = parse_side(&body.side)?;

    let response = state
        .send_command(|response_tx| OrderBookCommand::PlaceLimitOrder {
            user_id: user.id,
            side,
            price: Price::from_f64(body.price),
            quantity: Quantity::from_f64(body.quantity),
            response_tx,
        })
        .await?;

    match response {
        OrderBookResponse::OrderPlaced {
            order_id,
            trades,
            status,
        } => Ok(HttpResponse::Ok().json(serde_json::json!({
            "order_id": order_id.to_string(),
            "status": status,
            "trades_count": trades.len(),
            "trades": trades,
        }))),
        OrderBookResponse::Error(err) => Err(err.into()),
        OrderBookResponse::Rejected(msg) => Err(ApiError::Forbidden(msg)),
        _ => Err(ApiError::InternalError(
            "unexpected response from orderbook".to_string(),
        )),
    }
}

#[post("/market")]
pub async fn create_market_order(
    user: AuthUser,
    state: web::Data<AppState>,
    body: web::Json<MarketOrderRequest>,
) -> Result<impl Responder, ApiError> {
    let side = parse_side(&body.side)?;

    let response = state
        .send_command(|response_tx| OrderBookCommand::PlaceMarketOrder {
            user_id: user.id,
            side,
            quantity: Quantity::from_f64(body.quantity),
            response_tx,
        })
        .await?;

    match response {
        OrderBookResponse::OrderPlaced {
            order_id,
            trades,
            status,
        } => Ok(HttpResponse::Ok().json(serde_json::json!({
            "order_id": order_id.to_string(),
            "status": status,
            "trades_count": trades.len(),
            "trades": trades,
        }))),
        OrderBookResponse::Error(err) => Err(err.into()),
        OrderBookResponse::Rejected(msg) => Err(ApiError::Forbidden(msg)),
        _ => Err(ApiError::InternalError(
            "unexpected response from orderbook".to_string(),
        )),
    }
}

#[delete("/cancel")]
pub async fn cancel_order(
    user: AuthUser,
    state: web::Data<AppState>,
    body: web::Json<CancelOrderRequest>,
) -> Result<impl Responder, ApiError> {
    let order_id = Uuid::parse_str(&body.order_id)
        .map_err(|_| ApiError::BadRequest("invalid order_id format".to_string()))?;

    let response = state
        .send_command(|response_tx| OrderBookCommand::CancelOrder {
            user_id: user.id,
            order_id,
            response_tx,
        })
        .await?;

    match response {
        OrderBookResponse::OrderCancelled { order_id, success } => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "order_id": order_id.to_string(),
                "cancelled": success,
            })))
        }
        OrderBookResponse::Error(err) => Err(err.into()),
        OrderBookResponse::Rejected(msg) => Err(ApiError::Forbidden(msg)),
        _ => Err(ApiError::InternalError(
            "unexpected response from orderbook".to_string(),
        )),
    }
}
