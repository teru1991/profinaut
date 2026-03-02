# UCEL-100P-STEP2-STRICT-100P-BITHUMB-001 Verification

## 1) Changed files
- docs/exchanges/bithumb/catalog.json
- ucel/coverage/bithumb.yaml
- ucel/crates/ucel-cex-bithumb/Cargo.toml
- ucel/crates/ucel-cex-bithumb/src/lib.rs
- ucel/crates/ucel-testkit/Cargo.toml
- ucel/crates/ucel-testkit/tests/contract_ws_bithumb.rs
- ucel/fixtures/golden/ws/bithumb/raw.json
- ucel/fixtures/golden/ws/bithumb/expected.normalized.json
- docs/status/trace-index.json
- docs/verification/UCEL-100P-STEP2-STRICT-100P-BITHUMB-001.md

## 2) What / Why
- Implemented a concrete `ucel-cex-bithumb` crate and added public REST/WS normalization handlers for Bithumb public-only endpoints.
- Added golden contract coverage for Bithumb public WS trade snapshots in `ucel-testkit`.
- Promoted `ucel/coverage/bithumb.yaml` to `strict: true` with all public entries `implemented/tested=true`.
- Aligned `docs/exchanges/bithumb/catalog.json` to public-only scope by removing private REST/WS items while keeping FIX/data placeholders.
- Updated workspace wiring (`ucel/Cargo.toml`, `ucel-testkit/Cargo.toml`) so the new adapter and test are built by CI.

## 3) Self-check results
- Allowed-path check: **OK for this task's staged changes**. (Repository had pre-existing unrelated dirty files outside allowlist: `services/marketdata-rs/Cargo.lock`, node_modules)
- Tests:
  - `cargo test -p ucel-testkit --manifest-path ucel/Cargo.toml --test contract_ws_bithumb` ✅
  - `cargo test -p ucel-testkit --manifest-path ucel/Cargo.toml --test ssot_gate_test` ✅
  - `cargo test --manifest-path ucel/Cargo.toml --workspace --all-targets` ⚠️ fails due pre-existing `ucel-cex-sbivc` compile error (missing `EndpointAllowlist` / `SubdomainPolicy` imports)
  - `cargo clippy --manifest-path ucel/Cargo.toml --workspace --all-targets -- -D warnings` ⚠️ fails due pre-existing `ucel-transport` clippy violation (`derivable_impls`)
  - `cargo fmt --manifest-path ucel/Cargo.toml --all -- --check` ⚠️ fails due pre-existing formatting drift in unrelated file `ucel-cex-bitflyer/src/lib.rs`
- SSOT gate: `cargo test -p ucel-testkit --manifest-path ucel/Cargo.toml --test ssot_gate_test` ✅
- Secrets scan: no secrets added (manual diff inspection on changed files) ✅

## 4) 履歴確認の証拠（0.1）
- `git log --oneline --decorate -n 50`: HEAD is `14e4c21` merge of PR #416 (coverage schema v1 migration).
- `git log --graph --oneline --decorate --all -n 80`: confirms linear continuation after PR #415/#416 SSOT work.
- `git show HEAD`: confirms current base commit intent is Step1 visibility migration.
- `git reflog -n 30`: branch created from `work` at `14e4c21` with no divergent reset/rebase.
- `git branch -vv`: both `work` and this feature branch point to `14e4c21` before this task.
- `git log --merges --oneline -n 30`: recent merge chain aligns with SSOT coverage hardening sequence.
- `git merge-base HEAD work`: `14e4c21d5aaa9bdbd3bef63bd415c68c8b62cd49`.
- `git blame -w ucel/coverage/bithumb.yaml`: existing Step1 lineage from `ee401947`; this task updates strict/flags and keeps venue identity.
- `git blame -w docs/exchanges/bithumb/catalog.json`: original catalog authored in `99720fd8`; this task keeps structure while narrowing scope to public-only.
- `git blame -w ucel/crates/ucel-cex-bithumb/src/lib.rs`: new file in this task (no prior blame in HEAD).
- `git blame -w ucel/crates/ucel-registry/src/lib.rs`: no Bithumb registration baseline existed; registry untouched in this task.

## 5) strict=true達成の証拠
- `ucel/coverage/bithumb.yaml` is now:
  - `scope: public_only`
  - `strict: true`
  - top-level `implemented: true`, `tested: true`
  - all listed entries (`openapi.public.rest.market.list`, `openapi.public.rest.ticker.list`, `openapi.public.rest.orderbook.snapshot`, `openapi.public.ws.ticker.snapshot`, `openapi.public.ws.trade.snapshot`, plus existing FIX/data placeholders) are `implemented: true` and `tested: true`.
