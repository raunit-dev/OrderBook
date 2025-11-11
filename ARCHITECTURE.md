# Orderbook System Architecture

**Phase 1 & Phase 2 Implementation**

---

## Data Types (`types/`)

```
┌─────────────┐  ┌──────────────┐  ┌────────────┐  ┌─────────────┐
│   Price     │  │   Quantity   │  │   Order    │  │    Trade    │
│             │  │              │  │            │  │             │
│ u64 (6 dec) │  │ u64 (8 dec)  │  │ - id       │  │ - id        │
│             │  │              │  │ - user_id  │  │ - maker_id  │
│ Ord/PartialOrd│ Add/Sub ops  │  │ - side     │  │ - taker_id  │
└─────────────┘  └──────────────┘  │ - type     │  │ - price     │
                                    │ - price    │  │ - quantity  │
┌─────────────┐  ┌──────────────┐  │ - quantity │  │ - timestamp │
│    User     │  │ UserBalance  │  │ - status   │  └─────────────┘
│             │  │              │  │ - timestamp│
│ - id        │  │ HashMap<     │  └────────────┘
│ - username  │  │   String,    │
│ - email     │  │   f64>       │  OrderSide: Buy | Sell
│ - password  │  │              │  OrderType: Limit | Market
└─────────────┘  │ (USD, BTC)   │  OrderStatus: Open | PartiallyFilled
                 └──────────────┘               Filled | Cancelled
```

---

## OrderBook Structure (`orderbook/`)

```
┌──────────────────────────────────────────────────────────────────┐
│                         OrderBook                                 │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │  bids: BTreeMap<Reverse<Price>, PriceLevel>             │     │
│  │  ─────────────────────────────────────────────────       │     │
│  │  Key: Reverse<Price> (DESCENDING order - highest first) │     │
│  │                                                           │     │
│  │  100.50 (highest) ──► PriceLevel { orders: [O1, O2] }   │     │
│  │  100.25           ──► PriceLevel { orders: [O3] }       │     │
│  │  100.00 (lowest)  ──► PriceLevel { orders: [O4, O5] }   │     │
│  └─────────────────────────────────────────────────────────┘     │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │  asks: BTreeMap<Price, PriceLevel>                      │     │
│  │  ──────────────────────────────────────────              │     │
│  │  Key: Price (ASCENDING order - lowest first)            │     │
│  │                                                           │     │
│  │  99.00 (lowest)   ──► PriceLevel { orders: [O6, O7] }   │     │
│  │  99.25            ──► PriceLevel { orders: [O8] }       │     │
│  │  99.50 (highest)  ──► PriceLevel { orders: [O9] }       │     │
│  └─────────────────────────────────────────────────────────┘     │
│                                                                   │
│  orders: HashMap<Uuid, Order>      // O(1) order lookup          │
│  user_balances: HashMap<Uuid, UserBalance>  // User funds        │
│                                                                   │
│  Methods:                                                         │
│  • best_bid() -> Option<Price>                                   │
│  • best_ask() -> Option<Price>                                   │
│  • add_order(order)                                              │
│  • cancel_order(order_id)                                        │
│  • get_depth(levels) -> (bids, asks)                             │
│  • match_order(order) -> Vec<Trade>  [matching.rs]              │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│                      PriceLevel                                   │
├──────────────────────────────────────────────────────────────────┤
│  price: Price                                                     │
│  orders: VecDeque<Order>  // FIFO queue                          │
│  total_volume: Quantity                                           │
│                                                                   │
│  ┌────────┬────────┬────────┬────────┐                           │
│  │ Order1 │ Order2 │ Order3 │ Order4 │  (FIFO: First In First Out)│
│  └────────┴────────┴────────┴────────┘                           │
│     ▲                              ▲                              │
│   front()                      back()                             │
│   (matched first)            (added last)                         │
│                                                                   │
│  Methods:                                                         │
│  • add_order(order) - push to back                               │
│  • front_mut() - get first order                                 │
│  • pop_if_filled() - remove filled orders                        │
│  • remove_order(id) - cancel specific order                      │
└──────────────────────────────────────────────────────────────────┘
```

---

## Matching Engine (`matching.rs`)

**Implemented as impl block for OrderBook**

```
match_order(order) -> Result<Vec<Trade>, String>
                    │
                    ▼
          ┌──────────────────┐
          │  Order Type?     │
          └──────────────────┘
           /                \
          /                  \
┌────────────┐          ┌─────────────┐
│   LIMIT    │          │   MARKET    │
└────────────┘          └─────────────┘
      │                        │
      ▼                        ▼
┌──────────────────────┐  ┌──────────────────────┐
│ match_limit_order()  │  │ match_market_order() │
└──────────────────────┘  └──────────────────────┘
      │                        │
      └────────────┬───────────┘
                   ▼
         ┌──────────────────┐
         │  Matching Logic  │
         └──────────────────┘
                   │
   ┌───────────────┼───────────────────┐
   ▼               ▼                   ▼
┌──────────┐   ┌──────────┐   ┌──────────────────┐
│Price-Time│   │Fill      │   │Execute Trade     │
│Priority  │   │Orders    │   │Settlement        │
│          │   │(partial  │   │(update balances) │
│Best      │   │OK)       │   │USD ↔ BTC         │
│bid/ask   │   │          │   │                  │
│first     │   │Update    │   │Debit/Credit both │
│FIFO at   │   │state     │   │users             │
│same price│   │          │   │                  │
└──────────┘   └──────────┘   └──────────────────┘
```

### Limit Order Flow:
1. Check if can match with opposite side (price crosses)
2. Match as much as possible (may be partial)
3. If remaining quantity > 0, add to book

### Market Order Flow:
1. Match at best available prices
2. Continue until fully filled or insufficient liquidity
3. Never added to book (immediate execution only)

---

## Detailed Market Order Matching Example

### What is a Market Order?

**Market Order** = "Execute immediately at the best available price(s)"

**Key Differences from Limit Orders:**
- ✅ **No price limit** - will match at ANY available price
- ✅ **Immediate execution** - never added to orderbook
- ✅ **Can experience slippage** - may execute across multiple price levels
- ❌ **No price protection** - unlike limit orders which have a maximum/minimum price

### Complete Example: Market BUY Order

**Initial Orderbook State:**
```
Asks (Sell Orders):
  $98  → [5 SOL (UserA), 3 SOL (UserB)]   total: 8 SOL
  $100 → [10 SOL (UserC)]                  total: 10 SOL
  $105 → [20 SOL (UserD)]                  total: 20 SOL
```

**Incoming: Market BUY 15 SOL** (no price limit!)

#### Iteration 1: Match at $98
```rust
best_ask_price = $98 (lowest sell price)
// No price check - market orders take any price!

Match with UserA (front of queue at $98):
  - Fill: min(15 SOL needed, 5 SOL available) = 5 SOL
  - Execute: 5 SOL @ $98 = $490
  - UserA: Fully filled, removed from orderbook
  - Market order: 15 → 10 SOL remaining
```

**State after Iteration 1:**
```
Asks:
  $98  → [3 SOL (UserB)]   ← UserA removed
  $100 → [10 SOL (UserC)]
  $105 → [20 SOL (UserD)]

Trades: [5 SOL @ $98]
Remaining: 10 SOL
```

#### Iteration 2: Continue at $98
```rust
best_ask_price = $98 (still best)

Match with UserB (now front of queue):
  - Fill: min(10 SOL needed, 3 SOL available) = 3 SOL
  - Execute: 3 SOL @ $98 = $294
  - UserB: Fully filled, removed
  - Market order: 10 → 7 SOL remaining
  - Price level $98: Now empty, removed from orderbook
```

**State after Iteration 2:**
```
Asks:
  $100 → [10 SOL (UserC)]   ← $98 level removed!
  $105 → [20 SOL (UserD)]

Trades: [5 SOL @ $98, 3 SOL @ $98]
Remaining: 7 SOL
```

#### Iteration 3: Price Slippage to $100
```rust
best_ask_price = $100 (new best - price jumped!)

Match with UserC:
  - Fill: min(7 SOL needed, 10 SOL available) = 7 SOL
  - Execute: 7 SOL @ $100 = $700
  - UserC: Partially filled (3 SOL remaining)
  - Market order: 7 → 0 SOL ✅ FULLY FILLED!
```

**Final State:**
```
Asks:
  $100 → [3 SOL (UserC)]   ← UserC partially filled
  $105 → [20 SOL (UserD)]

Trades Executed:
  1. 5 SOL @ $98  = $490
  2. 3 SOL @ $98  = $294
  3. 7 SOL @ $100 = $700
  ────────────────────────
  Total: 15 SOL for $1,484
  Average Price: $98.93 per SOL
```

### Key Market Order Characteristics

**Price Slippage:**
- Started matching at $98
- Exhausted $98 level after 8 SOL
- Continued at $100 for remaining 7 SOL
- Paid more than initial best price

**Algorithm:**
```rust
while !order.is_fully_filled() {
    // 1. Get best available price
    best_price = get_best_ask()

    // 2. NO PRICE CHECK (unlike limit orders)
    // if best_price > limit_price { break } ← NOT DONE for market orders!

    // 3. Match with front order (FIFO)
    fill_quantity = min(order.remaining, maker.remaining)

    // 4. Execute trade
    execute_trade(fill_quantity, best_price)

    // 5. Continue until filled or no liquidity
}
```

### Market Order vs Limit Order

| Aspect | Market Order | Limit Order |
|--------|--------------|-------------|
| **Price Check** | None - accepts any price | Checks: `if best_price > limit { break }` |
| **Execution** | Immediate (or error) | May be partial, rest goes to book |
| **Added to Book** | Never | Yes, if not fully filled |
| **Price Protection** | None (can experience slippage) | Protected by limit price |
| **Guaranteed Fill** | Only if enough liquidity exists | Only if price matches |
| **Use Case** | "Buy NOW at any price" | "Buy only if price ≤ $100" |

### Insufficient Liquidity Example

**Orderbook:**
```
Asks:
  $98 → [2 SOL]
```

**Incoming: Market BUY 10 SOL**

**Result:**
```rust
// Iteration 1: Match 2 SOL @ $98
// Iteration 2: best_ask() returns None
return Err("Insufficient liquidity for market order")
```

❌ **Error returned** - market order cannot be filled completely

**Note:** In this implementation, market orders are "all or nothing" - they either fill completely or return an error. Some exchanges allow partial fills for market orders.

---

## Message Passing System (`messages/`)

### OrderBookCommand (enum)
```
• PlaceLimitOrder { user_id, side, price, quantity, response_tx }
• PlaceMarketOrder { user_id, side, quantity, response_tx }
• CancelOrder { user_id, order_id, response_tx }
• GetOrderBook { depth, response_tx }
• GetUserBalance { user_id, response_tx }
• AddFunds { user_id, currency, amount, response_tx }

Each variant contains:
response_tx: oneshot::Sender<OrderBookResponse>
```

### OrderBookResponse (enum)
```
• OrderPlaced { order_id, trades, status }
• OrderCancelled { order_id, success }
• OrderBookDepth { bids, asks }
• UserBalance { balance }
• FundsAdded { user_id, currency, new_balance }
• Error { message }
```

---

## Engine Architecture (`engine/`)

```
┌────────────────────────────────────────────────────────────────┐
│           run_orderbook_engine(rx: mpsc::Receiver)             │
│                                                                 │
│  Runs in SINGLE THREAD (no locks needed!)                      │
│                                                                 │
│  let mut orderbook = OrderBook::new();                         │
│                                                                 │
│  while let Some(command) = rx.recv().await {                   │
│      match command {                                            │
│          PlaceLimitOrder => {                                   │
│              1. Validate balance                                │
│              2. Reserve funds (deduct from balance)             │
│              3. Match order                                     │
│              4. Send response via oneshot                       │
│          }                                                       │
│          PlaceMarketOrder => { ... }                            │
│          CancelOrder => {                                       │
│              1. Remove from orderbook                           │
│              2. Refund reserved balance                         │
│              3. Send response                                   │
│          }                                                       │
│          GetOrderBook => {                                      │
│              Send depth snapshot                                │
│          }                                                       │
│          ... other commands                                     │
│      }                                                           │
│  }                                                               │
└────────────────────────────────────────────────────────────────┘
```

### Key Features:
- **Single-threaded** = No mutex/rwlock needed
- **Sequential processing** = Consistent state
- **Balance validation** before placing orders
- **Automatic refunds** on cancellation

---

## Full System Data Flow

```
[Future: HTTP Handler]
         │
         │ 1. Create oneshot channel
         │    let (tx, rx) = oneshot::channel()
         │
         │ 2. Send command via mpsc
         ▼
┌─────────────────────┐
│  mpsc::Sender       │────────────────────┐
│  (in AppState)      │                    │
└─────────────────────┘                    │
       │                                    │
       │ OrderBookCommand                  │
       │ { data, response_tx: tx }         │
       │                                    │
       ▼                                    │
┌──────────────────────────────────────────▼──────────────────┐
│            OrderBook Engine Thread                           │
│            (tokio::spawn)                                    │
│                                                              │
│    mpsc::Receiver ───► Process Command                      │
│                             │                                │
│                             ▼                                │
│                     ┌──────────────┐                         │
│                     │  OrderBook   │                         │
│                     │              │                         │
│                     │  • bids      │                         │
│                     │  • asks      │                         │
│                     │  • orders    │                         │
│                     │  • balances  │                         │
│                     └──────────────┘                         │
│                             │                                │
│                             ▼                                │
│                     Execute & Generate                       │
│                     OrderBookResponse                        │
│                             │                                │
└─────────────────────────────┼────────────────────────────────┘
                              │
                              │ Send via oneshot
                              ▼
[Future: HTTP Handler]
         │
         │ 3. Await response
         │    let response = rx.await?
         │
         │ 4. Return HTTP response
         ▼
[User receives result]
```

---

## Application State (`state/`)

```rust
┌──────────────────────────────────────────────────────────┐
│                      AppState                             │
├──────────────────────────────────────────────────────────┤
│  orderbook_tx: Arc<mpsc::Sender<OrderBookCommand>>       │
│                                                           │
│  • Shared across all Actix-web workers                   │
│  • Arc allows cheap cloning                              │
│  • All handlers get access to same sender                │
└──────────────────────────────────────────────────────────┘
```

**Usage in handlers:**
```rust
async fn handler(state: web::Data<AppState>) {
    let (tx, rx) = oneshot::channel();
    state.orderbook_tx.send(command).await?;
    let response = rx.await?;
}
```

---

## Key Design Decisions

### 1. Single-Threaded OrderBook
- ✓ No mutex/rwlock overhead
- ✓ Guaranteed sequential consistency
- ✓ Simpler reasoning about state

### 2. BTreeMap for Price Levels
- ✓ O(log n) insertion/removal
- ✓ Auto-sorted by price
- ✓ Easy to get best bid/ask

### 3. VecDeque for Orders at Price Level
- ✓ FIFO ordering (price-time priority)
- ✓ O(1) front/back operations

### 4. Fixed-Point Arithmetic
- ✓ No floating point errors
- ✓ Deterministic comparisons
- ✓ Price: 6 decimals, Quantity: 8 decimals

### 5. Message Passing with MPSC + Oneshot
- ✓ Clean separation of concerns
- ✓ HTTP layer and orderbook decoupled
- ✓ Each request gets individual response

### 6. Balance Reservation
- ✓ Funds locked when limit order placed
- ✓ Prevents double-spending
- ✓ Refunded on cancellation

---

## Implementation Status

### ✅ Phase 1: Core Types & OrderBook
- [x] Price, Quantity, Order, Trade, User, UserBalance types
- [x] OrderBook with BTreeMap structure
- [x] PriceLevel with VecDeque FIFO queue
- [x] Matching engine (limit & market orders)
- [x] Balance management and trade settlement

### ✅ Phase 2: Message Passing
- [x] OrderBookCommand/Response enums
- [x] OrderBook engine thread
- [x] Tokio MPSC channels
- [x] Oneshot response channels
- [x] AppState with Arc<Sender>

### 🔄 Phase 3: HTTP Endpoints (Next)
- [ ] Signup/Signin handlers
- [ ] Order placement endpoints
- [ ] OrderBook query endpoint
- [ ] Balance management endpoints

### 🔄 Phase 4: Authentication (Next)
- [ ] JWT token generation/validation
- [ ] Password hashing with bcrypt
- [ ] Auth middleware

---

**Generated**: Phase 1 & 2 Complete
