use crate::orderbook::{OrderBook, OrderBookError};
use crate::types::{Currency, OrderSide, Trade};

impl OrderBook {
    /// Move balances between the trade's buyer and seller according to the taker side.
    /// All amounts are in integer minor units — no f64 math.
    pub(crate) fn execute_trade_settlement(
        &mut self,
        trade: &Trade,
        taker_side: OrderSide,
    ) -> Result<(), OrderBookError> {
        let btc_amount = trade.quantity.raw(); // satoshis
        let usd_amount = trade.price.times_quantity(trade.quantity); // USD millionths

        match taker_side {
            OrderSide::Buy => {
                self.deduct_balance(trade.taker_user_id, Currency::Usd, usd_amount)?;
                self.credit_balance(trade.taker_user_id, Currency::Btc, btc_amount);
                self.deduct_balance(trade.maker_user_id, Currency::Btc, btc_amount)?;
                self.credit_balance(trade.maker_user_id, Currency::Usd, usd_amount);
            }
            OrderSide::Sell => {
                self.deduct_balance(trade.taker_user_id, Currency::Btc, btc_amount)?;
                self.credit_balance(trade.taker_user_id, Currency::Usd, usd_amount);
                self.deduct_balance(trade.maker_user_id, Currency::Usd, usd_amount)?;
                self.credit_balance(trade.maker_user_id, Currency::Btc, btc_amount);
            }
        }

        Ok(())
    }
}
