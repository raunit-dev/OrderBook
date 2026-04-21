use crate::messages::{OrderBookCommand, OrderBookResponse};
use crate::utils::error::ApiError;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

/// Application state shared across Actix-web workers.
/// - `orderbook_tx`: sender end of the mpsc channel into the OrderBook engine
/// - `db`: Postgres connection pool (source of truth for user accounts)
#[derive(Clone)]
pub struct AppState {
    pub orderbook_tx: Arc<mpsc::Sender<OrderBookCommand>>,
    pub db: PgPool,
}

impl AppState {
    pub fn new(orderbook_tx: mpsc::Sender<OrderBookCommand>, db: PgPool) -> Self {
        AppState {
            orderbook_tx: Arc::new(orderbook_tx),
            db,
        }
    }

    /// Send a command to the engine and await its response.
    ///
    /// The caller supplies a builder closure that receives a fresh `oneshot::Sender`
    /// and wraps it into the appropriate command variant — this keeps the mpsc send
    /// + oneshot await boilerplate out of every handler.
    pub async fn send_command<F>(&self, build: F) -> Result<OrderBookResponse, ApiError>
    where
        F: FnOnce(oneshot::Sender<OrderBookResponse>) -> OrderBookCommand,
    {
        let (response_tx, response_rx) = oneshot::channel();
        self.orderbook_tx
            .send(build(response_tx))
            .await
            .map_err(|_| ApiError::InternalError("orderbook engine is unavailable".to_string()))?;
        response_rx.await.map_err(|_| {
            ApiError::InternalError("orderbook engine dropped the response".to_string())
        })
    }
}
