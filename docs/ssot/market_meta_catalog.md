# MarketMeta Catalog SSOT

This catalog is the single source of truth for exchanges where **tick/step/min constraints cannot be obtained reliably from public REST**.

## Rules
- **No guessing**: If constraints are not confirmed, do not add an entry. UCEL must return `Err`.
- `tick_size` and `step_size` are required and must be positive.
- `min_qty` and `min_notional` are optional.
- Each entry key is unique by `(exchange, market_type, raw_symbol)`.

## Update procedure
1. Confirm official constraints (API docs / exchange UI / official support message).
2. Update `docs/ssot/market_meta_catalog.json` only.
3. Run:
   - `cd ucel && cargo test -p ucel-market-meta-catalog -q`
   - `cd ucel && cargo test --all-features -q`
4. Commit with a note referencing the evidence source in `note`.

## Consumers
- JP exchange connectors fallback to this catalog when public REST cannot provide constraints.
- If catalog entries for an exchange are absent, connector APIs return explicit `Err` (no silent skip).
