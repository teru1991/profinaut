# UCEL-AUDIT-FIX-006 Verification

## 1) Changed files
- (paste) `git diff --name-only`
- docs/specs/ucel/coverage_strict_policy_v1.md
- docs/specs/ucel/market_data_coverage_scope_policy_v1.md
- docs/specs/ucel/ssot_coverage_schema_v1.md
- docs/specs/ucel/ssot_gate_spec_v1.md
- docs/specs/ucel/ssot_integrity_gate_v2.md
- docs/specs/ucel/ty_100_definition_spec_v1.md
- docs/status/trace-index.json
- docs/verification/UCEL-AUDIT-FIX-006.md
- ucel/coverage/coverage_v2/exchanges/deribit.json
- ucel/crates/ucel-cex-deribit/src/channels/mod.rs
- ucel/crates/ucel-testkit/src/coverage_gate.rs
- ucel/crates/ucel-testkit/tests/discoverability_coverage_v2_gate.rs
- ucel/docs/policies/coverage_policy.md

## 2) What / Why
- deribit の `supported_ws_ops()` が空だったため、公開WSの最小実装セット（book/ticker/trades）を追加して discoverability を埋めた。
- coverage_v2 の deribit も public.ws=true + ws_ops を同じ最小セットに更新し、宣言と実装の整合を確保した。
- testkit に `public_ws_enabled()` を追加し、新規 Gate テストで「public.ws=true なのに supported_ws_ops が空」を CI で検知可能にした。
- v1 前提 docs/specs は削除せず、冒頭に LEGACY/NOT USED を追記して混乱を防止した。
- バイナリ資産は追加していない（text only）。

## 3) Self-check results
- Allowed-path check OK:
  - 許可パス外の変更は無し（`ucel/**`, `docs/**` のみ）
- Tests added/updated OK:
  - `ucel/crates/ucel-testkit/tests/discoverability_coverage_v2_gate.rs`
- Build/Test commands:
  - `cargo test --manifest-path ucel/Cargo.toml -p ucel-testkit --all-targets`（失敗: 既存 `ssot_gate_test` deribit coverage.entries不足）
  - `cargo test --manifest-path ucel/Cargo.toml -p ucel-cex-deribit --all-targets`（成功）
  - `cargo test --manifest-path ucel/Cargo.toml -p ucel-cex-coincheck --all-targets`（成功）
- fmt/clippy:
  - `cargo fmt --manifest-path ucel/Cargo.toml --all -- --check`（失敗: 既存 drift）
  - `cargo clippy --manifest-path ucel/Cargo.toml --workspace --all-targets -- -D warnings`（失敗: 既存 `ucel-transport` テスト型不一致）
- Secrets scan:
  - `git diff | rg -n "(api_key|secret|token|passphrase|PRIVATE KEY)"`（追加なし）
- Binary file check:
  - `git diff --name-only --diff-filter=A | rg -n "\.(png|jpg|jpeg|gif|zip|bin|exe|dll|so|dylib)$"`（ヒット無し）

## 4) ★History evidence (required)
- `git log --oneline --decorate -n 50` / `--graph` / `--merges` / `git show HEAD` / `git reflog -n 30` を確認。
  - 直前HEAD: `015a1f0`（前回実装コミット）。
  - 系譜上、`5a39164` で discoverability / coverage_v2 整備方針が入っているため、本修正はその不足補完として整合。
- `git merge-base HEAD origin/...` は、環境に `origin` remote が無いため算出不可（コマンド実行証跡あり）。
- `git log/blame` 要点:
  - `ucel-cex-deribit/src/channels/mod.rs` は空 `vec![]` のまま残っていた。
  - `ucel-testkit/src/coverage_gate.rs` は coverage_v2読み取り基盤が既にあり、`public_ws_enabled` 追記は既存意図に沿う最小拡張。

