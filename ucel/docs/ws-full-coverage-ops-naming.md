# WS Coverage Op-ID Naming Conventions

This document defines how WS `op_id` strings map to source filenames and module names.

## Rule

Given `op_id` (example: `crypto.public.ws.orderbook.diff`):

1. Lowercase the string.
2. Replace `.` with `_`.
3. Replace `-` with `_`.
4. Collapse repeated `_` into a single `_`.
5. Trim leading/trailing `_`.

Result:

- `crypto.public.ws.orderbook.diff` -> `crypto_public_ws_orderbook_diff`

## File layout

For each exchange crate:

- `src/channels/mod.rs` exports supported ops and dispatch.
- `src/channels/<normalized_op_id>.rs` contains handler/payload mapping for that op.

## Examples

- `crypto.public.ws.trade` -> `crypto_public_ws_trade.rs`
- `crypto.public.ws.kline-1m` -> `crypto_public_ws_kline_1m.rs`
- `crypto.private.ws.balance.update` -> `crypto_private_ws_balance_update.rs`

## Collision handling

If two op IDs normalize to the same filename, append a deterministic suffix:

- `<normalized>__v2`, `<normalized>__v3`, ...

`channels/mod.rs` must retain explicit mapping from original `op_id` to file module.

## Supported ops declaration

Each `ucel-cex-*` crate exposes:

- `supported_ws_ops() -> Vec<&'static str>`

Values must be original op IDs from coverage (not normalized names) to keep coverage-gate exact.
