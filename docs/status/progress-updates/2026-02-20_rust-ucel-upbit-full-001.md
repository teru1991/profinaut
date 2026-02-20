# RUST-UCEL-UPBIT-FULL-001 progress

## Catalog evidence summary
- Source SSOT: `docs/exchanges/upbit/catalog.json`
- Catalog row count:
  - REST: 22
  - WS: 7
  - Total tracked rows: 29

## Implemented in this task
- `ucel-registry` now loads and validates Upbit catalog rows with fail-fast checks:
  - duplicate `id` across REST+WS rejected (`CATALOG_DUPLICATE_ID`)
  - required fields rejected when empty or missing (`CATALOG_MISSING_FIELD`)
  - malformed REST `method/base_url/path` and malformed `ws_url` rejected (`CATALOG_INVALID`)
  - `visibility` and `requires_auth` contradiction rejected (`CATALOG_INVALID`)
- OpName SSOT rule is centralized in one mapping function for Upbit IDs.
- `requires_auth` is mechanically derived from `visibility=private` (no inference).
- `ucel-testkit` contract index now includes Upbit coverage tests so every catalog `id` is test-addressable.
- Added `ucel/coverage/upbit.yaml` with all 29 catalog IDs and `implemented/tested` flags.
- Coverage gate mode for Upbit is warn-only in this task (`strict: false`) to be flipped in Task3.

## Mapping rule (single source)
- Upbit `id` is mapped deterministically by id segments:
  - WS `ticker/trade/orderbook/candle/myorder/myasset` prefixes map to corresponding subscribe/fetch ops.
  - REST `orders.create/cancel/open/closed`, `accounts`, `ticker`, `trades`, `orderbook`, `candles` map to trading/market ops.
  - unmatched rows fallback to `fetch_status`.

## Coverage gate design
- Manifest file: `ucel/coverage/upbit.yaml`
- Tracks *all* REST and WS IDs from catalog SSOT.
- Gate output categories: `implemented` and `tested` missing sets.
- Current mode: warn-only for bootstrapping.
- Next task declaration: **all 29 rows will be implemented/tested and gate will be switched to strict mode**.

## Perf baseline policy for Upbit rail
- Typed deserialize required (`serde_json::Value` as app-layer payload type is disallowed by policy).
- Bytes-first transport path to avoid unnecessary copies.
- WS ingestion must use bounded channels with explicit backpressure strategy.

