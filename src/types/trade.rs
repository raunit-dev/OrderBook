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
