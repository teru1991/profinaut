# Verification: UCEL-SSOT-CONSISTENCY-003

## 1) Changed files (`git diff --name-only`)
```bash
docs/specs/ucel/ssot_consistency_gate_v1.md
docs/status/trace-index.json
docs/verification/UCEL-SSOT-CONSISTENCY-003.md
ucel/Cargo.lock
ucel/crates/ucel-registry/src/hub/registry.rs
ucel/crates/ucel-testkit/Cargo.toml
ucel/crates/ucel-testkit/src/coverage_v2.rs
ucel/crates/ucel-testkit/src/lib.rs
ucel/crates/ucel-testkit/src/ssot_consistency.rs
ucel/crates/ucel-testkit/tests/ssot_catalog_coverage_links.rs
ucel/crates/ucel-testkit/tests/ssot_consistency_gate.rs
ucel/crates/ucel-testkit/tests/ssot_ws_rules_alignment.rs
ucel/crates/ucel-ws-rules/src/lib.rs
ucel/crates/ucel-ws-rules/src/validation.rs
ucel/docs/ssot/coverage_migration_policy.md
ucel/docs/ssot/ssot_consistency_policy.md
```

## 2) What / Why
- SSOT 役割分担（catalog / coverage / coverage_v2 / ws_rules）と fail-closed ルールを spec と運用ポリシーに固定した。
- testkit に `ssot_consistency` validator を追加し、registry canonical 名、coverage_v2 family、ws_rules entitlement、legacy coverage scope の整合を機械検証化した。
- ws_rules 側に validation index reader を追加し、rules の exchange_id/entitlement/support を test から読み取れるようにした。
- SSOT drift gate テスト（consistency / ws_rules alignment / catalog-coverage link）を追加し、未知 drift は fail、明示例外のみ warning 扱いにした。
- registry helper を最小追加し、registered exchange names/ids/catalog keys を consistency validator が一元参照できるようにした。

## 3) Self-check results
- Allowed-path check OK
  - `git diff --name-only | awk '...allowlist...'` => 出力なし
- Tests added/updated OK
  - `ucel/crates/ucel-testkit/tests/ssot_consistency_gate.rs`
  - `ucel/crates/ucel-testkit/tests/ssot_ws_rules_alignment.rs`
  - `ucel/crates/ucel-testkit/tests/ssot_catalog_coverage_links.rs`
  - `ucel/crates/ucel-testkit/src/ssot_consistency.rs`
- Build/Unit test command results
  - `cd ucel && cargo test -p ucel-ws-rules` => PASS
  - `cd ucel && cargo test -p ucel-testkit --test ssot_consistency_gate -- --nocapture` => PASS
  - `cd ucel && cargo test -p ucel-testkit --test ssot_ws_rules_alignment -- --nocapture` => PASS
  - `cd ucel && cargo test -p ucel-testkit --test ssot_catalog_coverage_links -- --nocapture` => PASS
  - `cd ucel && cargo test -p ucel-ws-rules -p ucel-registry -p ucel-testkit` => FAIL（既存 fixture 不足: `support_bundle_manifest_fixture_is_sane` が `/workspace/profinaut/fixtures/support_bundle/manifest.json` を読めず失敗）
- trace-index json.tool OK
  - `python -m json.tool docs/status/trace-index.json > /dev/null`
- Secrets scan OK
  - `rg -n "AKIA|SECRET|PRIVATE KEY|BEGIN RSA" <changed files>` => ヒットなし
- docsリンク存在チェック OK
  - `rg -n "docs/" docs/specs/ucel/ssot_consistency_gate_v1.md ucel/docs/ssot/ssot_consistency_policy.md`

## 4) ★履歴確認の証拠（必須）
- 実行コマンド
  - `git log --oneline --decorate -n 50`
  - `git log --graph --oneline --decorate --all -n 80`
  - `git show <HEAD>`
  - `git show <ucel/coverage_v2 last sha>`
  - `git show <ucel/crates/ucel-ws-rules/src/lib.rs last sha>`
  - `git show <ucel/crates/ucel-registry/src/hub/registry.rs last sha>`
  - `git show <ucel/crates/ucel-testkit/src/coverage_v2.rs last sha>`
  - `git blame -w ucel/coverage/binance.yaml`
  - `git blame -w ucel/coverage_v2/binance-spot.yaml`
  - `git blame -w ucel/crates/ucel-ws-rules/src/lib.rs`
  - `git blame -w ucel/crates/ucel-registry/src/hub/registry.rs`
  - `git blame -w ucel/crates/ucel-testkit/src/coverage_v2.rs`
  - `git reflog -n 30`
  - `git merge-base HEAD origin/master`（remote 未設定のため不可）
  - `git branch -vv`
  - `git log --merges --oneline -n 30`
  - `git show <latest merge> --stat`
- 主要 SHA（確認対象）
  - HEAD(着手時): `8f7a9ab2`
  - coverage_v2 recent: `58868c79930bac39f7de92650126bb8bb1227129`
  - ws_rules/lib recent: `58868c79930bac39f7de92650126bb8bb1227129`
  - hub/registry recent: `8f7a9ab2f1b5bffa011c16f9bb61194348c8f250`
  - testkit/coverage_v2 recent: `39967099f760277c4cc8acf2f4df96054be6a4b5`
- 役割整理（coverage / coverage_v2 / ws_rules / registry）
  - registry canonical 名は Hub/Invoker 到達性の anchor。
  - coverage_v2 は family/surface support の主判定。
  - ws_rules は runtime entitlement/heartbeat/rate の主判定。
  - legacy coverage は bridge として scope drift を検知。
- 発見した不整合一覧
  - registry canonical と coverage_v2 venue が分離（family splitで未統合）
    - bitbank/bitflyer/coincheck/coinbase/deribit/sbivc/upbit など coverage_v2 未移行 venue
  - coverage_v2 supported WS family に ws_rules が未定義
    - `bitget-coin-futures`, `bitget-usdc-futures`
  - ws_rules 一部ファイルは strict parse 不能（duplicate key）
- 最小修正方針
  - validator/report を先に実装し、未知 drift は fail。
  - 上記既知ギャップは docs に explicit exception として固定し warning 化。
  - 新規 drift は exception に追加しない限り fail する。

## 5) Exception notes
- 現時点の explicit exceptions は `ucel/docs/ssot/ssot_consistency_policy.md` に列挙。
- 将来タスクで coverage_v2 未移行 venue と bitget futures rules を解消したら allowlist から削除する。
