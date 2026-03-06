# UCEL Roadmap (TASK-ID: UCEL-ROADMAP-001)

- Status: fixed task boundary / execution-order locked
- Branch: `planning/ucel-roadmap-001`
- Goal: UCELライブラリの不足分を、確実・安全・堅牢・高速に実装するための最小独立タスクへ分割し、順序と項目を固定する。

## Scope and constraints (locked)
- Allowed path policy: `docs/**`, `crates/**`, `tests/**`, `examples/**`, `scripts/**`, `.github/workflows/**`, `Cargo.toml`, `Cargo.lock`
- Existing file deletion: prohibited
- SSOT-first: docs更新を先行し、その後コードを変更
- Breaking changes: gate/test mandatory
- Verification: mandatory for every task
- History check: pre/post commit and merge history must be confirmed
- Conflict minimization: enforce minimal diffs

## Why exactly 13 tasks
「ポリシー/SSOT」「Hub/Registry」「認証」「Private REST」「Private WS」「Execution」「Symbol/Meta」「Public Adapter」「Ingest Runtime」「On-chain」「Stocks」「Diagnostics」は契約面・失敗モード・テスト戦略が異なるため、これ以上まとめると1タスクあたりの変更境界が大きくなり、履歴確認・回帰切り分け・競合回避が難しくなる。

## Evidence snapshot (repo-grounded)
- `ucel/crates/ucel-registry/src/hub/mod.rs`: `ExchangeId` が 9 venue のみ
- `ucel/crates/ucel-registry/src/hub/registry.rs`: catalog include が 9 venue のみ
- `ucel/crates/ucel-registry/src/support_bundle.rs`: `coverage_hash = "unknown"`
- `ucel/crates/ucel-chain-ethereum/src/lib.rs`: `ErrorCode::NotSupported`
- `ucel/crates/ucel-market-meta-catalog/src/lib.rs`: `map_exchange()` が一部 venue のみ
- `ucel/crates/ucel-cex-bittrade/src/execution.rs`: concrete execution connector の実質唯一例
- `ucel/crates/ucel-cex-bithumb/`: private/ws/execution/symbol 実装未整備
- `ucel/crates/ucel-cex-*/src/ws_manager.rs`: `Ok(())` スタブが残存
- `ucel/coverage_v2/`: family split 済みだが未整備 venue/family が残る
- `ucel/crates/ucel-ws-rules/rules/*.toml` と `coverage*.yaml`: entitlement/scope の整合未固定

## Locked task breakdown (minimum independent set)

1. `UCEL-POLICY-001`: Residency/Capability SSOT を導入し private/public gate を fail-fast 化
2. `UCEL-HUB-REGISTRY-002`: 全 CEX crate を Hub/Registry/Invoker 単一入口に統合
3. `UCEL-SSOT-CONSISTENCY-003`: catalog/coverage/ws_rules の strict consistency gate を確立
4. `UCEL-AUTH-CORE-004`: private 共通認証基盤（時刻・nonce・署名・redaction）を共通化
5. `UCEL-PRIVATE-REST-005`: 国内 private allowlist venue の REST private surface 完成
6. `UCEL-PRIVATE-WS-006`: 国内 private allowlist venue の private WS 認証/再接続/再購読完成
7. `UCEL-EXECUTION-007`: ExecutionConnectorAsync を allowlist venue へ拡張し統一
8. `UCEL-SYMBOL-META-008`: Symbol/MarketMeta の全 venue/family 完成
9. `UCEL-PUBLIC-ADAPTERS-009`: public-only/未完成 venue の public REST/WS adapter 完成
10. `UCEL-WS-INGEST-010`: ws ingest runtime（planner/store/journal/supervisor）を本番導線化
11. `UCEL-CHAIN-EVM-011`: `ucel-chain-ethereum` を実用 EVM adapter へ昇格
12. `UCEL-EQUITY-DATA-012`: 株API向け共通 adapter 層を新設
13. `UCEL-DIAGNOSTICS-013`: support bundle/analyzer/compatibility gate を証拠品質に完成

## Recommended execution order (fixed)
- Phase 1（土台固定）: `1 -> 2 -> 3 -> 4`
- Phase 2（国内private完成）: `5 -> 6 -> 7`
- Phase 3（public + meta + ingest）: `8 -> 9 -> 10`
- Phase 4（資産クラス拡張）: `11 -> 12`
- Phase 5（証明/保守性）: `13`

## Completion criteria (UCEL overall)
- 全 CEX crate が Hub/Registry/Invoker から到達可能
- 日本居住者ポリシーに沿って public/private 可否が自動判定
- 国内 private allowlist venue で REST/WS/Execution が統一 surface で利用可能
- public-only venue が market-data adapter として完成
- Symbol/MarketMeta が全 venue/family で統一
- WS ingest が durable queue + journal-first + reconnect resume を満たす
- EVM on-chain adapter が最小実用面を提供
- 株API vendor abstraction が追加
- support bundle/analyzer/compatibility gate により SSOT と実行状態を bundle から証明可能

## Next action
- 個別着手は `UCEL-POLICY-001` から 1 本ずつ詳細化する。
