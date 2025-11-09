use crate::messages::OrderBookCommand;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Application state shared across Actix-web workers
/// Contains the sender end of the mpsc channel to communicate with OrderBook engine
#[derive(Clone)]
pub struct AppState {
    pub orderbook_tx: Arc<mpsc::Sender<OrderBookCommand>>,
}

impl AppState {
    pub fn new(orderbook_tx: mpsc::Sender<OrderBookCommand>) -> Self {
        AppState {
            orderbook_tx: Arc::new(orderbook_tx),
        }
    }
}
