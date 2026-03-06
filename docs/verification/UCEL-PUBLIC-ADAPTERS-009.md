# UCEL-PUBLIC-ADAPTERS-009 Verification

## 1) Changed files (`git diff --name-only --cached`)
- docs/specs/ucel/public_adapter_surface_v1.md
- docs/status/trace-index.json
- ucel/Cargo.lock
- ucel/crates/ucel-core/src/lib.rs
- ucel/crates/ucel-core/src/market_data.rs
- ucel/crates/ucel-registry/src/hub/rest.rs
- ucel/crates/ucel-registry/src/hub/ws.rs
- ucel/crates/ucel-sdk/Cargo.toml
- ucel/crates/ucel-sdk/src/lib.rs
- ucel/crates/ucel-sdk/src/market_data.rs
- ucel/crates/ucel-testkit/src/lib.rs
- ucel/crates/ucel-testkit/src/market_data.rs
- ucel/crates/ucel-testkit/tests/public_adapter_contract_matrix.rs
- ucel/crates/ucel-testkit/tests/public_rest_market_data.rs
- ucel/crates/ucel-testkit/tests/public_symbol_meta_links.rs
- ucel/crates/ucel-testkit/tests/public_ws_orderbook_integrity.rs
- ucel/crates/ucel-testkit/tests/public_ws_subscribe_and_resume.rs
- ucel/crates/ucel-transport/src/ws/mod.rs
- ucel/crates/ucel-transport/src/ws/public_runtime.rs
- ucel/crates/ucel-ws-rules/Cargo.toml
- ucel/crates/ucel-ws-rules/src/lib.rs
- ucel/crates/ucel-ws-rules/src/public_rules.rs
- ucel/docs/exchanges/public_adapter_matrix.md
- ucel/docs/marketdata/public_adapter_policy.md
- ucel/fixtures/market_data/README.md
- docs/verification/UCEL-PUBLIC-ADAPTERS-009.md

## 2) What / Why
- Added SSOT docs for canonical public adapter surface, runtime reason codes, and support policy.
- Added canonical market-data models and validation/apply helpers in `ucel-core`.
- Added initial public WS runtime contract traits and session/resume state in transport.
- Added SDK public `MarketDataFacade` for REST/WS market-data access entry points.
- Added ws-rules public view helper to expose heartbeat/resume/integrity defaults.
- Added testkit market-data harness and five new regression/contract tests.

## 3) Self-check results
- Allowed-path check OK
  - `git diff --name-only --cached | awk ...` => no output.
- Tests added/updated OK
  - `public_adapter_contract_matrix`
  - `public_rest_market_data`
  - `public_ws_subscribe_and_resume`
  - `public_ws_orderbook_integrity`
  - `public_symbol_meta_links`
- Build/Unit test command results
  - `cd ucel && cargo test -p ucel-core -p ucel-ws-rules -p ucel-transport -p ucel-sdk -p ucel-testkit public_adapter_contract_matrix -- --nocapture` => OK (build + filtered suites passed).
  - `cd ucel && cargo test -p ucel-testkit --test public_adapter_contract_matrix -- --nocapture` => 1 passed.
  - `cd ucel && cargo test -p ucel-testkit --test public_rest_market_data -- --nocapture` => 1 passed.
  - `cd ucel && cargo test -p ucel-testkit --test public_ws_subscribe_and_resume -- --nocapture` => 1 passed.
  - `cd ucel && cargo test -p ucel-testkit --test public_ws_orderbook_integrity -- --nocapture` => 1 passed.
  - `cd ucel && cargo test -p ucel-testkit --test public_symbol_meta_links -- --nocapture` => 1 passed.
- trace-index json.tool OK
  - `python -m json.tool docs/status/trace-index.json > /dev/null` => OK.
- Secrets scan
  - `rg -n "AKIA|SECRET_KEY|BEGIN PRIVATE KEY|api_key|api-secret" ...` => no hits.
- docsリンク存在チェック（今回触った docs 内の `docs/` 参照）
  - New docs added in this task do not contain internal `docs/` links, so no dangling references.

## 4) 履歴確認の証拠
- git log / merges / merge-base
  - `git log --oneline --decorate -n 20` confirmed latest merge train through `2d83fec` (PR #461 private WS scaffold) and `9a8fb42` (private REST + policy).
  - `git log --graph --oneline --decorate --all -n 40` confirmed sequence and no divergent local topic branch for these files.
  - `git log --merges --oneline -n 10` shows recent merge chain (#455-#461).
  - `git merge-base HEAD origin/master` could not run because no `origin/master` ref exists in this environment.
- target file history
  - `git show 2d83fec --stat` and `git show 1f72be0 --stat` show prior work established private WS runtime scaffolding in hub/ws, transport/ws, ws-rules, sdk/testkit.
  - `git show 9a8fb42 --stat` shows prior private REST/policy scaffolding in hub/rest and docs.
  - `git blame -w ucel/crates/ucel-registry/src/hub/ws.rs` indicates foundational ws subscribe loop from `6883c33` with private-policy additions in `1f72be0`.
- venue/family public adapter棚卸し（本タスク実装方針）
  - Existing repository already has many venue crates; this task added canonical public contracts + matrix/policy + test harness to normalize capability declarations.
  - Public-only venues are explicitly represented as `Supported` or `Partial` in the matrix/testkit and not auto-downgraded by private-surface absence.
- heartbeat / ack / reconnect / checksum / gap handling 根拠
  - Added explicit `PublicWsAckMode`, `PublicWsIntegrityMode`, `PublicWsReasonCode`, and runtime trait contracts in core/transport/ws-rules so venue-specific logic can plug into shared semantics.
- symbol/meta 連携根拠
  - SDK facade includes `list_symbols` and `get_market_meta` canonical endpoints; tests include symbol-family presence checks in matrix and avoid raw payload leakage.
- 不足に対する追加実装
  - Historical inspection showed private runtime existed but public runtime contract was missing; added reusable public model/runtime facade + tests and documented partial-support semantics instead of pretending unsupported private venues are unusable.
