use crate::orderbook::OrderBook;
use crate::types::{Order, OrderSide, Trade};
use std::cmp::Reverse;

impl OrderBook {
    pub(crate) fn match_market_order(
        &mut self,
        taker_order: &mut Order,
    ) -> Result<Vec<Trade>, String> {
        let mut trades = Vec::new();

        match taker_order.side {
            OrderSide::Buy => {
                trades = self.match_market_buy(taker_order)?;
            }
            OrderSide::Sell => {
                trades = self.match_market_sell(taker_order)?;
            }
        }

        Ok(trades)
    }

    // Match a market buy order (taker buys at best ask prices)
    fn match_market_buy(&mut self, taker_order: &mut Order) -> Result<Vec<Trade>, String> {
        let mut trades = Vec::new();

        while !taker_order.is_fully_filled() {
            let best_ask_price = match self.best_ask() {
                Some(price) => price,
                None => return Err("Insufficient liquidity for market order".to_string()),
            };

            let (trade, maker_id, maker_filled) = {
                let price_level = self.asks.get_mut(&best_ask_price).unwrap();

                if let Some(maker_order) = price_level.front_mut() {
                    let fill_quantity = std::cmp::min(
                        taker_order.remaining_quantity,
                        maker_order.remaining_quantity,
                    );

                    let maker_id = maker_order.id;
                    let maker_user_id = maker_order.user_id;

                    maker_order.fill(fill_quantity);
                    taker_order.fill(fill_quantity);

                    let maker_filled = maker_order.is_fully_filled();
                    price_level.update_volume(fill_quantity);

                    let trade = Trade::new(
                        maker_id,
                        taker_order.id,
                        maker_user_id,
                        taker_order.user_id,
                        best_ask_price,
                        fill_quantity,
                    );

                    (Some(trade), maker_id, maker_filled)
                } else {
                    (None, uuid::Uuid::nil(), false)
                }
            };

            if let Some(trade) = trade {
                self.execute_trade_settlement(&trade, OrderSide::Buy)?;
                trades.push(trade);

                if maker_filled {
                    self.orders.remove(&maker_id);
                } else if let Some(price_level) = self.asks.get(&best_ask_price) {
                    if let Some(maker_order) = price_level.front() {
                        self.orders.insert(maker_id, maker_order.clone());
                    }
                }
            }

            if let Some(price_level) = self.asks.get_mut(&best_ask_price) {
                price_level.pop_if_filled();

                if price_level.is_empty() {
                    self.asks.remove(&best_ask_price);
                }
            }
        }

        Ok(trades)
    }

    fn match_market_sell(&mut self, taker_order: &mut Order) -> Result<Vec<Trade>, String> {
        let mut trades = Vec::new();

        while !taker_order.is_fully_filled() {
            let best_bid_price = match self.best_bid() {
                Some(price) => price,
                None => return Err("Insufficient liquidity for market order".to_string()),
            };

            let (trade, maker_id, maker_filled) = {
                let price_level = self.bids.get_mut(&Reverse(best_bid_price)).unwrap();

                if let Some(maker_order) = price_level.front_mut() {
                    let fill_quantity = std::cmp::min(
                        taker_order.remaining_quantity,
                        maker_order.remaining_quantity,
                    );

                    let maker_id = maker_order.id;
                    let maker_user_id = maker_order.user_id;

                    maker_order.fill(fill_quantity);
                    taker_order.fill(fill_quantity);

                    let maker_filled = maker_order.is_fully_filled();
                    price_level.update_volume(fill_quantity);

                    let trade = Trade::new(
                        maker_id,
                        taker_order.id,
                        maker_user_id,
                        taker_order.user_id,
                        best_bid_price,
                        fill_quantity,
                    );

                    (Some(trade), maker_id, maker_filled)
                } else {
                    (None, uuid::Uuid::nil(), false)
                }
            };

            if let Some(trade) = trade {
                self.execute_trade_settlement(&trade, OrderSide::Sell)?;
                trades.push(trade);

                if maker_filled {
                    self.orders.remove(&maker_id);
                } else if let Some(price_level) = self.bids.get(&Reverse(best_bid_price)) {
                    if let Some(maker_order) = price_level.front() {
                        self.orders.insert(maker_id, maker_order.clone());
                    }
                }
            }

            if let Some(price_level) = self.bids.get_mut(&Reverse(best_bid_price)) {
                price_level.pop_if_filled();

                if price_level.is_empty() {
                    self.bids.remove(&Reverse(best_bid_price));
                }
            }
        }

        Ok(trades)
    }
}
