use crate::messages::{OrderBookCommand, OrderBookResponse};
use crate::orderbook::{OrderBook, OrderBookError};
use crate::types::{Currency, Order, OrderSide::*};
use tokio::sync::mpsc;

pub async fn run_orderbook_engine(mut rx: mpsc::Receiver<OrderBookCommand>) {
    let mut orderbook = OrderBook::new();

    println!("OrderBook engine started and listening for commands...");

    while let Some(command) = rx.recv().await {
        match command {
            OrderBookCommand::PlaceLimitOrder {
                user_id,
                side,
                price,
                quantity,
                response_tx,
            } => {
                let mut order = Order::new_limit(user_id, side, price, quantity);
                let order_id = order.id;

                // Reserve funds in integer minor units before the order touches the book.
                let (currency, amount) = match side {
                    Buy => (Currency::Usd, price.times_quantity(quantity)),
                    Sell => (Currency::Btc, quantity.raw()),
                };
                if let Err(e) = orderbook.deduct_balance(user_id, currency, amount) {
                    let _ = response_tx.send(OrderBookResponse::Error(e));
                    continue;
                }

                match orderbook.match_limit_order(&mut order) {
                    Ok(trades) => {
                        let fully_filled = order.is_fully_filled();
                        let matched = !trades.is_empty();

                        let status = if fully_filled {
                            "Filled"
                        } else if matched {
                            orderbook.add_order(order);
                            "Partially filled, remainder on book"
                        } else {
                            orderbook.add_order(order);
                            "Added to book"
                        };

                        let _ = response_tx.send(OrderBookResponse::OrderPlaced {
                            order_id,
                            trades,
                            status: status.to_string(),
                        });
                    }
                    Err(e) => {
                        let _ = response_tx.send(OrderBookResponse::Error(e));
                    }
                }
            }

            OrderBookCommand::PlaceMarketOrder {
                user_id,
                side,
                quantity,
                response_tx,
            } => {
                let mut order = Order::new_market(user_id, side, quantity);
                let order_id = order.id;

                match orderbook.match_market_order(&mut order) {
                    Ok(trades) => {
                        let status = if order.is_fully_filled() {
                            "Filled"
                        } else {
                            "Partially filled"
                        };

                        let _ = response_tx.send(OrderBookResponse::OrderPlaced {
                            order_id,
                            trades,
                            status: status.to_string(),
                        });
                    }
                    Err(e) => {
                        let _ = response_tx.send(OrderBookResponse::Error(e));
                    }
                }
            }

            OrderBookCommand::CancelOrder {
                user_id,
                order_id,
                response_tx,
            } => match orderbook.cancel_order(order_id) {
                Ok(cancelled) => {
                    if cancelled.user_id != user_id {
                        let _ = response_tx.send(OrderBookResponse::Rejected(
                            "not authorized to cancel this order".to_string(),
                        ));
                        continue;
                    }

                    // Refund reserved balance on the unfilled remainder (integer math).
                    match cancelled.side {
                        Buy => {
                            if let Some(price) = cancelled.price {
                                let refund = price.times_quantity(cancelled.remaining_quantity);
                                orderbook.credit_balance(user_id, Currency::Usd, refund);
                            }
                        }
                        Sell => {
                            orderbook.credit_balance(
                                user_id,
                                Currency::Btc,
                                cancelled.remaining_quantity.raw(),
                            );
                        }
                    }

                    let _ = response_tx.send(OrderBookResponse::OrderCancelled {
                        order_id,
                        success: true,
                    });
                }
                Err(e) => {
                    let _ = response_tx.send(OrderBookResponse::Error(e));
                }
            },

            OrderBookCommand::GetOrderBook { depth, response_tx } => {
                let (bids, asks) = orderbook.get_depth(depth);
                let _ = response_tx.send(OrderBookResponse::OrderBookDepth { bids, asks });
            }

            OrderBookCommand::GetUserBalance {
                user_id,
                response_tx,
            } => match orderbook.get_user_balance(user_id) {
                Some(balance) => {
                    let _ = response_tx.send(OrderBookResponse::UserBalance {
                        balance: balance.clone(),
                    });
                }
                None => {
                    let _ = response_tx.send(OrderBookResponse::Error(
                        OrderBookError::UserNotFound(user_id),
                    ));
                }
            },

            OrderBookCommand::AddFunds {
                user_id,
                currency,
                amount,
                response_tx,
            } => {
                orderbook.add_funds(user_id, currency, amount);
                let new_balance = orderbook.get_or_create_balance(user_id).get(currency);

                let _ = response_tx.send(OrderBookResponse::FundsAdded {
                    user_id,
                    currency,
                    new_balance,
                });
            }
        }
    }

    println!("OrderBook engine shutting down...");
}
