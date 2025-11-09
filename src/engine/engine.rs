use crate::messages::{OrderBookCommand, OrderBookResponse};
use crate::orderbook::OrderBook;
use crate::types::{Order, OrderType};
use tokio::sync::mpsc;

/// Starts the OrderBook engine in a separate thread
/// Receives commands via mpsc channel and processes them sequentially
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
                let order = Order::new_limit(user_id, side, price, quantity);
                let order_id = order.id;

                // Check balance before placing order
                let required_balance = match side {
                    crate::types::OrderSide::Buy => {
                        // Need USD to buy BTC
                        let usd_needed = price.to_f64() * quantity.to_f64();
                        if !orderbook.has_sufficient_balance(user_id, "USD", usd_needed) {
                            let _ = response_tx.send(OrderBookResponse::Error {
                                message: "Insufficient USD balance".to_string(),
                            });
                            continue;
                        }
                        // Reserve USD
                        if let Err(e) = orderbook.deduct_balance(user_id, "USD", usd_needed) {
                            let _ = response_tx.send(OrderBookResponse::Error {
                                message: format!("Failed to reserve USD: {}", e),
                            });
                            continue;
                        }
                    }
                    crate::types::OrderSide::Sell => {
                        // Need BTC to sell
                        let btc_needed = quantity.to_f64();
                        if !orderbook.has_sufficient_balance(user_id, "BTC", btc_needed) {
                            let _ = response_tx.send(OrderBookResponse::Error {
                                message: "Insufficient BTC balance".to_string(),
                            });
                            continue;
                        }
                        // Reserve BTC
                        if let Err(e) = orderbook.deduct_balance(user_id, "BTC", btc_needed) {
                            let _ = response_tx.send(OrderBookResponse::Error {
                                message: format!("Failed to reserve BTC: {}", e),
                            });
                            continue;
                        }
                    }
                };

                match orderbook.match_order(order) {
                    Ok(trades) => {
                        let status = if trades.is_empty() {
                            "Added to book".to_string()
                        } else {
                            "Matched".to_string()
                        };

                        let _ = response_tx.send(OrderBookResponse::OrderPlaced {
                            order_id,
                            trades,
                            status,
                        });
                    }
                    Err(e) => {
                        let _ = response_tx.send(OrderBookResponse::Error {
                            message: format!("Failed to place order: {}", e),
                        });
                    }
                }
            }

            OrderBookCommand::PlaceMarketOrder {
                user_id,
                side,
                quantity,
                response_tx,
            } => {
                let order = Order::new_market(user_id, side, quantity);
                let order_id = order.id;

                // For market orders, we need to check balance based on estimated execution
                // For simplicity, we'll skip balance check here and let matching engine handle it
                // In production, you'd estimate the required balance based on orderbook depth

                match orderbook.match_order(order) {
                    Ok(trades) => {
                        let status = if trades.is_empty() {
                            "No liquidity".to_string()
                        } else {
                            "Filled".to_string()
                        };

                        let _ = response_tx.send(OrderBookResponse::OrderPlaced {
                            order_id,
                            trades,
                            status,
                        });
                    }
                    Err(e) => {
                        let _ = response_tx.send(OrderBookResponse::Error {
                            message: format!("Failed to place market order: {}", e),
                        });
                    }
                }
            }

            OrderBookCommand::CancelOrder {
                user_id,
                order_id,
                response_tx,
            } => {
                match orderbook.cancel_order(order_id) {
                    Ok(cancelled_order) => {
                        // Verify ownership
                        if cancelled_order.user_id != user_id {
                            let _ = response_tx.send(OrderBookResponse::Error {
                                message: "Not authorized to cancel this order".to_string(),
                            });
                            continue;
                        }

                        // Refund reserved balance
                        match cancelled_order.side {
                            crate::types::OrderSide::Buy => {
                                // Refund USD
                                if let Some(price) = cancelled_order.price {
                                    let usd_refund = price.to_f64()
                                        * cancelled_order.remaining_quantity.to_f64();
                                    orderbook.credit_balance(user_id, "USD", usd_refund);
                                }
                            }
                            crate::types::OrderSide::Sell => {
                                // Refund BTC
                                let btc_refund = cancelled_order.remaining_quantity.to_f64();
                                orderbook.credit_balance(user_id, "BTC", btc_refund);
                            }
                        }

                        let _ = response_tx.send(OrderBookResponse::OrderCancelled {
                            order_id,
                            success: true,
                        });
                    }
                    Err(e) => {
                        let _ = response_tx.send(OrderBookResponse::Error {
                            message: format!("Failed to cancel order: {}", e),
                        });
                    }
                }
            }

            OrderBookCommand::GetOrderBook { depth, response_tx } => {
                let (bids, asks) = orderbook.get_depth(depth);
                let _ = response_tx.send(OrderBookResponse::OrderBookDepth { bids, asks });
            }

            OrderBookCommand::GetUserBalance {
                user_id,
                response_tx,
            } => {
                if let Some(balance) = orderbook.get_user_balance(user_id) {
                    let _ =
                        response_tx.send(OrderBookResponse::UserBalance {
                            balance: balance.clone(),
                        });
                } else {
                    let _ = response_tx.send(OrderBookResponse::Error {
                        message: "User not found".to_string(),
                    });
                }
            }

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
