# UCEL-MARKETMETA-001 Verification

## 1) Changed files
```
docs/status/trace-index.json
services/marketdata-rs/Cargo.lock
ucel/Cargo.lock
ucel/crates/ucel-cex-binance-coinm/Cargo.toml
ucel/crates/ucel-cex-binance-coinm/src/symbols.rs
ucel/crates/ucel-cex-binance-options/Cargo.toml
ucel/crates/ucel-cex-binance-options/src/symbols.rs
ucel/crates/ucel-cex-binance-usdm/Cargo.toml
ucel/crates/ucel-cex-binance-usdm/src/symbols.rs
ucel/crates/ucel-cex-binance/src/symbols.rs
ucel/crates/ucel-cex-bitbank/Cargo.toml
ucel/crates/ucel-cex-bitbank/src/symbols.rs
ucel/crates/ucel-cex-bitflyer/Cargo.toml
ucel/crates/ucel-cex-bitflyer/src/symbols.rs
ucel/crates/ucel-cex-bitget/Cargo.toml
ucel/crates/ucel-cex-bitget/src/symbols.rs
ucel/crates/ucel-cex-bitmex/Cargo.toml
ucel/crates/ucel-cex-bitmex/src/symbols.rs
ucel/crates/ucel-cex-bittrade/Cargo.toml
ucel/crates/ucel-cex-bittrade/src/symbols.rs
ucel/crates/ucel-cex-bybit/Cargo.toml
ucel/crates/ucel-cex-bybit/src/symbols.rs
ucel/crates/ucel-cex-coinbase/Cargo.toml
ucel/crates/ucel-cex-coinbase/src/symbols.rs
ucel/crates/ucel-cex-coincheck/Cargo.toml
ucel/crates/ucel-cex-coincheck/src/symbols.rs
ucel/crates/ucel-cex-deribit/Cargo.toml
ucel/crates/ucel-cex-deribit/src/symbols.rs
ucel/crates/ucel-cex-gmocoin/src/symbols.rs
ucel/crates/ucel-cex-htx/Cargo.toml
ucel/crates/ucel-cex-htx/src/symbols.rs
ucel/crates/ucel-cex-kraken/Cargo.toml
ucel/crates/ucel-cex-kraken/src/symbols.rs
ucel/crates/ucel-cex-okx/Cargo.toml
ucel/crates/ucel-cex-okx/src/symbols.rs
ucel/crates/ucel-cex-sbivc/Cargo.toml
ucel/crates/ucel-cex-sbivc/src/symbols.rs
ucel/crates/ucel-cex-upbit/Cargo.toml
ucel/crates/ucel-cex-upbit/src/symbols.rs
ucel/crates/ucel-symbol-adapter/src/lib.rs
ucel/crates/ucel-symbol-core/src/market_meta.rs
ucel/crates/ucel-symbol-store/src/lib.rs
```

## 2) What / Why
- Added additive MarketMeta derivation support from `StandardizedInstrument` and lightweight validation helpers.
- Synced derived MarketMeta cache in `SymbolStore` so snapshot add/update/remove keeps SSOT market metadata aligned.
- Added adapter-level `MarketMetaFetcher` + snapshot conversion helper so connectors can expose market meta without breaking existing symbol interfaces.
- Implemented `fetch_market_meta()` for Binance spot and GMO Coin; added explicit `NotSupported` stubs for remaining specified connectors.
- Updated trace index task entry and captured this verification record for reviewer handoff.

## 3) Self-check results
- Allowed-path check: NG (pre-existing non-allowlisted dirty file: services/marketdata-rs/Cargo.lock)
- Tests added/updated: 
  - `ucel-symbol-core`: conversion test `market_meta_is_derived_from_standardized_instrument`
  - `ucel-symbol-store`: snapshot sync test `market_meta_is_synced_on_apply_snapshot`
- Build/Unit test commands:
  - `cargo test -q` => pass
  - `python -m json.tool docs/status/trace-index.json > /dev/null` => pass
- trace-index json.tool: OK
- Secrets scan (simple): `rg -n "(AKIA|SECRET|TOKEN|PASSWORD)"` over changed files => no hits
- docs link check: `docs/verification/UCEL-MARKETMETA-001.md` exists
