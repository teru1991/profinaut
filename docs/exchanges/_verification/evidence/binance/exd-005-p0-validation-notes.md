# Binance EXD-005 P0 Endpoint Validation Notes

- confirmed_date: 2026-02-21
- primary_reference:
  - https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints
  - https://developers.binance.com/docs/binance-spot-api-docs/rest-api/trading-endpoints
  - https://developers.binance.com/docs/binance-spot-api-docs/rest-api/account-endpoints
- source_material: official Binance spot docs mirror (`binance-spot-api-docs/rest-api.md`)

## P0 verification results

### 1) Ticker (`GET /api/v3/ticker/price`)
- required/optional:
  - `symbol` optional
  - `symbols` optional
- conditional rule:
  - both omitted => all symbols
  - both provided => error (`-1102`)
- response shape:
  - one symbol => object
  - multiple/all => array<object>
- unit/type: price string(decimal)

### 2) Orderbook (`GET /api/v3/depth`)
- required/optional:
  - `symbol` required
  - `limit` optional (default 100; max 5000)
- response fields:
  - `lastUpdateId` int
  - `bids`/`asks`: array of `[price, qty]` strings
- pagination: none (snapshot endpoint)

### 3) Place order (`POST /api/v3/order`, SIGNED)
- required common:
  - `symbol`, `side`, `type`, `timestamp`
- conditional required (per order type):
  - `LIMIT`: `timeInForce`, `quantity`, `price`
  - `MARKET`: `quantity` or `quoteOrderQty`
  - `STOP_LOSS`/`TAKE_PROFIT`: `quantity`, `stopPrice` or `trailingDelta`
  - `STOP_LOSS_LIMIT`/`TAKE_PROFIT_LIMIT`: `timeInForce`, `quantity`, `price`, and (`stopPrice` or `trailingDelta`)
- precision note:
  - `recvWindow` supports up to 3 decimals

### 4) Cancel order (`DELETE /api/v3/order`, SIGNED)
- required common:
  - `symbol`, `timestamp`
- conditional required:
  - `orderId` or `origClientOrderId` (either required)
  - `newClientOrderId` optional replacement for current cancel request

### 5) Balance (`GET /api/v3/account`, SIGNED)
- required/optional:
  - `timestamp` required
  - `recvWindow`, `omitZeroBalances` optional
- response fields:
  - `balances[]` with `asset`, `free`, `locked`
  - commission and account flags

## Open implementation clarifications
1. `ticker/price` response polymorphism (object vs array) should be explicitly handled in implementation.
2. `new order` has many conditional required fields; client-side validator should branch by `type`.
3. `cancel order` should enforce OR-condition (`orderId` OR `origClientOrderId`).
