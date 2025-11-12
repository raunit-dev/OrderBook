# OrderBook - High-Performance Cryptocurrency Exchange Engine

A production-ready, low-latency orderbook matching engine built with Rust, featuring price-time priority matching, real-time trade settlement, and a modern HTTP API.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-24%20passing-brightgreen.svg)]()

---

## Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Architecture](#architecture)
- [Technology Stack](#technology-stack)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Running the Server](#running-the-server)
- [API Documentation](#api-documentation)
  - [Authentication Endpoints](#authentication-endpoints)
  - [Order Endpoints](#order-endpoints)
  - [Market Data Endpoints](#market-data-endpoints)
  - [User Endpoints](#user-endpoints)
- [Core Concepts](#core-concepts)
  - [OrderBook Structure](#orderbook-structure)
  - [Matching Engine](#matching-engine)
  - [Message Passing Architecture](#message-passing-architecture)
  - [Balance Management](#balance-management)
- [Project Structure](#project-structure)
- [Design Decisions](#design-decisions)
- [Testing](#testing)
- [Performance Characteristics](#performance-characteristics)
- [Known Limitations](#known-limitations)
- [Future Enhancements](#future-enhancements)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

This project implements a **limit order book (LOB)** with a matching engine designed for high throughput and low latency. It uses a **single-threaded actor model** for the core matching engine, eliminating lock contention while maintaining strict sequential consistency.

**What is an OrderBook?**

An orderbook is a real-time list of buy and sell orders for a specific asset (in this case, BTC/USD). It matches buyers with sellers based on price-time priority:
1. **Best price first** - Orders with better prices match first
2. **FIFO at same price** - Orders at the same price level match in chronological order (first-in, first-out)

**Use Cases:**
- Cryptocurrency exchanges
- Stock trading platforms
- Automated market makers (AMMs)
- Educational/research projects
- High-frequency trading (HFT) simulations

---

## Key Features

### Core Functionality
- ✅ **Limit Orders** - Place orders at specific prices
- ✅ **Market Orders** - Execute immediately at best available prices
- ✅ **Order Cancellation** - Cancel open orders with automatic fund refunds
- ✅ **Real-time Matching** - Price-time priority matching algorithm
- ✅ **Balance Management** - Fund reservation and settlement system
- ✅ **OrderBook Depth Queries** - View current market depth

### Technical Highlights
- ✅ **Single-Threaded Engine** - No lock contention, guaranteed consistency
- ✅ **Message Passing Architecture** - Clean separation via async channels
- ✅ **Fixed-Point Arithmetic** - No floating-point precision errors
- ✅ **JWT Authentication** - Secure user authentication with bcrypt
- ✅ **Async/Await** - Built on Tokio runtime for scalability
- ✅ **Type Safety** - Leverages Rust's ownership system

### Performance
- **O(log n)** order insertion/removal (BTreeMap)
- **O(1)** best bid/ask lookup
- **O(1)** FIFO matching at price levels (VecDeque)
- **O(1)** order lookup by ID (HashMap)

---

## Architecture

### High-Level Data Flow

```
┌─────────────────┐
│  HTTP Client    │
└────────┬────────┘
         │ REST API (JSON)
         ▼
┌─────────────────────────────────────────────┐
│         Actix-Web HTTP Server               │
│  ┌───────────────────────────────────────┐  │
│  │  Routes:                              │  │
│  │  • POST /api/auth/signup              │  │
│  │  • POST /api/auth/signin              │  │
│  │  • POST /api/orders/limit (auth)      │  │
│  │  • POST /api/orders/market (auth)     │  │
│  │  • DELETE /api/orders/:id (auth)      │  │
│  │  • GET /api/depth                     │  │
│  │  • GET /api/user/balance (auth)       │  │
│  │  • POST /api/user/onramp (auth)       │  │
│  └───────────────────────────────────────┘  │
└────────┬────────────────────────────────────┘
         │
         │ OrderBookCommand
         │ (via mpsc channel)
         ▼
┌─────────────────────────────────────────────┐
│      OrderBook Engine (Single Thread)       │
│                                             │
│  ┌─────────────────────────────────────┐   │
│  │       OrderBook State               │   │
│  │  • bids: BTreeMap<Price, Orders>    │   │
│  │  • asks: BTreeMap<Price, Orders>    │   │
│  │  • orders: HashMap<Uuid, Order>     │   │
│  │  • balances: HashMap<Uuid, Balance> │   │
│  └─────────────────────────────────────┘   │
│                                             │
│  Processing:                                │
│  1. Validate balance                        │
│  2. Reserve funds                           │
│  3. Match order                             │
│  4. Settle trades                           │
│  5. Update balances                         │
└────────┬────────────────────────────────────┘
         │
         │ OrderBookResponse
         │ (via oneshot channel)
         ▼
┌─────────────────┐
│  HTTP Response  │
└─────────────────┘
```

### Single-Threaded Actor Model

The orderbook engine runs in a **single async task**, processing commands sequentially:

```rust
pub async fn run_orderbook_engine(mut rx: mpsc::Receiver<OrderBookCommand>) {
    let mut orderbook = OrderBook::new();

    while let Some(command) = rx.recv().await {
        // Process ONE command at a time
        match command {
            PlaceLimitOrder { .. } => { /* ... */ }
            CancelOrder { .. } => { /* ... */ }
            // ... other commands
        }
    }
}
```

**Benefits:**
- No mutex/rwlock overhead
- Guaranteed sequential consistency
- No race conditions
- Simpler reasoning about state

**How concurrency works:**
- Multiple HTTP handlers can send commands concurrently (multi-producer)
- Commands queue in the mpsc channel
- Engine processes them one-by-one (single-consumer)
- Each command gets a response via its own oneshot channel

---

## Technology Stack

| Component | Technology | Purpose |
|-----------|-----------|---------|
| **Language** | Rust 1.75+ | Memory safety, zero-cost abstractions |
| **Async Runtime** | Tokio | Async I/O, task scheduling |
| **Web Framework** | Actix-Web | HTTP server, routing |
| **Authentication** | jsonwebtoken + bcrypt | JWT tokens, password hashing |
| **Data Structures** | BTreeMap, VecDeque, HashMap | Orderbook storage |
| **Channels** | tokio::sync::mpsc, oneshot | Message passing |
| **Serialization** | Serde + serde_json | JSON API |
| **Logging** | env_logger | Request/response logging |
| **Testing** | Rust built-in | Unit tests |

---

## Getting Started

### Prerequisites

- **Rust 1.75 or higher**
  ```bash
  # Install Rust via rustup
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

  # Verify installation
  rustc --version
  cargo --version
  ```

- **Optional: httpie or curl** for testing API endpoints
  ```bash
  # macOS
  brew install httpie

  # Linux
  sudo apt install httpie
  ```

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/orderbook.git
   cd orderbook
   ```

2. **Build the project**
   ```bash
   # Debug build (faster compilation)
   cargo build

   # Release build (optimized for performance)
   cargo build --release
   ```

3. **Run tests**
   ```bash
   cargo test
   ```

### Running the Server

```bash
# Run in development mode (with logs)
RUST_LOG=info cargo run

# Or run the compiled binary
./target/debug/Orderbook

# For production (release mode)
./target/release/Orderbook
```

**Expected output:**
```
🚀 Starting Orderbook System...
📊 Orderbook engine started
🌐 Starting HTTP server on http://127.0.0.1:8080
OrderBook engine started and listening for commands...
```

**The server is now running on `http://127.0.0.1:8080`**

---

## API Documentation

All endpoints return JSON. Protected endpoints require a JWT token in the `Authorization: Bearer <token>` header.

### Authentication Endpoints

#### 1. Sign Up

Create a new user account.

**Endpoint:** `POST /api/auth/signup`

**Request Body:**
```json
{
  "username": "trader1",
  "email": "trader1@example.com",
  "password": "secure_password"
}
```

**Response (200 OK):**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "trader1",
  "email": "trader1@example.com"
}
```

**Example using httpie:**
```bash
http POST :8080/api/auth/signup \
  username=trader1 \
  email=trader1@example.com \
  password=secure_password
```

**Example using curl:**
```bash
curl -X POST http://127.0.0.1:8080/api/auth/signup \
  -H "Content-Type: application/json" \
  -d '{"username":"trader1","email":"trader1@example.com","password":"secure_password"}'
```

---

#### 2. Sign In

Authenticate and receive a JWT token.

**Endpoint:** `POST /api/auth/signin`

**Request Body:**
```json
{
  "email": "trader1@example.com",
  "password": "secure_password"
}
```

**Response (200 OK):**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "user_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Token expires in 24 hours.**

**Example:**
```bash
http POST :8080/api/auth/signin \
  email=trader1@example.com \
  password=secure_password
```

**Save the token for authenticated requests:**
```bash
export TOKEN="eyJ0eXAiOiJKV1QiLCJhbGc..."
```

---

### Order Endpoints

All order endpoints require authentication (`Authorization: Bearer <token>`).

#### 3. Create Limit Order

Place a limit order at a specific price.

**Endpoint:** `POST /api/orders/limit`

**Request Body:**
```json
{
  "side": "Buy",        // "Buy" or "Sell"
  "price": 50000.0,     // Price per BTC in USD
  "quantity": 0.5       // Amount of BTC
}
```

**Response (200 OK):**
```json
{
  "order_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
  "status": "Matched",  // or "Added to book"
  "trades": [
    {
      "trade_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
      "maker_id": "buyer-uuid",
      "taker_id": "seller-uuid",
      "price": 50000.0,
      "quantity": 0.3,
      "timestamp": "2025-01-15T10:30:00Z"
    }
  ]
}
```

**Example:**
```bash
http POST :8080/api/orders/limit \
  "Authorization: Bearer $TOKEN" \
  side=Buy \
  price:=50000.0 \
  quantity:=0.5
```

**Notes:**
- **Buy orders** require sufficient USD balance (price × quantity)
- **Sell orders** require sufficient BTC balance
- Funds are reserved when order is placed
- If the order matches, trades are executed immediately
- Unmatched portion remains in the orderbook

---

#### 4. Create Market Order

Execute immediately at the best available price(s).

**Endpoint:** `POST /api/orders/market`

**Request Body:**
```json
{
  "side": "Buy",        // "Buy" or "Sell"
  "quantity": 1.0       // Amount of BTC (no price specified)
}
```

**Response (200 OK):**
```json
{
  "order_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "status": "Filled",   // or "No liquidity"
  "trades": [
    {
      "trade_id": "...",
      "price": 49800.0,  // Executed at best ask
      "quantity": 0.7,
      "timestamp": "2025-01-15T10:31:00Z"
    },
    {
      "trade_id": "...",
      "price": 49850.0,  // Slippage to next level
      "quantity": 0.3,
      "timestamp": "2025-01-15T10:31:00Z"
    }
  ]
}
```

**Example:**
```bash
http POST :8080/api/orders/market \
  "Authorization: Bearer $TOKEN" \
  side=Sell \
  quantity:=1.0
```

**Market Order Behavior:**
- Executes immediately or fails
- Never added to orderbook
- May experience price slippage across multiple levels
- Returns error if insufficient liquidity

---

#### 5. Cancel Order

Cancel an open order and refund reserved funds.

**Endpoint:** `DELETE /api/orders/:order_id`

**Path Parameter:**
- `order_id` - UUID of the order to cancel

**Response (200 OK):**
```json
{
  "order_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
  "success": true
}
```

**Response (400 Bad Request):**
```json
{
  "error": "Not authorized to cancel this order"
}
```

**Example:**
```bash
http DELETE :8080/api/orders/3fa85f64-5717-4562-b3fc-2c963f66afa6 \
  "Authorization: Bearer $TOKEN"
```

**Notes:**
- Only the user who placed the order can cancel it
- Reserved funds are automatically refunded
- Partially filled orders refund the remaining quantity

---

### Market Data Endpoints

#### 6. Get OrderBook Depth

View current market depth (top 10 price levels on each side).

**Endpoint:** `GET /api/depth`

**No authentication required.**

**Response (200 OK):**
```json
{
  "bids": [
    [49900.0, 2.5],    // [price, total_quantity]
    [49850.0, 1.8],
    [49800.0, 3.2]
  ],
  "asks": [
    [50100.0, 1.5],
    [50150.0, 2.0],
    [50200.0, 0.8]
  ]
}
```

**Example:**
```bash
http GET :8080/api/depth
```

**Notes:**
- Returns top 10 levels by default
- Bids sorted descending (highest first)
- Asks sorted ascending (lowest first)
- Each level shows aggregated quantity at that price

---

### User Endpoints

#### 7. Get Balance

Check your current USD and BTC balances.

**Endpoint:** `GET /api/user/balance`

**Requires authentication.**

**Response (200 OK):**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "balances": {
    "USD": 25000.0,
    "BTC": 0.5
  }
}
```

**Example:**
```bash
http GET :8080/api/user/balance \
  "Authorization: Bearer $TOKEN"
```

---

#### 8. Deposit Funds (Onramp)

Add funds to your account.

**Endpoint:** `POST /api/user/onramp`

**Requires authentication.**

**Request Body:**
```json
{
  "currency": "USD",   // "USD" or "BTC"
  "amount": 10000.0
}
```

**Response (200 OK):**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "currency": "USD",
  "new_balance": 35000.0
}
```

**Example:**
```bash
http POST :8080/api/user/onramp \
  "Authorization: Bearer $TOKEN" \
  currency=USD \
  amount:=10000.0
```

**Notes:**
- This simulates depositing funds (no real payment processing)
- Use to fund your account for testing
- Supports both USD and BTC deposits

---

### Complete Usage Example

```bash
# 1. Sign up
http POST :8080/api/auth/signup \
  username=alice email=alice@example.com password=test123

# 2. Sign in and save token
export TOKEN=$(http POST :8080/api/auth/signin \
  email=alice@example.com password=test123 | jq -r '.token')

# 3. Deposit funds
http POST :8080/api/user/onramp \
  "Authorization: Bearer $TOKEN" \
  currency=USD amount:=100000.0

# 4. Check balance
http GET :8080/api/user/balance \
  "Authorization: Bearer $TOKEN"

# 5. Place a limit buy order
http POST :8080/api/orders/limit \
  "Authorization: Bearer $TOKEN" \
  side=Buy price:=50000.0 quantity:=1.0

# 6. View orderbook
http GET :8080/api/depth

# 7. Cancel order (if needed)
http DELETE :8080/api/orders/<order_id> \
  "Authorization: Bearer $TOKEN"
```

---

## Core Concepts

### OrderBook Structure

The orderbook maintains two sorted maps of price levels:

```
OrderBook
├─ bids: BTreeMap<Reverse<Price>, PriceLevel>  (descending - best bid first)
│   └─ 50000 → [Order1, Order2, Order3]        (FIFO queue)
│   └─ 49900 → [Order4]
│   └─ 49800 → [Order5, Order6]
│
├─ asks: BTreeMap<Price, PriceLevel>            (ascending - best ask first)
│   └─ 50100 → [Order7, Order8]
│   └─ 50200 → [Order9]
│   └─ 50300 → [Order10]
│
├─ orders: HashMap<Uuid, Order>                 (fast lookup by ID)
└─ user_balances: HashMap<Uuid, UserBalance>    (user funds)
```

**Key Properties:**
- **BTreeMap** provides O(log n) insertion and sorted iteration
- **Reverse wrapper** on bid prices ensures highest bid is first
- **VecDeque** at each price level maintains FIFO order
- **HashMap** allows O(1) order lookup for cancellations

---

### Matching Engine

#### Limit Order Matching

```rust
1. Create order with price limit
2. Check if price crosses spread:
   - Buy: if order_price >= best_ask → can match
   - Sell: if order_price <= best_bid → can match
3. Match against opposite side while price favorable:
   - Execute trades at maker's price
   - Update order quantities
   - Settle balances
4. If remaining quantity > 0:
   - Add to orderbook at order's price level
```

**Example:**
```
OrderBook:
  Asks: [($50,100, 2 BTC), ($50,200, 1 BTC)]

Incoming: Buy 3 BTC @ $50,150

Matching:
  - Match 2 BTC @ $50,100 (fully fills ask)
  - Remaining: 1 BTC
  - Check next ask: $50,200 > $50,150 (limit exceeded)
  - Stop matching

Result:
  - 2 BTC executed @ $50,100
  - 1 BTC added to bids @ $50,150
```

#### Market Order Matching

```rust
1. Create order with no price limit
2. Match against best available prices:
   - Keep matching until fully filled
   - No price limit check
   - May cross multiple price levels (slippage)
3. If insufficient liquidity:
   - Return error (order not placed)
```

**Example:**
```
OrderBook:
  Asks: [($50,000, 1 BTC), ($50,100, 2 BTC), ($50,200, 1 BTC)]

Incoming: Market Buy 3.5 BTC

Matching:
  - Match 1 BTC @ $50,000
  - Match 2 BTC @ $50,100
  - Match 0.5 BTC @ $50,200

Result:
  - Total: 3.5 BTC
  - Average price: $50,085.71
  - Price slippage experienced
```

---

### Message Passing Architecture

The system uses **async channels** to decouple the HTTP layer from the orderbook engine.

```rust
// HTTP Handler (Many threads)
let (response_tx, response_rx) = oneshot::channel();

orderbook_tx.send(OrderBookCommand::PlaceLimitOrder {
    user_id, side, price, quantity,
    response_tx,  // ← Oneshot sender for response
}).await?;

let response = response_rx.await?;  // ← Wait for engine to respond

// OrderBook Engine (Single thread)
while let Some(command) = mpsc_rx.recv().await {
    match command {
        PlaceLimitOrder { response_tx, .. } => {
            // Process order
            let result = orderbook.match_order(order);

            // Send response back
            response_tx.send(result);
        }
    }
}
```

**Channel Types:**
- **mpsc** (multi-producer, single-consumer) - For commands from HTTP handlers
- **oneshot** (one-time response) - For engine responses back to handlers

---

### Balance Management

#### Fund Reservation

When a limit order is placed, funds are **reserved** (locked):

```rust
Buy Order:
  Required: price × quantity (in USD)
  Reserve: Deduct USD from available balance

Sell Order:
  Required: quantity (in BTC)
  Reserve: Deduct BTC from available balance
```

**Example:**
```
User balance: { USD: 100,000, BTC: 2.0 }

Place Buy 1 BTC @ $50,000:
  - Reserve: $50,000 USD
  - New balance: { USD: 50,000 (available), BTC: 2.0 }
  - Reserved: $50,000 (locked in order)

Place Sell 0.5 BTC @ $51,000:
  - Reserve: 0.5 BTC
  - New balance: { USD: 50,000, BTC: 1.5 (available) }
  - Reserved: $50,000 USD + 0.5 BTC
```

#### Trade Settlement

When orders match, funds are **settled** (exchanged):

```rust
Buyer (taker): Buy 1 BTC @ $50,000
Seller (maker): Sell 1 BTC @ $50,000

Settlement:
  1. Debit buyer: -$50,000 USD (from reserved)
  2. Credit buyer: +1 BTC
  3. Debit seller: -1 BTC (from reserved)
  4. Credit seller: +$50,000 USD
```

#### Cancellation Refunds

When an order is cancelled, reserved funds are **refunded**:

```rust
Cancelled Buy Order:
  - Refund: price × remaining_quantity (in USD)

Cancelled Sell Order:
  - Refund: remaining_quantity (in BTC)
```

**Example:**
```
Order: Buy 10 BTC @ $50,000 (reserved $500,000)
Matched: 3 BTC (used $150,000)
Cancelled with 7 BTC remaining

Refund: $350,000 USD (7 × $50,000)
```

---

## Project Structure

```
orderbook/
├── Cargo.toml                  # Dependencies and project metadata
├── README.md                   # This file
├── ARCHITECTURE.md             # Detailed architecture documentation
├── engine.md                   # Engine implementation details
├── src/
│   ├── main.rs                 # HTTP server setup and entry point
│   ├── lib.rs                  # Library exports
│   │
│   ├── engine/
│   │   ├── mod.rs
│   │   └── engine.rs           # OrderBook engine event loop
│   │
│   ├── handlers/               # HTTP request handlers
│   │   ├── mod.rs
│   │   ├── auth.rs             # Signup/signin endpoints
│   │   ├── orders.rs           # Order placement/cancellation
│   │   ├── market.rs           # OrderBook depth queries
│   │   └── user.rs             # Balance and onramp endpoints
│   │
│   ├── messages/               # Inter-task communication
│   │   ├── mod.rs
│   │   └── commands.rs         # Command/Response enums
│   │
│   ├── orderbook/              # Core orderbook logic
│   │   ├── mod.rs
│   │   ├── orderbook.rs        # OrderBook struct and data structure
│   │   ├── price_level.rs      # FIFO queue for orders at a price
│   │   ├── matching.rs         # Limit order matching algorithm
│   │   ├── market_matching.rs  # Market order matching algorithm
│   │   └── settlement.rs       # Trade settlement and balance updates
│   │
│   ├── types/                  # Domain types
│   │   ├── mod.rs
│   │   ├── price.rs            # Fixed-point price (6 decimals)
│   │   ├── quantity.rs         # Fixed-point quantity (8 decimals)
│   │   ├── order.rs            # Order struct and enums
│   │   ├── trade.rs            # Trade execution record
│   │   └── user.rs             # User and UserBalance structs
│   │
│   ├── state/
│   │   ├── mod.rs
│   │   └── app_state.rs        # Shared application state
│   │
│   └── utils/                  # Utilities
│       ├── mod.rs
│       ├── auth.rs             # JWT and password utilities
│       ├── error.rs            # Error types and conversions
│       └── middleware.rs       # JWT validation middleware
```

### Module Responsibilities

| Module | Responsibility |
|--------|---------------|
| **engine** | Single-threaded event loop processing orderbook commands |
| **handlers** | HTTP request/response handling and routing |
| **messages** | Command and response types for message passing |
| **orderbook** | Core matching engine and orderbook data structure |
| **types** | Domain types (Price, Quantity, Order, Trade, User) |
| **state** | Shared application state (mpsc sender) |
| **utils** | Authentication, error handling, middleware |

---

## Design Decisions

### 1. Single-Threaded OrderBook Engine

**Decision:** Run the orderbook in a single async task, processing commands sequentially.

**Rationale:**
- Eliminates need for locks/mutexes
- Guarantees sequential consistency
- Simpler to reason about state transitions
- No race conditions possible

**Trade-offs:**
- Single point of processing (but high throughput due to no locking)
- Cannot leverage multi-core for orderbook operations
- Good for single trading pair; multiple pairs need separate engines

---

### 2. BTreeMap for Price Levels

**Decision:** Use `BTreeMap<Price, PriceLevel>` for bids and asks.

**Rationale:**
- Auto-sorts by price (O(log n) insertion)
- Easy to iterate in price order
- Fast best bid/ask lookup (first/last entry)

**Alternatives considered:**
- HashMap - O(1) insertion but not sorted
- SkipList - Similar performance but not in standard library

---

### 3. VecDeque for FIFO Queue

**Decision:** Use `VecDeque<Order>` at each price level.

**Rationale:**
- Implements price-time priority (FIFO)
- O(1) front/back operations
- Efficient for queue operations

**Alternatives considered:**
- Vec - Less efficient for removing front elements
- LinkedList - More allocations, cache-unfriendly

---

### 4. Fixed-Point Arithmetic

**Decision:** Store prices and quantities as `u64` with fixed decimal places.

**Rationale:**
- Avoids floating-point precision errors
- Deterministic comparisons
- Critical for financial calculations

**Implementation:**
- Price: 6 decimals (e.g., 50000.123456)
- Quantity: 8 decimals (e.g., 1.23456789 BTC)

---

### 5. Message Passing with MPSC + Oneshot

**Decision:** Use async channels for communication between HTTP and engine.

**Rationale:**
- Clean separation of concerns
- Decouples HTTP layer from orderbook
- Each request gets individual response
- Follows actor model pattern

**Trade-offs:**
- Additional serialization/deserialization overhead
- Slightly higher latency than direct function calls
- More complex than shared-memory approach

---

### 6. Balance Reservation System

**Decision:** Reserve funds when limit orders are placed.

**Rationale:**
- Prevents double-spending
- Ensures atomic operations
- Clear accounting of available vs reserved funds

**Implementation:**
- Deduct on order placement
- Credit on trade execution or cancellation

---

## Testing

### Run All Tests

```bash
cargo test
```

### Test Coverage

**24 unit tests covering:**

| Module | Tests | Coverage |
|--------|-------|----------|
| **types/order.rs** | 16 | Order creation, filling, cancellation, status updates |
| **types/price.rs** | 2 | Ordering, arithmetic operations |
| **types/quantity.rs** | 2 | Addition, subtraction |
| **types/trade.rs** | 2 | Trade creation, ID uniqueness |
| **types/user.rs** | 3 | User creation, balance operations |
| **utils/auth.rs** | 2 | Password hashing, JWT generation |

### Example Test Output

```
running 24 tests
test types::order::tests::test_cancel_order ... ok
test types::order::tests::test_fill_order ... ok
test types::order::tests::test_order_creation ... ok
test types::order::tests::test_partial_fill ... ok
test types::price::tests::test_price_ordering ... ok
test types::quantity::tests::test_quantity_add ... ok
test utils::auth::tests::test_hash_password ... ok
test utils::auth::tests::test_create_jwt ... ok
...

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Manual Integration Testing

Use the [API examples](#complete-usage-example) to test the full system end-to-end.

---

## Performance Characteristics

### Time Complexity

| Operation | Complexity | Data Structure |
|-----------|-----------|----------------|
| Insert order | O(log n) | BTreeMap insertion |
| Cancel order | O(log n) | BTreeMap removal |
| Match order | O(m × log n) | m = matches, n = price levels |
| Get best bid/ask | O(1) | BTreeMap first/last |
| Get order by ID | O(1) | HashMap lookup |
| FIFO at price level | O(1) | VecDeque front/back |

### Memory Usage

**Per order:** ~200 bytes
- Order struct: ~120 bytes
- HashMap entry: ~40 bytes
- BTreeMap entry: ~40 bytes

**Example capacity:**
- 1 GB RAM → ~5 million orders
- 10 GB RAM → ~50 million orders

### Throughput Estimates

**Single-threaded engine:**
- Simple limit order: ~100,000 ops/sec
- Complex matching: ~50,000 ops/sec
- OrderBook queries: ~200,000 ops/sec

**Bottlenecks:**
- BTreeMap operations (logarithmic)
- Trade settlement (balance updates)
- Serialization/deserialization (JSON)

**Note:** These are rough estimates; actual performance depends on hardware and workload.

---

## Known Limitations

### 1. In-Memory Storage Only

**Issue:** All data stored in RAM, lost on shutdown.

**Impact:**
- No persistence across restarts
- Limited by available memory
- No audit trail

**Future:** Add PostgreSQL or Redis persistence layer.

---

### 2. Market Order Balance Check Skipped

**Issue:** Market orders don't pre-validate balance (see `src/engine/engine.rs:86`).

**Impact:**
- May fail mid-execution if user lacks funds
- Could lead to partial fills being rejected

**Future:** Estimate required balance from orderbook depth.

---

### 3. Single Trading Pair

**Issue:** Hardcoded BTC/USD only.

**Impact:**
- Cannot trade other pairs (ETH/USD, etc.)
- Need to run multiple engines for multiple pairs

**Future:** Add pair parameter to orderbook, support multiple pairs.

---

### 4. JWT Secret Hardcoded

**Issue:** Secret key hardcoded in `src/utils/auth.rs:11`.

**Impact:**
- Security risk if code exposed
- Cannot rotate keys

**Future:** Use environment variable (`JWT_SECRET`).

---

### 5. No WebSocket Support

**Issue:** Only HTTP polling for orderbook updates.

**Impact:**
- Higher latency for real-time data
- More network overhead

**Future:** Add WebSocket endpoint for subscriptions.

---

### 6. No Order History

**Issue:** Cannot query past trades or cancelled orders.

**Impact:**
- Limited analytics
- No audit trail

**Future:** Add trade history storage and query endpoint.

---

### 7. Fixed OrderBook Depth

**Issue:** Depth query always returns 10 levels.

**Impact:**
- Cannot customize depth
- May be insufficient for analysis

**Future:** Add `depth` query parameter.

---

## Future Enhancements

### High Priority
- [ ] Database persistence (PostgreSQL)
- [ ] Order history and trade log
- [ ] WebSocket real-time updates
- [ ] Environment-based configuration
- [ ] Market order balance estimation
- [ ] Request rate limiting

### Medium Priority
- [ ] Multiple trading pairs
- [ ] Stop-loss / take-profit orders
- [ ] Order book snapshots
- [ ] Admin dashboard
- [ ] Performance metrics (Prometheus)
- [ ] Comprehensive integration tests

### Low Priority
- [ ] OCO (One-Cancels-Other) orders
- [ ] Iceberg orders
- [ ] Fee structure
- [ ] Maker/taker rebates
- [ ] Trading API keys (separate from user auth)
- [ ] Deposit/withdrawal workflow

---

## Contributing

Contributions are welcome! Please follow these guidelines:

1. **Fork the repository**
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Write tests** for new functionality
4. **Ensure all tests pass** (`cargo test`)
5. **Format code** (`cargo fmt`)
6. **Run linter** (`cargo clippy`)
7. **Commit changes** (`git commit -m 'Add amazing feature'`)
8. **Push to branch** (`git push origin feature/amazing-feature`)
9. **Open a Pull Request**

### Code Style

- Follow Rust naming conventions (snake_case for functions/variables)
- Add documentation comments for public APIs
- Keep functions focused and small
- Prefer explicit error handling over panics

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

- Built with [Actix-Web](https://actix.rs/) - Fast, pragmatic web framework
- Powered by [Tokio](https://tokio.rs/) - Asynchronous runtime for Rust
- Inspired by real-world cryptocurrency exchange architectures

---

## Contact

For questions or support:
- Open an issue on GitHub
- Email: your.email@example.com
- Discord: Your#1234

---

## Additional Documentation

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Detailed system architecture
- **[engine.md](engine.md)** - OrderBook engine implementation details

---

**Built with ❤️ and Rust**
