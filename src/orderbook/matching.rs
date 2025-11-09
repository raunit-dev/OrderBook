use crate::orderbook::OrderBook;
use crate::types::{Order, OrderSide, OrderType, Price, Quantity, Trade};
use std::cmp::Reverse;

impl OrderBook {
    /// Match a new incoming order against the orderbook
    /// Returns a vector of trades executed
    pub fn match_order(&mut self, mut order: Order) -> Result<Vec<Trade>, String> {
        let mut trades = Vec::new();

        match order.order_type {
            OrderType::Limit => {
                trades = self.match_limit_order(&mut order)?;

                // If order still has remaining quantity, add it to the book
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

    /// Match a limit order
    fn match_limit_order(&mut self, taker_order: &mut Order) -> Result<Vec<Trade>, String> {
        let mut trades = Vec::new();
        let taker_price = taker_order.price.ok_or("Limit order must have price")?;

        match taker_order.side {
            OrderSide::Buy => {
                // Match against asks (sell orders)
                // We can buy if ask price <= our bid price
                while !taker_order.is_fully_filled() {
                    // Get best ask (lowest sell price)
                    let best_ask_price = match self.best_ask() {
                        Some(price) => price,
                        None => break, // No more sellers
                    };

                    // Check if we can match
                    if best_ask_price > taker_price {
                        break; // Ask price too high
                    }

                    // Get the price level and create trade
                    let (trade, fill_quantity, maker_id, maker_filled) = {
                        let price_level = self.asks.get_mut(&best_ask_price).unwrap();

                        // Match with the first order in the level (FIFO)
                        if let Some(maker_order) = price_level.front_mut() {
                            let fill_qty = std::cmp::min(
                                taker_order.remaining_quantity,
                                maker_order.remaining_quantity,
                            );

                            let maker_id = maker_order.id;
                            let maker_user_id = maker_order.user_id;

                            // Update orders
                            maker_order.fill(fill_qty);
                            taker_order.fill(fill_qty);

                            let maker_filled = maker_order.is_fully_filled();

                            // Update price level volume
                            price_level.update_volume(fill_qty);

                            // Create trade
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
                        // Update balances (no longer holding price_level reference)
                        self.execute_trade_settlement(&trade, OrderSide::Buy)?;

                        trades.push(trade);

                        // Update order index for maker
                        if maker_filled {
                            self.orders.remove(&maker_id);
                        } else if let Some(price_level) = self.asks.get(&best_ask_price) {
                            if let Some(maker_order) = price_level.front() {
                                self.orders.insert(maker_id, maker_order.clone());
                            }
                        }
                    }

                    // Remove filled orders from the front
                    if let Some(price_level) = self.asks.get_mut(&best_ask_price) {
                        price_level.pop_if_filled();

                        // Remove empty price level
                        if price_level.is_empty() {
                            self.asks.remove(&best_ask_price);
                        }
                    }
                }
            }
            OrderSide::Sell => {
                // Match against bids (buy orders)
                // We can sell if bid price >= our ask price
                while !taker_order.is_fully_filled() {
                    // Get best bid (highest buy price)
                    let best_bid_price = match self.best_bid() {
                        Some(price) => price,
                        None => break, // No more buyers
                    };

                    // Check if we can match
                    if best_bid_price < taker_price {
                        break; // Bid price too low
                    }

                    // Get the price level and create trade
                    let (trade, fill_quantity, maker_id, maker_filled) = {
                        let price_level = self.bids.get_mut(&Reverse(best_bid_price)).unwrap();

                        // Match with the first order in the level (FIFO)
                        if let Some(maker_order) = price_level.front_mut() {
                            let fill_qty = std::cmp::min(
                                taker_order.remaining_quantity,
                                maker_order.remaining_quantity,
                            );

                            let maker_id = maker_order.id;
                            let maker_user_id = maker_order.user_id;

                            // Update orders
                            maker_order.fill(fill_qty);
                            taker_order.fill(fill_qty);

                            let maker_filled = maker_order.is_fully_filled();

                            // Update price level volume
                            price_level.update_volume(fill_qty);

                            // Create trade
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
                        // Update balances (no longer holding price_level reference)
                        self.execute_trade_settlement(&trade, OrderSide::Sell)?;

                        trades.push(trade);

                        // Update order index for maker
                        if maker_filled {
                            self.orders.remove(&maker_id);
                        } else if let Some(price_level) = self.bids.get(&Reverse(best_bid_price)) {
                            if let Some(maker_order) = price_level.front() {
                                self.orders.insert(maker_id, maker_order.clone());
                            }
                        }
                    }

                    // Remove filled orders from the front
                    if let Some(price_level) = self.bids.get_mut(&Reverse(best_bid_price)) {
                        price_level.pop_if_filled();

                        // Remove empty price level
                        if price_level.is_empty() {
                            self.bids.remove(&Reverse(best_bid_price));
                        }
                    }
                }
            }
        }

        Ok(trades)
    }

    /// Match a market order (executes immediately at best available price)
    fn match_market_order(&mut self, taker_order: &mut Order) -> Result<Vec<Trade>, String> {
        let mut trades = Vec::new();

        match taker_order.side {
            OrderSide::Buy => {
                // Buy at best ask prices
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
            }
            OrderSide::Sell => {
                // Sell at best bid prices
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
            }
        }

        Ok(trades)
    }

    /// Execute trade settlement (update user balances)
    /// For BTC/USD market:
    /// - Buy: Deduct USD from buyer, Credit BTC to buyer, Deduct BTC from seller, Credit USD to seller
    /// - Sell: Deduct BTC from seller, Credit USD to seller, Deduct USD from buyer, Credit BTC to buyer
    fn execute_trade_settlement(
        &mut self,
        trade: &Trade,
        taker_side: OrderSide,
    ) -> Result<(), String> {
        let btc_amount = trade.quantity.to_f64();
        let usd_amount = trade.price.to_f64() * btc_amount;

        match taker_side {
            OrderSide::Buy => {
                // Taker is buying BTC with USD
                // Deduct USD from taker (buyer)
                self.deduct_balance(trade.taker_user_id, "USD", usd_amount)?;
                // Credit BTC to taker (buyer)
                self.credit_balance(trade.taker_user_id, "BTC", btc_amount);

                // Deduct BTC from maker (seller)
                self.deduct_balance(trade.maker_user_id, "BTC", btc_amount)?;
                // Credit USD to maker (seller)
                self.credit_balance(trade.maker_user_id, "USD", usd_amount);
            }
            OrderSide::Sell => {
                // Taker is selling BTC for USD
                // Deduct BTC from taker (seller)
                self.deduct_balance(trade.taker_user_id, "BTC", btc_amount)?;
                // Credit USD to taker (seller)
                self.credit_balance(trade.taker_user_id, "USD", usd_amount);

                // Deduct USD from maker (buyer)
                self.deduct_balance(trade.maker_user_id, "USD", usd_amount)?;
                // Credit BTC to maker (buyer)
                self.credit_balance(trade.maker_user_id, "BTC", btc_amount);
            }
        }

        Ok(())
    }
}
