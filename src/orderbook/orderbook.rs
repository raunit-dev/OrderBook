use crate::orderbook::{OrderBookError, PriceLevel};
use crate::types::{Order, OrderSide, Price, Quantity, UserBalance};
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

pub type DepthLevels = Vec<(Price, Quantity)>;

/// Where a resting order lives in the book, indexed from `orders` for O(log n) cancellation.
/// The actual Order value lives inside the corresponding PriceLevel's VecDeque — one source of truth.
#[derive(Debug, Clone, Copy)]
pub struct OrderLocation {
    pub side: OrderSide,
    pub price: Price,
}

pub struct OrderBook {
    pub bids: BTreeMap<Reverse<Price>, PriceLevel>,
    pub asks: BTreeMap<Price, PriceLevel>,
    /// Index by order id → where it lives. NOT a second copy of the Order.
    pub orders: HashMap<Uuid, OrderLocation>,
    pub user_balances: HashMap<Uuid, UserBalance>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders: HashMap::new(),
            user_balances: HashMap::new(),
        }
    }

    pub fn best_bid(&self) -> Option<Price> {
        self.bids.keys().next().map(|r| r.0)
    }

    pub fn best_ask(&self) -> Option<Price> {
        self.asks.keys().next().copied()
    }

    /// Add a limit order to the book, placing it at the tail of its price-level FIFO queue.
    pub fn add_order(&mut self, order: Order) {
        let order_id = order.id;
        let side = order.side;
        let price = order.price.expect("limit order must have a price");

        match side {
            OrderSide::Buy => {
                self.bids
                    .entry(Reverse(price))
                    .or_default()
                    .enqueue_order(order);
            }
            OrderSide::Sell => {
                self.asks.entry(price).or_default().enqueue_order(order);
            }
        }

        self.orders.insert(order_id, OrderLocation { side, price });
    }

    /// Remove an order from both the price-level queue and the location index.
    pub fn cancel_order(&mut self, order_id: Uuid) -> Result<Order, OrderBookError> {
        let loc = self
            .orders
            .remove(&order_id)
            .ok_or(OrderBookError::OrderNotFound(order_id))?;

        let order = match loc.side {
            OrderSide::Buy => {
                let level = self
                    .bids
                    .get_mut(&Reverse(loc.price))
                    .ok_or(OrderBookError::OrderNotFound(order_id))?;
                let order = level
                    .dequeue_order_by_id(order_id)
                    .ok_or(OrderBookError::OrderNotFound(order_id))?;
                if level.is_empty() {
                    self.bids.remove(&Reverse(loc.price));
                }
                order
            }
            OrderSide::Sell => {
                let level = self
                    .asks
                    .get_mut(&loc.price)
                    .ok_or(OrderBookError::OrderNotFound(order_id))?;
                let order = level
                    .dequeue_order_by_id(order_id)
                    .ok_or(OrderBookError::OrderNotFound(order_id))?;
                if level.is_empty() {
                    self.asks.remove(&loc.price);
                }
                order
            }
        };

        Ok(order)
    }

    pub fn get_or_create_balance(&mut self, user_id: Uuid) -> &mut UserBalance {
        self.user_balances
            .entry(user_id)
            .or_insert_with(|| UserBalance::new(user_id))
    }

    pub fn get_user_balance(&self, user_id: Uuid) -> Option<&UserBalance> {
        self.user_balances.get(&user_id)
    }

    pub fn add_funds(&mut self, user_id: Uuid, currency: &str, amount: f64) {
        self.get_or_create_balance(user_id)
            .add_balance(currency, amount);
    }

    pub fn deduct_balance(
        &mut self,
        user_id: Uuid,
        currency: &str,
        amount: f64,
    ) -> Result<(), OrderBookError> {
        let balance = self
            .user_balances
            .get_mut(&user_id)
            .ok_or(OrderBookError::UserNotFound(user_id))?;
        balance.subtract_balance(currency, amount)
    }

    pub fn credit_balance(&mut self, user_id: Uuid, currency: &str, amount: f64) {
        self.get_or_create_balance(user_id)
            .add_balance(currency, amount);
    }

    pub fn get_depth(&self, levels: usize) -> (DepthLevels, DepthLevels) {
        let bids = self
            .bids
            .iter()
            .take(levels)
            .map(|(Reverse(price), level)| (*price, level.total_volume))
            .collect();

        let asks = self
            .asks
            .iter()
            .take(levels)
            .map(|(price, level)| (*price, level.total_volume))
            .collect();

        (bids, asks)
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}
