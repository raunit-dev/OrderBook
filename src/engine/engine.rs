use crate::messages::{OrderBookCommand, OrderBookResponse};
use crate::orderbook::{OrderBook, OrderBookError};
use crate::types::Order;
use crate::types::OrderSide::*;
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

                // Reserve funds before the order touches the book.
                let reservation = match side {
                    Buy => orderbook.deduct_balance(
                        user_id,
                        "USD",
                        price.to_f64() * quantity.to_f64(),
                    ),
                    Sell => orderbook.deduct_balance(user_id, "BTC", quantity.to_f64()),
                };
                if let Err(e) = reservation {
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

                // Market orders don't pre-reserve — settlement deducts per fill.

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

                    // Refund the reserved balance on the unfilled remainder.
                    match cancelled.side {
                        crate::types::OrderSide::Buy => {
                            if let Some(price) = cancelled.price {
                                let usd_refund =
                                    price.to_f64() * cancelled.remaining_quantity.to_f64();
                                orderbook.credit_balance(user_id, "USD", usd_refund);
                            }
                        }
                        crate::types::OrderSide::Sell => {
                            let btc_refund = cancelled.remaining_quantity.to_f64();
                            orderbook.credit_balance(user_id, "BTC", btc_refund);
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
                    let _ = response_tx
                        .send(OrderBookResponse::Error(OrderBookError::UserNotFound(user_id)));
                }
            },

            OrderBookCommand::AddFunds {
                user_id,
                currency,
                amount,
                response_tx,
            } => {
                orderbook.add_funds(user_id, &currency, amount);
                let new_balance = orderbook
                    .get_or_create_balance(user_id)
                    .get_balance(&currency);

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
