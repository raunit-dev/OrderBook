Overview

  This is the heart of your orderbook system - a single-threaded engine that processes all orderbook operations sequentially. It runs in its own async task and communicates with HTTP
  handlers via channels.

  ---
  Function Signature

  pub async fn run_orderbook_engine(mut rx: mpsc::Receiver<OrderBookCommand>)

  - async fn - Runs asynchronously using Tokio
  - mut rx - Mutable receiver to read commands from the channel
  - mpsc::Receiver - Multi-Producer, Single-Consumer channel (many handlers can send, one engine receives)

  ---
  Initialization

  let mut orderbook = OrderBook::new();
  println!("OrderBook engine started and listening for commands...");

  Creates the in-memory orderbook:
  - Empty bids BTreeMap
  - Empty asks BTreeMap
  - Empty orders HashMap
  - Empty user_balances HashMap

  This orderbook lives ONLY in this function - isolated from the rest of the system!

  ---
  Main Event Loop

  while let Some(command) = rx.recv().await {
      match command { ... }
  }

  How it works:
  1. .recv().await - Waits for next command from the channel (blocks until one arrives)
  2. Some(command) - Got a command, process it
  3. None - Channel closed, exit loop and shut down
  4. match command - Pattern match on command type and execute

  This is a SINGLE THREAD - processes one command at a time, sequentially!

  ---
  Command Processing - Let me explain each one:

  1. PlaceLimitOrder Command

  OrderBookCommand::PlaceLimitOrder {
      user_id,
      side,
      price,
      quantity,
      response_tx,  // ← Oneshot channel to send response back
  } => {

  Step 1: Create the Order

  let order = Order::new_limit(user_id, side, price, quantity);
  let order_id = order.id;

  Creates a new limit order with:
  - Unique UUID
  - User who placed it
  - Buy or Sell side
  - Price and quantity
  - Status: Open
  - Timestamp: now

  Step 2: Balance Check & Reservation

  For BUY orders (buying BTC with USD):
  crate::types::OrderSide::Buy => {
      let usd_needed = price.to_f64() * quantity.to_f64();

      // Check: Does user have enough USD?
      if !orderbook.has_sufficient_balance(user_id, "USD", usd_needed) {
          let _ = response_tx.send(OrderBookResponse::Error {
              message: "Insufficient USD balance".to_string(),
          });
          continue; // ← Skip to next command, don't place order
      }

      // Reserve USD (lock the funds)
      if let Err(e) = orderbook.deduct_balance(user_id, "USD", usd_needed) {
          let _ = response_tx.send(OrderBookResponse::Error {
              message: format!("Failed to reserve USD: {}", e),
          });
          continue;
      }
  }

  Example:
  - User wants to buy 10 BTC @ $50,000 each
  - Needs: $500,000 USD
  - Check if user has $500,000 in their balance
  - If yes: Deduct $500,000 (reserve it)
  - If no: Return error, don't place order

  For SELL orders (selling BTC for USD):
  crate::types::OrderSide::Sell => {
      let btc_needed = quantity.to_f64();

      // Check: Does user have enough BTC to sell?
      if !orderbook.has_sufficient_balance(user_id, "BTC", btc_needed) {
          let _ = response_tx.send(OrderBookResponse::Error {
              message: "Insufficient BTC balance".to_string(),
          });
          continue;
      }

      // Reserve BTC (lock the coins)
      if let Err(e) = orderbook.deduct_balance(user_id, "BTC", btc_needed) {
          let _ = response_tx.send(OrderBookResponse::Error {
              message: format!("Failed to reserve BTC: {}", e),
          });
          continue;
      }
  }

  Why reserve funds?
  - Prevents double-spending
  - User can't use the same funds for multiple orders
  - Funds released when order filled or cancelled

  Step 3: Match Order

  match orderbook.match_order(order) {
      Ok(trades) => {
          let status = if trades.is_empty() {
              "Added to book".to_string()  // No matches, sitting in book
          } else {
              "Matched".to_string()  // Executed some/all trades
          };

          let _ = response_tx.send(OrderBookResponse::OrderPlaced {
              order_id,
              trades,  // Vec of executed trades
              status,
          });
      }
      Err(e) => {
          let _ = response_tx.send(OrderBookResponse::Error {
              message: format!("Failed to place order: {}", e),
          });
      }
  }

  What happens:
  1. Try to match against opposite side of orderbook
  2. Execute any possible trades
  3. If not fully filled, add remainder to orderbook
  4. Send response back with trade results

  Response sent via oneshot channel - handler is waiting on the other end!

  ---
  2. PlaceMarketOrder Command

  OrderBookCommand::PlaceMarketOrder {
      user_id,
      side,
      quantity,
      response_tx,
  } => {
      let order = Order::new_market(user_id, side, quantity);
      let order_id = order.id;

  Key Difference from Limit Order:
  // For market orders, we need to check balance based on estimated execution
  // For simplicity, we'll skip balance check here and let matching engine handle it
  // In production, you'd estimate the required balance based on orderbook depth

  Why skip balance check?
  - Market orders execute at unknown prices
  - Don't know total cost until matching
  - Would need to pre-calculate from orderbook depth
  - Current implementation: Let matching engine fail if insufficient balance

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

  Market orders:
  - Never added to book
  - Either fill completely or error
  - Execute immediately at best prices

  ---
  3. CancelOrder Command

  OrderBookCommand::CancelOrder {
      user_id,
      order_id,
      response_tx,
  } => {

  Step 1: Find and Remove Order

  match orderbook.cancel_order(order_id) {
      Ok(cancelled_order) => {
          // Got the cancelled order

  This removes order from:
  - BTreeMap (bids/asks)
  - VecDeque (price level queue)
  - HashMap (order lookup)

  Step 2: Verify Ownership

          if cancelled_order.user_id != user_id {
              let _ = response_tx.send(OrderBookResponse::Error {
                  message: "Not authorized to cancel this order".to_string(),
              });
              continue;
          }

  Security check: Only the user who placed the order can cancel it!

  Step 3: Refund Reserved Funds

  For BUY orders:
  crate::types::OrderSide::Buy => {
      if let Some(price) = cancelled_order.price {
          let usd_refund = price.to_f64()
              * cancelled_order.remaining_quantity.to_f64();
          orderbook.credit_balance(user_id, "USD", usd_refund);
      }
  }

  Example:
  - Order: Buy 10 BTC @ $50,000
  - Matched: 3 BTC (7 remaining)
  - Reserved: $500,000
  - Used: $150,000 (3 BTC × $50k)
  - Refund: $350,000 (7 BTC × $50k)

  For SELL orders:
  crate::types::OrderSide::Sell => {
      let btc_refund = cancelled_order.remaining_quantity.to_f64();
      orderbook.credit_balance(user_id, "BTC", btc_refund);
  }

  Refunds the reserved BTC back to user's balance.

  Step 4: Send Response

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

  ---
  4. GetOrderBook Command

  OrderBookCommand::GetOrderBook { depth, response_tx } => {
      let (bids, asks) = orderbook.get_depth(depth);
      let _ = response_tx.send(OrderBookResponse::OrderBookDepth { bids, asks });
  }

  Simple query:
  - Get top N price levels from both sides
  - Return snapshot of current orderbook
  - No state changes

  Example response:
  OrderBookDepth {
      bids: [(Price($100), Quantity(50)), (Price($99), Quantity(30))],
      asks: [(Price($101), Quantity(40)), (Price($102), Quantity(25))],
  }

  ---
  5. GetUserBalance Command

  OrderBookCommand::GetUserBalance {
      user_id,
      response_tx,
  } => {
      if let Some(balance) = orderbook.get_user_balance(user_id) {
          let _ = response_tx.send(OrderBookResponse::UserBalance {
              balance: balance.clone(),
          });
      } else {
          let _ = response_tx.send(OrderBookResponse::Error {
              message: "User not found".to_string(),
          });
      }
  }

  Returns user's current balance:
  UserBalance {
      user_id: uuid,
      balances: {
          "USD": 10000.0,
          "BTC": 2.5
      }
  }

  ---
  6. AddFunds Command

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

  Simulates depositing funds:
  - Add USD or BTC to user's balance
  - Used for testing or "onramp" functionality
  - Returns new total balance

  ---
  Shutdown

  println!("OrderBook engine shutting down...");

  When the channel closes (all senders dropped), the loop exits and function returns.

  ---
  Complete Flow Example

  Scenario: User places limit order

  1. HTTP Handler creates command:
  let (response_tx, response_rx) = oneshot::channel();

  orderbook_tx.send(OrderBookCommand::PlaceLimitOrder {
      user_id: user_uuid,
      side: OrderSide::Buy,
      price: Price::from_f64(50000.0),
      quantity: Quantity::from_f64(2.0),
      response_tx,
  }).await?;

  2. Engine receives command:
  while let Some(command) = rx.recv().await {  // ← Receives here
      match command {
          OrderBookCommand::PlaceLimitOrder { ... } => {

  3. Engine processes:
  - Creates order
  - Checks balance (needs $100,000 USD)
  - Reserves $100,000
  - Tries to match
  - No matches found
  - Adds to orderbook at $50k level

  4. Engine sends response:
  response_tx.send(OrderBookResponse::OrderPlaced {
      order_id,
      trades: vec![],  // No trades
      status: "Added to book".to_string(),
  })

  5. Handler receives response:
  let response = response_rx.await?;  // ← Gets response here
  // Return HTTP response to user

  ---
  Why This Design?

  Single-Threaded Benefits:

  - ✅ No race conditions
  - ✅ No mutex/lock overhead
  - ✅ Guaranteed sequential consistency
  - ✅ Simple to reason about

  Message Passing Benefits:

  - ✅ Decouples HTTP layer from orderbook
  - ✅ Each request gets own response channel
  - ✅ Can handle concurrent HTTP requests
  - ✅ Clean separation of concerns

  Balance Reservation:

  - ✅ Prevents double-spending
  - ✅ Funds locked when order placed
  - ✅ Refunded on cancellation
  - ✅ Released on fill

  This engine is the single source of truth for all orderbook state!
