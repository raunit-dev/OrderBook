use crate::types::Currency;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum OrderBookError {
    #[error("insufficient {currency} balance")]
    InsufficientBalance { currency: Currency },

    #[error("user {0} not found")]
    UserNotFound(Uuid),

    #[error("order {0} not found")]
    OrderNotFound(Uuid),

    #[error("order is missing a price")]
    MissingPrice,

    #[error("insufficient liquidity")]
    InsufficientLiquidity,
}
