# RUST-UCEL-MARKETMETA-004 Verification

## Scope
- Added an SDK integration test that wires `MarketMetaService::preload()` with a mock `MarketMetaFetcher`, applies snapshot data into `MarketMetaStore`, then calls `normalize_limit_from_store` to verify end-to-end behavior.
- Locked both order-side rounding paths in one e2e test (`Buy` price down, `Sell` price up, quantity down to step).
- Added a negative-path e2e test that verifies missing metadata returns an error (`MetaNotFound` surfaced as `not found`).

## DoD mapping
1. Mock fetcher returns `MarketMetaSnapshot`.
   - Covered by `MockFetcher::fetch_market_meta_snapshot`.
2. `MarketMetaService.preload()` reflects snapshot in store.
   - Covered by calling `svc.preload().await` and normalizing from `svc.store()` with the same market id.
3. `normalize_limit_from_store` performs expected quantization/constraints.
   - Covered by Buy/Sell assertions on normalized price and quantity.
4. Failure case is fixed as regression.
   - Covered by `e2e_meta_not_found_is_error`.

## Commands and results
- `cargo fmt --manifest-path ucel/Cargo.toml --all`
  - pass.
- `cargo test --manifest-path ucel/Cargo.toml -p ucel-sdk -p ucel-core`
  - pass.
- `python -m json.tool docs/status/trace-index.json > /dev/null`
  - pass.

## Notes
- The e2e test is network-free and deterministic because it uses an in-process mock fetcher.
- No production behavior changes were required in `market_meta.rs` or `order_normalize.rs`; integration coverage was added in `ucel-sdk/tests`.
