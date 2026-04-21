use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::collections::HashMap;

use crate::messages::{OrderBookCommand, OrderBookResponse};
use crate::state::AppState;
use crate::types::Currency;
use crate::utils::error::ApiError;
use crate::utils::AuthUser;

#[derive(Debug, Deserialize)]
pub struct OnrampRequest {
    pub currency: String,
    pub amount: f64,
}

#[get("/balance")]
pub async fn get_balance(
    user: AuthUser,
    state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let response = state
        .send_command(|response_tx| OrderBookCommand::GetUserBalance {
            user_id: user.id,
            response_tx,
        })
        .await?;

    match response {
        OrderBookResponse::UserBalance { balance } => {
            // Render balances as human-readable decimals at the API edge.
            let display: HashMap<String, f64> = balance
                .balances
                .iter()
                .map(|(c, raw)| (c.to_string(), c.to_f64(*raw)))
                .collect();

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "user_id": balance.user_id.to_string(),
                "balances": display,
            })))
        }
        OrderBookResponse::Error(err) => Err(err.into()),
        _ => Err(ApiError::InternalError(
            "unexpected response from orderbook".to_string(),
        )),
    }
}

#[post("/onramp")]
pub async fn onramp(
    user: AuthUser,
    state: web::Data<AppState>,
    body: web::Json<OnrampRequest>,
) -> Result<impl Responder, ApiError> {
    let currency = Currency::parse(&body.currency)
        .ok_or_else(|| ApiError::BadRequest("currency must be 'USD' or 'BTC'".to_string()))?;

    if body.amount <= 0.0 {
        return Err(ApiError::BadRequest("amount must be positive".to_string()));
    }

    let amount_raw = currency.to_raw(body.amount);

    let response = state
        .send_command(|response_tx| OrderBookCommand::AddFunds {
            user_id: user.id,
            currency,
            amount: amount_raw,
            response_tx,
        })
        .await?;

    match response {
        OrderBookResponse::FundsAdded {
            user_id,
            currency,
            new_balance,
        } => Ok(HttpResponse::Ok().json(serde_json::json!({
            "user_id": user_id.to_string(),
            "currency": currency.to_string(),
            "new_balance": currency.to_f64(new_balance),
        }))),
        _ => Err(ApiError::InternalError(
            "unexpected response from orderbook".to_string(),
        )),
    }
}
