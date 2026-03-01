# UCEL-DECIMAL-CODE-CORE-001 Verification

## 1) Changed files（git diff --name-only + untracked）
- docs/status/trace-index.json
- docs/verification/UCEL-DECIMAL-CODE-CORE-001.md
- ucel/crates/ucel-core/Cargo.toml
- ucel/crates/ucel-core/src/lib.rs
- ucel/crates/ucel-core/src/types.rs
- ucel/crates/ucel-core/src/decimal/mod.rs
- ucel/crates/ucel-core/src/decimal/guard.rs
- ucel/crates/ucel-core/src/decimal/tick_step.rs
- ucel/crates/ucel-core/src/decimal/policy.rs
- ucel/crates/ucel-core/src/decimal/serde.rs
- ucel/crates/ucel-core/src/value/mod.rs
- ucel/crates/ucel-core/src/value/tick_step.rs
- ucel/crates/ucel-core/src/value/price.rs
- ucel/crates/ucel-core/src/value/qty.rs
- ucel/crates/ucel-core/src/value/notional.rs
- ucel/crates/ucel-core/src/order_gate/mod.rs
- ucel/crates/ucel-core/src/order_gate/gate.rs
- ucel/crates/ucel-core/tests/order_gate_enforcement.rs
- ucel/crates/ucel-core/tests/value_class_serde.rs

## 2) What / Why
- Added `ucel-core` Decimal SSOT modules (`decimal::{guard, policy, serde, tick_step}`) to centralize guard, rounding, tick/step validation, and quantization behavior.
- Added value newtypes (`Price`, `Qty`, `Notional`, `TickSize`, `StepSize`) and `OrderGate` to enforce execution boundary checks/quantization.
- Replaced `FillEvent` and `Balance` floating-point fields with `Decimal` to remove `f64` from `ucel-core` trade/account boundary structs.
- Added serde and order-gate regression tests to lock policy behavior for zero/negative guard and tick/step enforcement.
- Updated trace-index task entry with branch, artifacts, and verification evidence to keep task traceability consistent.

## 3) Self-check results
- Allowed-path check OK
  - Only `docs/**` and `ucel/crates/**` were modified.
- Tests added/updated OK
  - Added `ucel/crates/ucel-core/tests/order_gate_enforcement.rs`
  - Added `ucel/crates/ucel-core/tests/value_class_serde.rs`
- Build/Unit test command results
  - `cargo test -p ucel-core` => PASS
- trace-index json.tool OK
  - `python -m json.tool docs/status/trace-index.json > /dev/null` => PASS
- Secrets scan
  - Checked changed files for key-like patterns => PASS (no findings)
- docsリンク存在チェック（今回触った docs の追加参照のみ）
  - Added refs under updated trace-index entry validated => PASS
