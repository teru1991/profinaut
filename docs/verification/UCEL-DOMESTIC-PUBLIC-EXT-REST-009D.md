# UCEL-DOMESTIC-PUBLIC-EXT-REST-009D Verification

## 1) Changed files (`git diff --name-only` + new files)
- docs/specs/ucel/domestic_public_ext_rest_surface_v1.md
- docs/specs/ucel/domestic_public_ext_rest_schema_policy_v1.md
- docs/status/trace-index.json
- docs/verification/UCEL-DOMESTIC-PUBLIC-EXT-REST-009D.md
- ucel/docs/exchanges/domestic_public_rest_extension_matrix.md
- ucel/docs/exchanges/domestic_public_rest_extension_schema_matrix.md
- ucel/docs/exchanges/domestic_public_rest_extension_usage.md
- ucel/crates/ucel-core/src/public_rest_ext.rs
- ucel/crates/ucel-core/src/lib.rs
- ucel/crates/ucel-sdk/src/public_rest_ext.rs
- ucel/crates/ucel-sdk/src/lib.rs
- ucel/crates/ucel-registry/src/hub/rest.rs
- ucel/crates/ucel-testkit/src/domestic_public_rest_ext.rs
- ucel/crates/ucel-testkit/src/lib.rs
- ucel/crates/ucel-testkit/tests/domestic_public_ext_rest_contract_matrix.rs
- ucel/crates/ucel-testkit/tests/domestic_public_ext_rest_schemas.rs
- ucel/crates/ucel-testkit/tests/domestic_public_ext_rest_docs_drift.rs
- ucel/crates/ucel-testkit/tests/domestic_public_ext_rest_compat.rs
- ucel/examples/domestic_public_ext_rest_preview.rs
- ucel/fixtures/domestic_public_ext_rest/cases.json

## 2) What / Why
- 009A inventory に基づく `vendor_public_extension` + `api_kind=rest` の全 16 endpoint を typed extension surface へ固定した。
- `ucel-core` に extension category/schema/payload type/metadata/envelope と operation spec を追加し、unknown operation を fail-fast 化した。
- `ucel-registry` に inventory-backed extension route を追加し、`call_vendor_public_typed` で typed envelope を返す経路を追加した。
- `ucel-sdk` に `DomesticPublicRestExtensionFacade` を追加し、status/reference/generic の typed call を公開した。
- docs matrix/schema matrix/usage と fixtures + test gates を追加して drift/compat/schema 不整合を CI で fail させるようにした。

## 3) Self-check results
- Allowed-path check OK（差分中の allowlist 外は既存汚染 `services/marketdata-rs/Cargo.lock` のみ）。
- Tests added/updated OK:
  - `domestic_public_ext_rest_contract_matrix`
  - `domestic_public_ext_rest_schemas`
  - `domestic_public_ext_rest_docs_drift`
  - `domestic_public_ext_rest_compat`
- Build/Unit test command results:
  - `cd ucel && cargo test -p ucel-core -p ucel-registry -p ucel-sdk -p ucel-testkit` ✅
  - `cd ucel && cargo test -p ucel-testkit --test domestic_public_ext_rest_contract_matrix -- --nocapture` ✅
  - `cd ucel && cargo test -p ucel-testkit --test domestic_public_ext_rest_schemas -- --nocapture` ✅
  - `cd ucel && cargo test -p ucel-testkit --test domestic_public_ext_rest_docs_drift -- --nocapture` ✅
  - `cd ucel && cargo test -p ucel-testkit --test domestic_public_ext_rest_compat -- --nocapture` ✅
  - `cd ucel && cargo test -p ucel-cex-bitbank` ✅
  - `cd ucel && cargo test -p ucel-cex-bitflyer` ✅
  - `cd ucel && cargo test -p ucel-cex-coincheck` ✅
  - `cd ucel && cargo test -p ucel-cex-gmocoin` ✅
  - `cd ucel && cargo test -p ucel-cex-bittrade` ✅
  - `cd ucel && cargo test -p ucel-cex-sbivc` ✅
- trace-index json.tool OK:
  - `python -m json.tool docs/status/trace-index.json > /dev/null` ✅
- Secrets scan:
  - `rg -n 'AKIA|SECRET|PRIVATE KEY|BEGIN RSA|token' ...` 該当なし ✅
- docs リンク存在チェック（今回 docs 内の `docs/` 参照のみ）:
  - 該当リンク参照なし（外部 docs path 参照を新規追加していない）✅

## 4) 履歴確認の証拠（必須）
- 実行コマンド:
  - `git log --oneline --decorate -n 50`
  - `git log --graph --oneline --decorate --all -n 80`
  - `git show HEAD --stat`
  - `git show 6cbe745 --stat`
  - `git blame -w ucel/coverage_v2/domestic_public/jp_public_inventory.json`
  - `git blame -w ucel/crates/ucel-core/src/public_rest.rs`（ファイル未存在）
  - `git blame -w ucel/crates/ucel-sdk/src/public_rest.rs`（ファイル未存在）
  - `git blame -w ucel/crates/ucel-registry/src/hub/rest.rs`
  - `git blame -w ucel/crates/ucel-registry/src/hub/registry.rs`
  - `git blame -w ucel/crates/ucel-cex-bitbank` ほか domestic venue crate paths（directory 指定は git blame 不可）
  - `git reflog -n 30`
  - `git merge-base HEAD origin/master`（remote 未設定で失敗）
  - `git branch -vv`
  - `git log --merges --oneline -n 30`
  - `git diff --name-only origin/master...HEAD`（remote 未設定で失敗）
  - `git log --oneline origin/master -- <hotspots>`（remote 未設定で失敗）
- 判定結果:
  - 直近 6cbe745（009A inventory 固定）を起点に、009D は extension typed surface 追加に局所化。
  - raw passthrough 的な extension surface は未整備だったため、core+registry+sdk+testkit で新規整備が必要。
  - hotspot は `ucel-core/lib`, `ucel-sdk/lib`, `registry/hub/rest`。本タスクでは最小差分の追加のみ実施。
  - `origin/master` 未設定のため upstream 差分比較は環境制約。ローカル履歴範囲で整合性確認。

### 国内取引所 public REST extension 実装現状棚卸し
- inventory (`jp_public_inventory.json`) 上の `vendor_public_extension + rest` は 16 件。
- 内訳: bitbank=2, bitflyer=9, coincheck=2, bittrade=3, gmocoin=0, sbivc=0。
- 本タスクで 16 件すべてを `VendorPublicRestOperationSpec` と docs schema matrix に固定。

### inventory から各 venue 実装へ落とし込んだ根拠
- routing source は inventory JSON。
- registry `list_vendor_public_rest_extension_operation_ids` と contract test で inventory との一致を gate。
- sdk facade は registry typed route のみを使用。

### category / schema_version / payload_type / metadata 設計根拠
- category/payload_type は spec policy に従い enum 固定。
- schema version は全 operation を 1.0.0 として固定し、compare/compat test を追加。
- metadata は venue/operation_id/source_endpoint/inventory_public_id を必須化し、builder で検証。

### 既存 parser/request builder の再利用・抽出の根拠
- 既存 venue crate を変更せず、hub rest response を extension typed builder で変換する方式にして競合を回避。
- endpoint call 自体は既存 catalog route を再利用。

### 不足があったため追加実装した対策
- 009A 時点では extension typed schema と compat gate が未実装だったため、以下を追加:
  - `public_rest_ext.rs` (core/sdk)
  - inventory-backed registry extension routing
  - fixtures + schema/docs/compat/contract gates
