use crate::types::{Order, Price, Quantity};
use std::collections::VecDeque;
use uuid::Uuid;

/// Represents all orders at a specific price level
/// Uses VecDeque for FIFO (First-In-First-Out) ordering
#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: Price,
    pub orders: VecDeque<Order>,
    pub total_volume: Quantity,
}

impl PriceLevel {
    pub fn new(price: Price) -> Self {
        PriceLevel {
            price,
            orders: VecDeque::new(),
            total_volume: Quantity::new(0),
        }
    }

    /// Add an order to the back of the queue (FIFO)
    pub fn add_order(&mut self, order: Order) {
        self.total_volume += order.remaining_quantity;
        self.orders.push_back(order);
    }

    /// Remove an order by ID
    pub fn remove_order(&mut self, order_id: Uuid) -> Option<Order> {
        if let Some(pos) = self.orders.iter().position(|o| o.id == order_id) {
            let order = self.orders.remove(pos)?;
            self.total_volume -= order.remaining_quantity;
            Some(order)
        } else {
            None
        }
    }

    /// Update total volume after an order is partially filled
    pub fn update_volume(&mut self, quantity_filled: Quantity) {
        self.total_volume -= quantity_filled;
    }

    /// Check if this price level has no orders
    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    /// Get the first order (FIFO)
    pub fn front(&self) -> Option<&Order> {
        self.orders.front()
    }

    /// Get mutable reference to the first order
    pub fn front_mut(&mut self) -> Option<&mut Order> {
        self.orders.front_mut()
    }

    /// Remove and return the first order if it's fully filled
    pub fn pop_if_filled(&mut self) -> Option<Order> {
        if let Some(order) = self.orders.front() {
            if order.is_fully_filled() {
                return self.orders.pop_front();
            }
        }
        None
    }
}
