use crate::orderbook::OrderBook;
use crate::types::{OrderSide, Trade};

impl OrderBook {
    pub(crate) fn execute_trade_settlement(
        &mut self,
        trade: &Trade,
        taker_side: OrderSide,
    ) -> Result<(), String> {
        let btc_amount = trade.quantity.to_f64();
        let usd_amount = trade.price.to_f64() * btc_amount;

        match taker_side {
            OrderSide::Buy => {
                self.deduct_balance(trade.taker_user_id, "USD", usd_amount)?;
                self.credit_balance(trade.taker_user_id, "BTC", btc_amount);
                self.deduct_balance(trade.maker_user_id, "BTC", btc_amount)?;
                self.credit_balance(trade.maker_user_id, "USD", usd_amount);
            }
            OrderSide::Sell => {
                self.deduct_balance(trade.taker_user_id, "BTC", btc_amount)?;
                self.credit_balance(trade.taker_user_id, "USD", usd_amount);
                self.deduct_balance(trade.maker_user_id, "USD", usd_amount)?;
                self.credit_balance(trade.maker_user_id, "BTC", btc_amount);
            }
        }

        Ok(())
    }
}
