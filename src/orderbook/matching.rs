use crate::orderbook::OrderBook;
use crate::types::{Order, OrderSide, OrderType, Quantity, Trade};
use std::cmp::Reverse;

impl OrderBook {
    /// Main entry point for matching an order against the orderbook
    pub fn match_order(&mut self, mut order: Order) -> Result<Vec<Trade>, String> {
        let mut trades = Vec::new();

        match order.order_type {
            OrderType::Limit => {
                trades = self.match_limit_order(&mut order)?;
                if !order.is_fully_filled() {
                    self.add_order(order);
                }
            }
            OrderType::Market => {
                trades = self.match_market_order(&mut order)?;
            }
        }

        Ok(trades)
    }

    fn match_limit_order(&mut self, taker_order: &mut Order) -> Result<Vec<Trade>, String> {
        let mut trades = Vec::new();
        let taker_price = taker_order.price.ok_or("Limit order must have price")?;

        match taker_order.side {
            OrderSide::Buy => {
                while !taker_order.is_fully_filled() {
                    let best_ask_price = match self.best_ask() {
                        Some(price) => price,
                        None => break,
                    };

                    if best_ask_price > taker_price {
                        break;
                    }

                    let (trade, _fill_quantity, maker_id, maker_filled) = {
                        let price_level = self.asks.get_mut(&best_ask_price).unwrap();

                        if let Some(maker_order) = price_level.front_mut() {
                            let fill_qty = std::cmp::min(
                                taker_order.remaining_quantity,
                                maker_order.remaining_quantity,
                            );

                            let maker_id = maker_order.id;
                            let maker_user_id = maker_order.user_id;

                            maker_order.fill(fill_qty);
                            taker_order.fill(fill_qty);

                            let maker_filled = maker_order.is_fully_filled();
                            price_level.update_volume(fill_qty);

                            let trade = Trade::new(
                                maker_id,
                                taker_order.id,
                                maker_user_id,
                                taker_order.user_id,
                                best_ask_price,
                                fill_qty,
                            );

                            (Some(trade), fill_qty, maker_id, maker_filled)
                        } else {
                            (None, Quantity::new(0), uuid::Uuid::nil(), false)
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
            }
            OrderSide::Sell => {
                while !taker_order.is_fully_filled() {
                    let best_bid_price = match self.best_bid() {
                        Some(price) => price,
                        None => break,
                    };

                    if best_bid_price < taker_price {
                        break;
                    }

                    let (trade, _fill_quantity, maker_id, maker_filled) = {
                        let price_level = self.bids.get_mut(&Reverse(best_bid_price)).unwrap();

                        if let Some(maker_order) = price_level.front_mut() {
                            let fill_qty = std::cmp::min(
                                taker_order.remaining_quantity,
                                maker_order.remaining_quantity,
                            );

                            let maker_id = maker_order.id;
                            let maker_user_id = maker_order.user_id;

                            maker_order.fill(fill_qty);
                            taker_order.fill(fill_qty);

                            let maker_filled = maker_order.is_fully_filled();
                            price_level.update_volume(fill_qty);

                            let trade = Trade::new(
                                maker_id,
                                taker_order.id,
                                maker_user_id,
                                taker_order.user_id,
                                best_bid_price,
                                fill_qty,
                            );

                            (Some(trade), fill_qty, maker_id, maker_filled)
                        } else {
                            (None, Quantity::new(0), uuid::Uuid::nil(), false)
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
            }
        }

        Ok(trades)
    }
}
