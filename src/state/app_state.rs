use crate::messages::OrderBookCommand;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Application state shared across Actix-web workers
/// - `orderbook_tx`: sender end of the mpsc channel to communicate with OrderBook engine
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
}
