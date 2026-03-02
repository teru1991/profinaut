# RUST-UCEL-MARKETMETA-003 Verification

## Scope
- Connected `MarketMeta` constraints with `ucel-core` `OrderGate` quantization through SDK normalize API.
- Fixed side-aware safe rounding contract in `OrderGate::recommended_modes`.
- Added regression tests to lock buy/sell price rounding and qty down-rounding behavior.

## Safety SSOT fixed in code/tests
- Limit Buy price: Down/Floor to tick.
- Limit Sell price: Up/Ceil to tick.
- Limit qty: Down/ToZero to step.

## Self-check
- Allowed-path OK (docs + ucel/crates only).
- No file deletions.
- Trace index updated only for `tasks["RUST-UCEL-MARKETMETA-003"]`.

## Commands and results
- `cargo fmt --manifest-path ucel/Cargo.toml --all -- --check`
  - preflight: failed due existing formatting drift at `ucel-cex-upbit` (outside task scope).
- `cargo test --manifest-path ucel/Cargo.toml -p ucel-core -p ucel-sdk`
  - pass.
- `cargo fmt --manifest-path ucel/Cargo.toml --all`
  - ran for pre-merge requirement.
- `cargo test --manifest-path ucel/Cargo.toml -p ucel-core -p ucel-sdk`
  - pass.
- `cargo test --manifest-path ucel/Cargo.toml -p ucel-symbol-core -p ucel-symbol-store`
  - pass.
- `python -m json.tool docs/status/trace-index.json > /dev/null`
  - pass.

## Notes
- `normalize_limit_with_meta` performs quantize via `OrderGate` and then validates `min_qty/max_qty/min_notional` via `MarketMeta::validate_order`.
- `normalize_limit_from_store` returns `MetaNotFound` when market metadata is unavailable.
