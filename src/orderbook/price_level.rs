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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::OrderSide;
    use uuid::Uuid;

    fn mk_order(q: u64) -> Order {
        let user_id = Uuid::new_v4();
        let price = Price::new(10_000);
        Order::new_limit(user_id, OrderSide::Buy, price, Quantity::new(q))
    }
 
    #[test]
    fn enqueue_order_updates_volume_and_fifo() {
        let price = Price::new(10_000);
        let mut level = PriceLevel::new(price);

        let o1 = mk_order(3);
        let o2 = mk_order(7);

        level.enqueue_order(o1.clone());
        level.enqueue_order(o2.clone());

        assert_eq!(level.price, price);
        assert_eq!(level.total_volume, Quantity::new(3 + 7));
        assert_eq!(level.front().unwrap().id, o1.id);
        assert!(!level.is_empty());
    }

    #[test]
    fn dequeue_order_by_id_updates_volume() {
        let price = Price::new(10_000);
        let mut level = PriceLevel::new(price);

        let o1 = mk_order(5);
        let o2 = mk_order(2);
        let o3 = mk_order(4);

        let o1_id = o1.id;
        let o2_id = o2.id;
        let o3_id = o3.id;

        level.enqueue_order(o1.clone());
        level.enqueue_order(o2.clone());
        level.enqueue_order(o3.clone());

        assert_eq!(level.total_volume, Quantity::new(5 + 2 + 4));

        let removed = level.dequeue_order_by_id(o2_id).expect("should remove");
        assert_eq!(removed.id, o2_id);
        assert_eq!(level.total_volume, Quantity::new(5 + 4));

        assert_eq!(level.front().unwrap().id, o1_id);

        let before = level.total_volume;
        assert!(level.dequeue_order_by_id(Uuid::new_v4()).is_none());
        assert_eq!(level.total_volume, before);

        level.dequeue_order_by_id(o1_id);
        level.dequeue_order_by_id(o3_id);
        assert!(level.is_empty());
        assert_eq!(level.total_volume, Quantity::new(0));
    }
}
