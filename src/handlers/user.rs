use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::messages::{OrderBookCommand, OrderBookResponse};
use crate::state::AppState;
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
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "user_id": balance.user_id.to_string(),
                "balances": balance.balances,
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
    if body.currency != "USD" && body.currency != "BTC" {
        return Err(ApiError::BadRequest(
            "currency must be 'USD' or 'BTC'".to_string(),
        ));
    }
    if body.amount <= 0.0 {
        return Err(ApiError::BadRequest("amount must be positive".to_string()));
    }

    let response = state
        .send_command(|response_tx| OrderBookCommand::AddFunds {
            user_id: user.id,
            currency: body.currency.clone(),
            amount: body.amount,
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
            "currency": currency,
            "new_balance": new_balance,
        }))),
        _ => Err(ApiError::InternalError(
            "unexpected response from orderbook".to_string(),
        )),
    }
}
