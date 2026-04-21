use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum OrderBookError {
    #[error("insufficient {currency} balance")]
    InsufficientBalance { currency: String },

    #[error("user {0} not found")]
    UserNotFound(Uuid),

    #[error("currency '{0}' not recognized")]
    UnknownCurrency(String),

    #[error("order {0} not found")]
    OrderNotFound(Uuid),

    #[error("order is missing a price")]
    MissingPrice,

    #[error("insufficient liquidity")]
    InsufficientLiquidity,
}
