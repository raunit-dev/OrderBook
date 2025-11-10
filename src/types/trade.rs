use super::{Price, Quantity};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub maker_order_id: Uuid,
    pub taker_order_id: Uuid,
    pub maker_user_id: Uuid,
    pub taker_user_id: Uuid,
    pub price: Price,
    pub quantity: Quantity,
    pub timestamp: DateTime<Utc>,
}

impl Trade {
    pub fn new(
        maker_order_id: Uuid,
        taker_order_id: Uuid,
        maker_user_id: Uuid,
        taker_user_id: Uuid,
        price: Price,
        quantity: Quantity,
    ) -> Self {
        Trade {
            id: Uuid::new_v4(),
            maker_order_id,
            taker_order_id,
            maker_user_id,
            taker_user_id,
            price,
            quantity,
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new_trade() {
        let maker_order_id = Uuid::new_v4();
        let taker_order_id = Uuid::new_v4();
        let maker_user_id = Uuid::new_v4();
        let taker_user_id = Uuid::new_v4();
        let price = Price::new(10000);
        let quantity = Quantity::new(10);

        let trade = Trade::new(
            maker_order_id,
            taker_order_id,
            maker_user_id,
            taker_user_id,
            price,
            quantity,
        );

        assert_eq!(trade.maker_order_id, maker_order_id);
        assert_eq!(trade.taker_order_id, taker_order_id);
        assert_eq!(trade.maker_user_id, maker_user_id);
        assert_eq!(trade.taker_user_id, taker_user_id);
        assert_eq!(trade.price, price);
        assert_eq!(trade.quantity, quantity);
    }

    #[test]
    fn test_trade_unique_ids() {
        let maker_order_id = Uuid::new_v4();
        let taker_order_id = Uuid::new_v4();
        let maker_user_id = Uuid::new_v4();
        let taker_user_id = Uuid::new_v4();
        let price = Price::new(5000);
        let quantity = Quantity::new(20);

        let trade1 = Trade::new(
            maker_order_id,
            taker_order_id,
            maker_user_id,
            taker_user_id,
            price,
            quantity,
        );
        let trade2 = Trade::new(
            maker_order_id,
            taker_order_id,
            maker_user_id,
            taker_user_id,
            price,
            quantity,
        );

        assert_ne!(trade1.id, trade2.id);
    }
}
