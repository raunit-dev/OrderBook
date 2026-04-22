use crate::orderbook::{OrderBook, OrderBookError};
use crate::types::{Order, OrderSide, Price, Trade};
use std::cmp::Reverse;

impl OrderBook {
    /// Match a limit order against resting orders on the opposite side.
    ///
    /// Mutates `taker.remaining_quantity` as fills happen. Caller is responsible for
    /// resting the unfilled remainder (if any) on the book via `add_order`.
    pub fn match_limit_order(&mut self, taker: &mut Order) -> Result<Vec<Trade>, OrderBookError> {
        let taker_limit = taker.price.ok_or(OrderBookError::MissingPrice)?;
        let taker_side = taker.side;
        let mut trades = Vec::new();

        while !taker.is_fully_filled() {
            // while there's still quantity to fill
            let best = match self.best_opposite(taker_side) {
                Some(p) => p,
                None => break, // if no opposite side exists, stop
            };

            if !crosses(taker_side, best, taker_limit) {
                break; // if price dont cross stop
            }

            match self.fill_once_at_level(taker_side, best, taker)? {
                Some(trade) => trades.push(trade),
                None => break, // otherwise fill one at the best price
            }
        }

        Ok(trades)
    }

    /// Match a market order against resting liquidity on the opposite side.
    ///
    /// Returns `Err(InsufficientLiquidity)` only if **no** fills were possible.
    /// A partial fill (some quantity matched, book then ran dry) returns `Ok(trades)` —
    /// any already-settled fills are kept rather than rolled back.
    pub fn match_market_order(&mut self, taker: &mut Order) -> Result<Vec<Trade>, OrderBookError> {
        let taker_side = taker.side;
        let mut trades = Vec::new();

        while !taker.is_fully_filled() {
            let best = match self.best_opposite(taker_side) {
                Some(p) => p,
                None => {
                    if trades.is_empty() {
                        return Err(OrderBookError::InsufficientLiquidity);
                    }
                    break;
                }
            };

            match self.fill_once_at_level(taker_side, best, taker)? {
                Some(trade) => trades.push(trade),
                None => break,
            }
        }

        Ok(trades)
    }

    /// Best resting price on the side **opposite** to the taker:
    /// lowest ask for a buy taker, highest bid for a sell taker.
    fn best_opposite(&self, taker_side: OrderSide) -> Option<Price> {
        match taker_side {
            OrderSide::Buy => self.best_ask(),
            OrderSide::Sell => self.best_bid(),
        }
    }

    /// Consume one maker from the top of the given opposite-side price level,
    /// fill the taker by the overlap, settle balances, and clean up if fully filled.
    ///
    /// Returns `Some(trade)` on a successful match, `None` if the level was unexpectedly empty.
    fn fill_once_at_level(
        &mut self,
        taker_side: OrderSide,
        level_price: Price,
        taker: &mut Order,
    ) -> Result<Option<Trade>, OrderBookError> {
        // Scoped mutable borrow of the level so we can release it before touching other state.
        let (trade, maker_id, maker_filled) = {
            let level = match taker_side {
                OrderSide::Buy => self.asks.get_mut(&level_price),
                OrderSide::Sell => self.bids.get_mut(&Reverse(level_price)),
            };
            let level = match level {
                Some(l) => l,
                None => return Ok(None),
            };

            let maker = match level.front_mut() {
                Some(m) => m,
                None => return Ok(None),
            };

            let fill_qty = std::cmp::min(taker.remaining_quantity, maker.remaining_quantity);
            let maker_id = maker.id;
            let maker_user_id = maker.user_id;

            maker.fill(fill_qty);
            taker.fill(fill_qty);
            let maker_filled = maker.is_fully_filled();
            level.update_volume(fill_qty);

            let trade = Trade::new(
                maker_id,
                taker.id,
                maker_user_id,
                taker.user_id,
                level_price,
                fill_qty,
            );

            (trade, maker_id, maker_filled)
        };

        self.execute_trade_settlement(&trade, taker_side)?;

        if maker_filled {
            self.orders.remove(&maker_id);
            self.remove_filled_front(taker_side, level_price);
        }

        Ok(Some(trade))
    }

    /// After a maker is fully filled, pop it off the front of its level and drop the level
    /// entirely if empty.
    fn remove_filled_front(&mut self, taker_side: OrderSide, level_price: Price) {
        match taker_side {
            OrderSide::Buy => {
                if let Some(level) = self.asks.get_mut(&level_price) {
                    level.pop_if_filled();
                    if level.is_empty() {
                        self.asks.remove(&level_price);
                    }
                }
            }
            OrderSide::Sell => {
                if let Some(level) = self.bids.get_mut(&Reverse(level_price)) {
                    level.pop_if_filled();
                    if level.is_empty() {
                        self.bids.remove(&Reverse(level_price));
                    }
                }
            }
        }
    }
}

/// Does `book_price` cross `taker_limit` for the given taker side?
/// - Buy taker: book ask must be ≤ my limit (willing to pay up to that).
/// - Sell taker: book bid must be ≥ my limit (willing to accept down to that).
fn crosses(taker_side: OrderSide, book_price: Price, taker_limit: Price) -> bool {
    match taker_side {
        OrderSide::Buy => book_price <= taker_limit,
        OrderSide::Sell => book_price >= taker_limit,
    }
}
