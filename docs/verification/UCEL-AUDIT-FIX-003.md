# UCEL-AUDIT-FIX-003 Verification

## 1) Changed files
- (paste) `git diff --name-only`

```
docs/status/trace-index.json
docs/verification/UCEL-AUDIT-FIX-003.md
ucel/Cargo.lock
ucel/coverage/coverage_v2/exchanges/binance-coinm.json
ucel/coverage/coverage_v2/exchanges/binance-options.json
ucel/coverage/coverage_v2/exchanges/binance-usdm.json
ucel/coverage/coverage_v2/exchanges/binance.json
ucel/coverage/coverage_v2/exchanges/bitbank.json
ucel/coverage/coverage_v2/exchanges/bitflyer.json
ucel/coverage/coverage_v2/exchanges/bitget.json
ucel/coverage/coverage_v2/exchanges/bithumb.json
ucel/coverage/coverage_v2/exchanges/bitmex.json
ucel/coverage/coverage_v2/exchanges/bittrade.json
ucel/coverage/coverage_v2/exchanges/bybit.json
ucel/coverage/coverage_v2/exchanges/coinbase.json
ucel/coverage/coverage_v2/exchanges/coincheck.json
ucel/coverage/coverage_v2/exchanges/deribit.json
ucel/coverage/coverage_v2/exchanges/gmocoin.json
ucel/coverage/coverage_v2/exchanges/htx.json
ucel/coverage/coverage_v2/exchanges/kraken.json
ucel/coverage/coverage_v2/exchanges/okx.json
ucel/coverage/coverage_v2/exchanges/sbivc.json
ucel/coverage/coverage_v2/exchanges/upbit.json
ucel/coverage/sbivc.yaml
ucel/crates/ucel-cex-binance-coinm/src/channels/mod.rs
ucel/crates/ucel-cex-binance-options/src/channels/mod.rs
ucel/crates/ucel-cex-binance-usdm/src/channels/mod.rs
ucel/crates/ucel-cex-bitget/src/channels/mod.rs
ucel/crates/ucel-cex-bitmex/src/channels/mod.rs
ucel/crates/ucel-cex-bittrade/src/channels/mod.rs
ucel/crates/ucel-cex-bybit/src/channels/mod.rs
ucel/crates/ucel-cex-coinbase/src/channels/mod.rs
ucel/crates/ucel-cex-coincheck/src/channels/mod.rs
ucel/crates/ucel-cex-htx/src/channels/mod.rs
ucel/crates/ucel-cex-kraken/src/channels/mod.rs
ucel/crates/ucel-cex-okx/src/channels/mod.rs
ucel/crates/ucel-cex-sbivc/Cargo.toml
ucel/crates/ucel-cex-sbivc/src/channels/mod.rs
ucel/crates/ucel-cex-upbit/src/channels/mod.rs
ucel/crates/ucel-testkit/Cargo.toml
ucel/crates/ucel-testkit/src/coverage_gate.rs
ucel/crates/ucel-testkit/src/lib.rs
ucel/crates/ucel-testkit/tests/coverage_gate.rs
ucel/crates/ucel-testkit/tests/coverage_policy_sbivc.rs
ucel/docs/policies/coverage_policy.md
```

## 2) What / Why
- Discoverability（`supported_ws_ops`）が空のCEXを coverage 実態に合わせて埋め、capabilities 自己申告の欠落を解消した。
- coverage v1/v2 併存で完了判定が揺れるため、`ucel/coverage/coverage_v2/exchanges/*.json` を正本として追加し、`ucel-testkit` に Gate テストを追加して CI で検証可能化した。
- SBIVC は private docs 不足による一時的 `public_only` 例外を policy + test で固定し、国内で sbivc 以外が public_only にならないようにした。
- バイナリ資産は追加していない（text only）。

## 3) Self-check results
- Allowed-path check OK:
  - `git diff --name-only | awk ...` の結果 0。
- Tests added/updated OK:
  - `ucel/crates/ucel-testkit/tests/coverage_gate.rs`
  - `ucel/crates/ucel-testkit/tests/coverage_policy_sbivc.rs`
- Build/Test commands:
  - `cargo test --manifest-path ucel/Cargo.toml -p ucel-testkit --test coverage_gate -- --nocapture` => PASS
  - `cargo test --manifest-path ucel/Cargo.toml -p ucel-testkit --all-targets` => WARN（既存 `ssot_gate_test` が deribit coverage 欠落で失敗）
  - `cargo test --manifest-path ucel/Cargo.toml -p ucel-cex-coincheck --all-targets` => PASS
  - `cargo test --manifest-path ucel/Cargo.toml -p ucel-cex-coinbase -p ucel-cex-bybit -p ucel-cex-bitget -p ucel-cex-bitmex -p ucel-cex-bittrade -p ucel-cex-deribit -p ucel-cex-okx -p ucel-cex-htx -p ucel-cex-kraken -p ucel-cex-upbit -p ucel-cex-sbivc -p ucel-cex-binance-usdm -p ucel-cex-binance-coinm -p ucel-cex-binance-options --lib` => WARN（既存 `ucel-cex-bitget` strict coverage test failure）
- fmt/clippy:
  - `cargo fmt --manifest-path ucel/Cargo.toml --all -- --check` => WARN（既存 `ucel-transport` ファイルで失敗）
  - `cargo clippy --manifest-path ucel/Cargo.toml --workspace --all-targets -- -D warnings` => WARN（既存 `ucel-transport` test compile error）
- Secrets scan:
  - `git diff | rg -n "(api_key|secret|token|passphrase|PRIVATE KEY)"`（コード由来ヒット無し）
- Binary file check:
  - `git diff --name-only --diff-filter=A | rg -n "\.(png|jpg|jpeg|gif|zip|bin|exe|dll|so|dylib)$"`（ヒット無し）

## 4) ★History evidence (required)
- `git log --oneline --decorate -n 50` / `git log --graph --oneline --decorate --all -n 80`:
  - 直近は `643c6e2`（UCEL-AUDIT-FIX-002）、その前が `79765d3`（UCEL-AUDIT-FIX-001）。監査タスク連鎖（001→002→003）の意図と整合。
- `git log --merges --oneline -n 30`:
  - #426〜#410 の連続 merge を確認。
- `git show HEAD`:
  - 直前コミットが checkpoint/replay + resync 契約であり、本タスクの discoverability/coverage policy 固定と矛盾なし。
- `git reflog -n 30`:
  - `feature/ucel-audit-fix-002` から `feature/ucel-audit-fix-003` を作成した履歴を確認。
- `git merge-base ...`:
  - origin remote 未設定のため算出不可（環境制約）。

- Discoverability / coverage / policy 調査
  - `rg -n "supported_ws_ops\(|supported_http_ops\(" ucel/crates -S`: 複数 CEX の `supported_ws_ops` が `vec![]` だった。
  - `rg -n "coverage_v2|coverage_v1|coverage/.*\.yaml" ucel -S`: 既存は v1 YAML 主体で、v2 実体が未固定だった。
  - `rg -n "public_only|sbivc|SBI" ucel/docs ucel/coverage -S`: sbivc の public_only は存在するが、例外ポリシーの SSOT 文書が未整備だった。
  - 代表履歴: `ucel/crates/ucel-cex-coincheck/src/channels/mod.rs`
    - `git log --oneline -n 30` / `git blame -w` より初期 scaffolding 由来で空実装が継続していた。

## 5) Conclusions
- supported_ws_ops 空実装は未着手由来であり、coverage 連動 gate が弱かったため残存していた。
- coverage v1/v2 併存の運用意図は「v1実績 + v2将来SSOT」だったが、明文化不足だったため、本タスクで v2 SSOT + policy + gate を固定した。
