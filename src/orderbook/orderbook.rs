use crate::orderbook::PriceLevel;
use crate::types::{Order, OrderSide, Price, Quantity, Trade, UserBalance};
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

/// Main OrderBook structure
/// - Bids: BTreeMap with Reverse<Price> for descending order (highest price first)
/// - Asks: BTreeMap with Price for ascending order (lowest price first)
pub struct OrderBook {
    pub bids: BTreeMap<Reverse<Price>, PriceLevel>, // Buy orders (descending)
    pub asks: BTreeMap<Price, PriceLevel>,          // Sell orders (ascending)
    pub orders: HashMap<Uuid, Order>,               // Order lookup by ID
    pub user_balances: HashMap<Uuid, UserBalance>,  // User balances
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

    /// Get best bid price (highest buy price)
    pub fn best_bid(&self) -> Option<Price> {
        self.bids.keys().next().map(|r| r.0)
    }

    /// Get best ask price (lowest sell price)
    pub fn best_ask(&self) -> Option<Price> {
        self.asks.keys().next().copied()
    }

    /// Add an order to the orderbook
    pub fn add_order(&mut self, order: Order) {
        let order_id = order.id;
        let price = order.price.expect("Limit order must have price");

        match order.side {
            OrderSide::Buy => {
                self.bids
                    .entry(Reverse(price))
                    .or_insert_with(|| PriceLevel::new(price))
                    .add_order(order.clone());
            }
            OrderSide::Sell => {
                self.asks
                    .entry(price)
                    .or_insert_with(|| PriceLevel::new(price))
                    .add_order(order.clone());
            }
        }

        self.orders.insert(order_id, order);
    }

    /// Cancel an order by ID
    pub fn cancel_order(&mut self, order_id: Uuid) -> Result<Order, String> {
        let order = self
            .orders
            .remove(&order_id)
            .ok_or("Order not found")?;

        let price = order.price.ok_or("Order has no price")?;

        match order.side {
            OrderSide::Buy => {
                if let Some(level) = self.bids.get_mut(&Reverse(price)) {
                    level.remove_order(order_id);
                    if level.is_empty() {
                        self.bids.remove(&Reverse(price));
                    }
                }
            }
            OrderSide::Sell => {
                if let Some(level) = self.asks.get_mut(&price) {
                    level.remove_order(order_id);
                    if level.is_empty() {
                        self.asks.remove(&price);
                    }
                }
            }
        }

        Ok(order)
    }

    /// Get order by ID
    pub fn get_order(&self, order_id: Uuid) -> Option<&Order> {
        self.orders.get(&order_id)
    }

    /// Get or create user balance
    pub fn get_or_create_balance(&mut self, user_id: Uuid) -> &mut UserBalance {
        self.user_balances
            .entry(user_id)
            .or_insert_with(|| UserBalance::new(user_id))
    }

    /// Get user balance
    pub fn get_user_balance(&self, user_id: Uuid) -> Option<&UserBalance> {
        self.user_balances.get(&user_id)
    }

    /// Add funds to user balance
    pub fn add_funds(&mut self, user_id: Uuid, currency: &str, amount: f64) {
        let balance = self.get_or_create_balance(user_id);
        balance.add_balance(currency, amount);
    }

    /// Check if user has sufficient balance
    pub fn has_sufficient_balance(
        &self,
        user_id: Uuid,
        currency: &str,
        required_amount: f64,
    ) -> bool {
        if let Some(balance) = self.user_balances.get(&user_id) {
            balance.get_balance(currency) >= required_amount
        } else {
            false
        }
    }

    /// Deduct balance from user
    pub fn deduct_balance(
        &mut self,
        user_id: Uuid,
        currency: &str,
        amount: f64,
    ) -> Result<(), String> {
        let balance = self
            .user_balances
            .get_mut(&user_id)
            .ok_or("User not found")?;
        balance.subtract_balance(currency, amount)
    }

    /// Credit balance to user
    pub fn credit_balance(&mut self, user_id: Uuid, currency: &str, amount: f64) {
        let balance = self.get_or_create_balance(user_id);
        balance.add_balance(currency, amount);
    }

    /// Get market depth (top N levels for bids and asks)
    pub fn get_depth(&self, levels: usize) -> (Vec<(Price, Quantity)>, Vec<(Price, Quantity)>) {
        let bids: Vec<(Price, Quantity)> = self
            .bids
            .iter()
            .take(levels)
            .map(|(Reverse(price), level)| (*price, level.total_volume))
            .collect();

        let asks: Vec<(Price, Quantity)> = self
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
