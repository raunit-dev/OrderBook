# orderbook

A limit-order matching engine that looks simple — until you open it.

On the surface: a few REST endpoints, signup, place an order, cancel.
Underneath: price–time priority, integer money, an actor-model engine,
and zero locks on the hot path.

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
docker compose up -d         # Postgres on :5432
cp .env.example .env
cargo run                     # migrates + serves on :8080
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

## Not yet

Multiple trading pairs. Persisted orders. A UI. Being an actual exchange.
Everything else — the interesting parts — is here.
