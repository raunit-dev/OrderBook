use crate::orderbook::OrderBookError;
use crate::types::{Currency, OrderSide, Price, Quantity, Trade, UserBalance};
use tokio::sync::oneshot;
use uuid::Uuid;

// Commands sent from HTTP handlers to the OrderBook engine thread
pub enum OrderBookCommand {
    // Order commands
    PlaceLimitOrder {
        user_id: Uuid,
        side: OrderSide,
        price: Price,
        quantity: Quantity,
        response_tx: oneshot::Sender<OrderBookResponse>,
    },
    PlaceMarketOrder {
        user_id: Uuid,
        side: OrderSide,
        quantity: Quantity,
        response_tx: oneshot::Sender<OrderBookResponse>,
    },
    CancelOrder {
        user_id: Uuid,
        order_id: Uuid,
        response_tx: oneshot::Sender<OrderBookResponse>,
    },

    // Query commands
    GetOrderBook {
        depth: usize,
        response_tx: oneshot::Sender<OrderBookResponse>,
    },
    GetUserBalance {
        user_id: Uuid,
        response_tx: oneshot::Sender<OrderBookResponse>,
    },

    // Balance commands
    AddFunds {
        user_id: Uuid,
        currency: Currency,
        amount: u64, // integer minor units of `currency`
        response_tx: oneshot::Sender<OrderBookResponse>,
    },
}

/// Responses sent from OrderBook engine thread back to HTTP handlers
#[derive(Debug)]
pub enum OrderBookResponse {
    // Order responses
    OrderPlaced {
        order_id: Uuid,
        trades: Vec<Trade>,
        status: String,
    },
    OrderCancelled {
        order_id: Uuid,
        success: bool,
    },

    // Query responses
    OrderBookDepth {
        bids: Vec<(Price, Quantity)>,
        asks: Vec<(Price, Quantity)>,
    },
    UserBalance {
        balance: UserBalance,
    },

    // Balance responses
    FundsAdded {
        user_id: Uuid,
        currency: Currency,
        new_balance: u64, // integer minor units
    },

    // Typed engine error (e.g. InsufficientBalance, OrderNotFound, …)
    Error(OrderBookError),

    // Non-engine failures surfaced from the command-handling path
    // (e.g. "not authorized to cancel this order").
    Rejected(String),
}
