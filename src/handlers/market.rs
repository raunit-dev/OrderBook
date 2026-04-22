use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::messages::{OrderBookCommand, OrderBookResponse};
use crate::state::AppState;
use crate::utils::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct OrderBookQuery {
    pub depth: Option<usize>,
}

#[get("/orderbook")]
pub async fn get_orderbook(
    state: web::Data<AppState>,
    query: web::Query<OrderBookQuery>,
) -> Result<impl Responder, ApiError> {
    let depth = query.depth.unwrap_or(10);

    let response = state
        .send_command(|response_tx| OrderBookCommand::GetOrderBook {
            depth,
            response_tx,
        })
        .await?;

    match response {
        OrderBookResponse::OrderBookDepth { bids, asks } => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "bids": bids.iter().map(|(price, qty)| {
                    serde_json::json!({
                        "price": price.to_f64(),
                        "quantity": qty.to_f64(),
                    })
                }).collect::<Vec<_>>(),
                "asks": asks.iter().map(|(price, qty)| {
                    serde_json::json!({
                        "price": price.to_f64(),
                        "quantity": qty.to_f64(),
                    })
                }).collect::<Vec<_>>(),
            })))
        }
        _ => Err(ApiError::InternalError(
            "unexpected response from orderbook".to_string(),
        )),
    }
}

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "demo-orderbook"
    }))
}
