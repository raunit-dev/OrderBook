use super::{Price, Quantity};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Option<Price>, // None for market orders
    pub original_quantity: Quantity,
    pub remaining_quantity: Quantity,
    pub status: OrderStatus,
    pub timestamp: DateTime<Utc>,
}

impl Order {
    pub fn new_limit(user_id: Uuid, side: OrderSide, price: Price, quantity: Quantity) -> Self {
        Order {
            id: Uuid::new_v4(),
            user_id,
            side,
            order_type: OrderType::Limit,
            price: Some(price),
            original_quantity: quantity,
            remaining_quantity: quantity,
            status: OrderStatus::Open,
            timestamp: Utc::now(),
        }
    }

    pub fn new_market(user_id: Uuid, side: OrderSide, quantity: Quantity) -> Self {
        Order {
            id: Uuid::new_v4(),
            user_id,
            side,
            order_type: OrderType::Market,
            price: None,
            original_quantity: quantity,
            remaining_quantity: quantity,
            status: OrderStatus::Open,
            timestamp: Utc::now(),
        }
    }

    pub fn is_fully_filled(&self) -> bool {
        self.remaining_quantity.is_zero()
    }

    pub fn fill(&mut self, quantity: Quantity) {
        self.remaining_quantity -= quantity;

        if self.is_fully_filled() {
            self.status = OrderStatus::Filled;
        } else {
            self.status = OrderStatus::PartiallyFilled;
        }
    }

    pub fn cancel(&mut self) {
        self.status = OrderStatus::Cancelled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_limit_order() {
        let user_id = Uuid::new_v4();
        let price = Price::new(10000);
        let quantity = Quantity::new(5);

        let order = Order::new_limit(user_id, OrderSide::Buy, price, quantity);

        assert_eq!(order.user_id, user_id);
        assert_eq!(order.side, OrderSide::Buy);
        assert_eq!(order.order_type, OrderType::Limit);
        assert_eq!(order.price, Some(price));
        assert_eq!(order.original_quantity, quantity);
        assert_eq!(order.remaining_quantity, quantity);
        assert_eq!(order.status, OrderStatus::Open);
    }

    #[test]
    fn test_new_market_order() {
        let user_id = Uuid::new_v4();
        let quantity = Quantity::new(5);

        let order = Order::new_market(user_id, OrderSide::Sell, quantity);

        assert_eq!(order.user_id, user_id);
        assert_eq!(order.side, OrderSide::Sell);
        assert_eq!(order.order_type, OrderType::Market);
        assert_eq!(order.price, None);
        assert_eq!(order.original_quantity, quantity);
        assert_eq!(order.remaining_quantity, quantity);
        assert_eq!(order.status, OrderStatus::Open);
    }

    #[test]
    fn test_partial_fill() {
        let user_id = Uuid::new_v4();
        let price = Price::new(10000);
        let quantity = Quantity::new(10);

        let mut order = Order::new_limit(user_id, OrderSide::Buy, price, quantity);

        order.fill(Quantity::new(3));

        assert_eq!(order.remaining_quantity, Quantity::new(7));
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        assert!(!order.is_fully_filled());
    }

    #[test]
    fn test_full_fill() {
        let user_id = Uuid::new_v4();
        let price = Price::new(10000);
        let quantity = Quantity::new(10);

        let mut order = Order::new_limit(user_id, OrderSide::Buy, price, quantity);

        order.fill(Quantity::new(10));

        assert_eq!(order.remaining_quantity, Quantity::new(0));
        assert_eq!(order.status, OrderStatus::Filled);
        assert!(order.is_fully_filled());
    }

    #[test]
    fn test_multiple_partial_fills() {
        let user_id = Uuid::new_v4();
        let price = Price::new(10000);
        let quantity = Quantity::new(10);

        let mut order = Order::new_limit(user_id, OrderSide::Buy, price, quantity);

        order.fill(Quantity::new(3));
        assert_eq!(order.remaining_quantity, Quantity::new(7));
        assert_eq!(order.status, OrderStatus::PartiallyFilled);

        order.fill(Quantity::new(4));
        assert_eq!(order.remaining_quantity, Quantity::new(3));
        assert_eq!(order.status, OrderStatus::PartiallyFilled);

        order.fill(Quantity::new(3));
        assert_eq!(order.remaining_quantity, Quantity::new(0));
        assert_eq!(order.status, OrderStatus::Filled);
        assert!(order.is_fully_filled());
    }

    #[test]
    fn test_cancel_order() {
        let user_id = Uuid::new_v4();
        let price = Price::new(10000);
        let quantity = Quantity::new(10);

        let mut order = Order::new_limit(user_id, OrderSide::Buy, price, quantity);

        order.cancel();

        assert_eq!(order.status, OrderStatus::Cancelled);
        assert_eq!(order.remaining_quantity, quantity);
    }

    #[test]
    fn test_cancel_partially_filled_order() {
        let user_id = Uuid::new_v4();
        let price = Price::new(10000);
        let quantity = Quantity::new(10);

        let mut order = Order::new_limit(user_id, OrderSide::Buy, price, quantity);

        order.fill(Quantity::new(4));
        assert_eq!(order.status, OrderStatus::PartiallyFilled);

        order.cancel();
        assert_eq!(order.status, OrderStatus::Cancelled);
        assert_eq!(order.remaining_quantity, Quantity::new(6));
    }

    #[test]
    fn test_order_side_variants() {
        let user_id = Uuid::new_v4();
        let price = Price::new(10000);
        let quantity = Quantity::new(5);

        let buy_order = Order::new_limit(user_id, OrderSide::Buy, price, quantity);
        assert_eq!(buy_order.side, OrderSide::Buy);

        let sell_order = Order::new_limit(user_id, OrderSide::Sell, price, quantity);
        assert_eq!(sell_order.side, OrderSide::Sell);
    }

    #[test]
    fn test_is_fully_filled_zero_quantity() {
        let user_id = Uuid::new_v4();
        let price = Price::new(10000);
        let quantity = Quantity::new(5);

        let mut order = Order::new_limit(user_id, OrderSide::Buy, price, quantity);

        assert!(!order.is_fully_filled());

        order.fill(quantity);
        assert!(order.is_fully_filled());
    }

    #[test]
    fn test_order_id_uniqueness() {
        let user_id = Uuid::new_v4();
        let price = Price::new(10000);
        let quantity = Quantity::new(5);

        let order1 = Order::new_limit(user_id, OrderSide::Buy, price, quantity);
        let order2 = Order::new_limit(user_id, OrderSide::Buy, price, quantity);

        assert_ne!(order1.id, order2.id);
    }

    #[test]
    fn test_order_timestamp() {
        let user_id = Uuid::new_v4();
        let price = Price::new(10000);
        let quantity = Quantity::new(5);

        let before = Utc::now();
        let order = Order::new_limit(user_id, OrderSide::Buy, price, quantity);
        let after = Utc::now();

        assert!(order.timestamp >= before);
        assert!(order.timestamp <= after);
    }
}
