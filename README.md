# Demo-OrderBook

> A **learning-grade** limit-order matching engine. Not a real exchange. Not to be trusted with real money.
> Name it to claim it: this is a **demo**.

What it is: a compact, typed, actor-model matching engine that looks simple —
until you open it. Signup, on-ramp, place, match, cancel. The interesting
parts are underneath.

```
  HTTP (actix)  ──┐
                  ├──►  mpsc channel  ──►  engine task  ──► OrderBook
  HTTP (actix)  ──┤                                           ├─ bids: BTreeMap<Reverse<Price>, Level>
  HTTP (actix)  ──┘                                           ├─ asks: BTreeMap<Price, Level>
                                                              └─ orders: HashMap<Uuid, OrderLocation>
```

Many workers push commands; one owner drains them. No shared mutable state,
no mutexes — the ownership model is the concurrency model.

## What's inside

- **Matching**: limit + market, price–time (FIFO) priority, partial fills, side-agnostic core
- **Money**: integer minor units throughout (USD millionths, BTC satoshis) — no f64 drift
- **Auth**: bcrypt + JWT, users persisted in Postgres via SQLx
- **Book**: `BTreeMap` for price levels (`Reverse<Price>` on bids so best = top on both sides)
- **Cancellation**: `HashMap<Uuid, OrderLocation>` index, `O(log n)` lookup
- **Errors**: typed `OrderBookError` enum, mapped to 400 / 403 / 404 automatically

## Run it

```bash
docker compose up -d          # Postgres on :5432
cp .env.example .env
cargo run                      # migrates + serves on :8080
```

## Talk to it

```bash
# Signup -> token
curl -X POST localhost:8080/api/auth/signup \
  -H 'content-type: application/json' \
  -d '{"username":"alice","email":"a@x.io","password":"secret1"}'

# On-ramp, place, match
TOKEN=...
curl -X POST localhost:8080/api/user/onramp \
  -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"currency":"USD","amount":100000}'

curl -X POST localhost:8080/api/orders/limit \
  -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"side":"buy","price":50000,"quantity":0.5}'

curl localhost:8080/api/orderbook
```

## What this is *not*

- Not a real exchange. No KYC, no custody, no settlement risk, no legal compliance.
- No stop / take-profit / conditional orders — only limit and market.
- No persisted orders across restart — the book lives in memory.
- Single trading pair (BTC/USD), single writer, no clustering.
- Do not use this to move real money. Really.

Built to learn trading-engine architecture in Rust. Everything else
— the interesting parts — is here.
