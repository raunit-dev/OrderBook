use crate::types::{Order, Quantity};
use std::collections::VecDeque;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct PriceLevel {
    pub orders: VecDeque<Order>,
    pub total_volume: Quantity,
}

impl PriceLevel {
    /// Enqueue an order to the back of the FIFO queue at this price level
    pub fn enqueue_order(&mut self, order: Order) {
        self.total_volume += order.remaining_quantity;
        self.orders.push_back(order);
    }

    /// Remove a specific order from the queue by its ID
    pub fn dequeue_order_by_id(&mut self, order_id: Uuid) -> Option<Order> {
        let pos = self.orders.iter().position(|o| o.id == order_id)?;
        let order = self.orders.remove(pos)?;
        self.total_volume -= order.remaining_quantity;
        Some(order)
    }

    pub fn update_volume(&mut self, quantity_filled: Quantity) {
        self.total_volume -= quantity_filled;
    }

    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    pub fn front_mut(&mut self) -> Option<&mut Order> {
        self.orders.front_mut()
    }

    pub fn pop_if_filled(&mut self) -> Option<Order> {
        if self.orders.front()?.is_fully_filled() {
            self.orders.pop_front()
        } else {
            None
        }
    }
}
