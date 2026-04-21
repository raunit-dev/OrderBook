use crate::types::{Order, Price, Quantity};
use std::collections::VecDeque;
use uuid::Uuid;
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
    // Enqueue an order to the back of the FIFO queue at this price level
    pub fn enqueue_order(&mut self, order: Order) {
        self.total_volume += order.remaining_quantity;
        self.orders.push_back(order);
    }
    // Remove a specific order from the queue by its ID
    pub fn dequeue_order_by_id(&mut self, order_id: Uuid) -> Option<Order> {
        if let Some(pos) = self.orders.iter().position(|o| o.id == order_id) {
            let order = self.orders.remove(pos)?;
            self.total_volume -= order.remaining_quantity;
            Some(order)
        } else {
            None
        }
    }
    pub fn update_volume(&mut self, quantity_filled: Quantity) {
        self.total_volume -= quantity_filled;
    }
    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }
    pub fn front(&self) -> Option<&Order> {
        self.orders.front()
    }
    pub fn front_mut(&mut self) -> Option<&mut Order> {
        self.orders.front_mut()
    }
    pub fn pop_if_filled(&mut self) -> Option<Order> {
        if let Some(order) = self.orders.front() {
            if order.is_fully_filled() {
                return self.orders.pop_front();
            }
        }
        None
    }
}
