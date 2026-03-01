# UCEL-DECIMAL-CODE-SYMBOL-002 Verification

## 1) Changed files（git diff --name-only + untracked）
- docs/status/trace-index.json
- docs/verification/UCEL-DECIMAL-CODE-SYMBOL-002.md
- ucel/crates/ucel-symbol-core/Cargo.toml
- ucel/crates/ucel-symbol-core/src/lib.rs

## 2) What / Why
- Unified symbol-layer Decimal SSOT ownership by removing direct `rust_decimal` dependency from `ucel-symbol-core` and depending on `ucel-core` instead.
- Switched symbol core imports to `ucel_core::Decimal` and `ucel_core::decimal::*` so rounding/tick/step behavior is delegated to `DecimalPolicy`.
- Added relaxed-policy helpers in symbol core for observation/meta use cases where zero/negative values may be present.
- Replaced local rounding implementation with `DecimalPolicy::round_price` / `round_qty` delegation.
- Added additive tick/step validate+quantize helper APIs that mirror core policy interfaces.

## 3) Self-check results
- Allowed-path check OK
  - Only `docs/**` and `ucel/crates/**` were modified.
- Tests added/updated OK
  - Updated existing unit tests in `ucel/crates/ucel-symbol-core/src/lib.rs` to cover validate/quantize helpers.
- Build/Unit test command results
  - `cargo test -p ucel-symbol-core` => PASS
- trace-index json.tool OK
  - `python -m json.tool docs/status/trace-index.json > /dev/null` => PASS
- Secrets scan
  - Checked changed files for key-like patterns => PASS (no findings)
- docsリンク存在チェック（今回触った docs の追加参照のみ）
  - Added refs under updated trace-index entry validated => PASS
