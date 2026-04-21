use crate::orderbook::{OrderBookError, PriceLevel};
use crate::types::{Order, OrderSide, Price, Quantity, UserBalance};
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

pub type DepthLevels = Vec<(Price, Quantity)>;

pub struct OrderBook {
    pub bids: BTreeMap<Reverse<Price>, PriceLevel>,
    pub asks: BTreeMap<Price, PriceLevel>,
    pub orders: HashMap<Uuid, Order>,
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
        let price = order.price.expect("limit order must have a price");

        match order.side {
            OrderSide::Buy => {
                self.bids
                    .entry(Reverse(price))
                    .or_default()
                    .enqueue_order(order.clone());
            }
            OrderSide::Sell => {
                self.asks
                    .entry(price)
                    .or_default()
                    .enqueue_order(order.clone());
            }
        }

        self.orders.insert(order_id, order);
    }

    /// Remove an order from both the price-level queue and the global order map.
    pub fn cancel_order(&mut self, order_id: Uuid) -> Result<Order, OrderBookError> {
        let order = self
            .orders
            .remove(&order_id)
            .ok_or(OrderBookError::OrderNotFound(order_id))?;
        let price = order.price.ok_or(OrderBookError::MissingPrice)?;

        match order.side {
            OrderSide::Buy => {
                if let Some(level) = self.bids.get_mut(&Reverse(price)) {
                    level.dequeue_order_by_id(order_id);
                    if level.is_empty() {
                        self.bids.remove(&Reverse(price));
                    }
                }
            }
            OrderSide::Sell => {
                if let Some(level) = self.asks.get_mut(&price) {
                    level.dequeue_order_by_id(order_id);
                    if level.is_empty() {
                        self.asks.remove(&price);
                    }
                }
            }
        }

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
