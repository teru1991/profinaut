# UCEL-DOMESTIC-PUBLIC-INVENTORY-009A Verification

## 1) Changed files (`git diff --name-only --cached`)
- docs/specs/ucel/domestic_public_api_inventory_v1.md
- docs/specs/ucel/domestic_public_surface_classification_v1.md
- docs/status/trace-index.json
- ucel/docs/exchanges/domestic_public_api_matrix.md
- ucel/docs/exchanges/domestic_public_endpoint_mapping.md
- ucel/docs/exchanges/domestic_public_scope_policy.md
- ucel/coverage_v2/domestic_public/jp_public_inventory.schema.json
- ucel/coverage_v2/domestic_public/jp_public_inventory.json
- ucel/crates/ucel-registry/src/hub/registry.rs
- ucel/crates/ucel-testkit/src/domestic_public_inventory.rs
- ucel/crates/ucel-testkit/src/lib.rs
- ucel/crates/ucel-testkit/tests/domestic_public_inventory_gate.rs
- ucel/crates/ucel-testkit/tests/domestic_public_surface_classification.rs
- ucel/crates/ucel-testkit/tests/domestic_public_docs_drift.rs
- ucel/examples/domestic_public_inventory_preview.rs

## 2) What / Why
- Repo evidence only で domestic venue public API inventory SSOT を新設した。
- catalog + coverage + coverage_v2 + ws_rules を証拠として `jp_public_inventory.json` を固定した。
- surface classification (core/extended/vendor/not_supported) の判定基準を spec 化した。
- inventory 漏れ/重複/docs drift を CI で落とす testkit gate を追加した。
- registry には runtime 影響のない public read helper (`list_public_rest_entries`, `list_public_ws_entries`) だけを最小差分追加した。

## 3) Self-check results
- Allowed-path check: staged diff は allowlist 内のみ。ワークツリーに task 外の既存変更 `services/marketdata-rs/Cargo.lock` が残存。
- Tests added/updated OK:
  - `domestic_public_inventory_gate`
  - `domestic_public_surface_classification`
  - `domestic_public_docs_drift`
- Build/Unit test command results:
  - `cd ucel && cargo test -p ucel-testkit --test domestic_public_inventory_gate -- --nocapture` ✅
  - `cd ucel && cargo test -p ucel-testkit --test domestic_public_surface_classification -- --nocapture` ✅
  - `cd ucel && cargo test -p ucel-testkit --test domestic_public_docs_drift -- --nocapture` ✅
  - `cd ucel && cargo test -p ucel-registry` ✅
  - `cd ucel && cargo test -p ucel-testkit` ✅
- trace-index json.tool OK:
  - `python -m json.tool docs/status/trace-index.json > /dev/null` ✅
- Secrets scan:
  - `rg -n 'AKIA|SECRET|PRIVATE KEY|BEGIN RSA|token' ...` で該当なし ✅
- docs リンク存在チェック（今回 docs 内の `docs/` 参照のみ）:
  - `docs/exchanges/<venue>/catalog.json` 参照は存在 ✅
  - `ucel/docs/policies/...` 参照は `docs/` 部分一致抽出では false-positive になったため手動確認で存在を確認 ✅

## 4) 履歴確認の証拠（必須）
- 実行コマンド:
  - `git log --oneline --decorate -n 50`
  - `git log --graph --oneline --decorate --all -n 80`
  - `git show 6ea3201 --stat --oneline`（直近 commit）
  - `git show 3d92e2c --stat --oneline`（registry.rs 最終更新）
  - `git blame -w ucel/Cargo.toml`
  - `git blame -w ucel/crates/ucel-registry/src/hub/mod.rs`
  - `git blame -w ucel/crates/ucel-registry/src/hub/registry.rs`
  - `git reflog -n 30`
  - `git merge-base HEAD origin/master`（remote 未設定で失敗）
  - `git branch -vv`
  - `git log --merges --oneline -n 30`
  - `git fetch --all --prune`
  - `git diff --name-only origin/master...HEAD`（remote 未設定で失敗）
  - `git log --oneline origin/master -- <paths>`（remote 未設定で失敗）
- 判定結果:
  - 国内取引所 crate（bitbank/bitflyer/coincheck/gmocoin/bittrade/sbivc）は `ucel/Cargo.toml` members と registry registration に存在し、削除/rename 痕跡は今回確認範囲でなし。
  - public inventory の既存 dedicated SSOT は無く、catalog + coverage + coverage_v2 を横断して新設が必要。
  - hotspot は `registry.rs`, `coverage_v2`, `ucel/docs/exchanges`。この task では registry は読み取り helper のみ追加し局所化。
  - `origin/master` がこの環境に無いため upstream 差分比較は実行不可（ローカル履歴ベースで矛盾確認）。

### Domestic scope 判定根拠
- workspace members: `ucel/Cargo.toml`
- registry include: `ucel/crates/ucel-registry/src/hub/registry.rs`
- exchange catalogs: `docs/exchanges/{bitbank,bitflyer,coincheck,gmocoin,bittrade,sbivc}/catalog.json`
- coverage evidence: `ucel/coverage/{bitbank,bitflyer,coincheck,gmocoin,bittrade,sbivc}.yaml`
- coverage_v2 evidence: `ucel/coverage/coverage_v2/exchanges/{...}.json`

### 各 venue の public API 棚卸し根拠
- source は全て `docs/exchanges/<venue>/catalog.json` の public visibility または public naming 規則。
- 実装状態は `ucel/coverage/<venue>.yaml` entry の implemented/tested から算出。
- WS runtime evidence は対応する `ucel/crates/ucel-ws-rules/rules/<venue>.toml`（存在時）を付与。

### classification 判断根拠
- canonical core: ticker/trades/orderbook/candles
- canonical extended: symbols/market meta/system status
- vendor extension: canonical 名称を持たない public API（例: circuit-break detail）
- not_supported は inventory enum に保持（今回 entry なし）

### evidence conflict
- catalog の `visibility` 欠落（特に bitflyer/coincheck/bittrade）があり、id naming (`.public.` or `public.` prefix) ルールで補完。
- conflict を回避するため scope policy に public detection rule を明記。

### 不足分への追加実装
- history と evidence 確認で inventory 漏れ防止に不足していたため、以下を task 内で実装:
  - registry public reader helper 追加
  - testkit inventory 完全性/分類/docs drift gate 追加
  - preview example 追加
