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
    pub fn new_limit(
        user_id: Uuid,
        side: OrderSide,
        price: Price,
        quantity: Quantity,
    ) -> Self {
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
