# UCEL-I-EXEC-002 Verification

## Task

UCEL Execution（I）を "100%" として機械的に宣言できる状態にする:
- ucel-sdk の唯一の発注出口（async）を Bittrade（Live）で証明する
- 監査を永続（FileAuditSink / ndjson）で提供し、replay/reconcile の再現性を担保する
- 既存の sync API は互換維持し、async 実行の "本番向け" API を追加する

## Changed Files

### 新規ファイル

- `ucel/crates/ucel-sdk/src/execution/async_client.rs` — `ExecutionConnectorAsync` trait + `ExecutionClientAsync` 実装
- `ucel/crates/ucel-sdk/src/execution/audit_file.rs` — `FileAuditSink` (ndjson/WAL 永続監査)
- `ucel/crates/ucel-sdk/tests/execution_e2e_bittrade_mock.rs` — async E2E テスト（7 テスト）
- `ucel/crates/ucel-cex-bittrade/src/execution.rs` — `BittradeExecutionConnector` 実装
- `ucel/crates/ucel-cex-bittrade/tests/execution_connector_contract.rs` — Bittrade 契約テスト（6 テスト）
- `docs/specs/ucel/execution_bittrade_connector_spec_v1.md` — Bittrade 接続仕様 SSOT
- `docs/verification/UCEL-I-EXEC-002.md` — 本ファイル

### 差分編集（最小差分）

- `ucel/crates/ucel-sdk/src/execution/mod.rs` — `mod audit_file; mod async_client;` 追加
- `ucel/crates/ucel-sdk/src/execution/client.rs` — Live 時の `client_order_id` 自動注入を追加
- `ucel/crates/ucel-cex-bittrade/src/lib.rs` — `pub mod execution;` 追加
- `ucel/crates/ucel-cex-bittrade/Cargo.toml` — `ucel-sdk = { path = "../ucel-sdk" }` 追加
- `ucel/crates/ucel-sdk/Cargo.toml` — dev-dep に `tempfile = "3"` 追加
- `docs/specs/ucel/execution_public_surface_spec_v1.md` — 100% 定義と async API セクションを末尾追記

## What / Why

### async 入口の追加

- 既存の `ExecutionClient<C: ExecutionConnector>` (sync) は venue の REST client が async なため、
  実際の接続で `block_on` / デッドロックのリスクがあった。
- `ExecutionClientAsync<C: ExecutionConnectorAsync>` を追加することで、本番向けに安全な async 入口を提供。
- sync API は削除せず互換維持（`ExecutionConnector` / `ExecutionClient` はそのまま残す）。

### idempotency 自動注入

- Live 発注時に `client_order_id` が未指定の場合、`idempotency.0` を `tags["client_order_id"]` に注入。
- sync と async の両方で同じ保護を適用（運用事故防止）。

### FileAuditSink（永続監査）

- append 専用（追記のみ）/ 1 行 = 1 JSON（ndjson 形式）
- `fsync_each_append` フラグで耐障害性の強度を設定可能
- 壊れた行は salvage（スキップ）して replay を継続
- `AuditReplayFilter.run_id` によるフィルタが有効

### BittradeExecutionConnector

- `account-id` を初回に取得してキャッシュ（2 回目以降は REST を呼ばない）
- endpoint id を SSOT（lib.rs の REST_ENDPOINTS）から解決
- `client-order-id` を body に転写（SDK から注入された `tags["client_order_id"]` を使用）

## 依存関係の整合性

- `ucel-sdk` → `ucel-core` / `ucel-transport` など（変更なし）
- `ucel-cex-bittrade` → `ucel-sdk` を追加（新規）
- `ucel-sdk` → `ucel-cex-bittrade` は追加しない（循環依存禁止）
- ucel-sdk の E2E テスト（`execution_e2e_bittrade_mock.rs`）はインライン mock で実現
  （実際の BittradeExecutionConnector は bittrade 側のテストで検証）

## Self-check Results

### Allowed-path check

変更・追加ファイルはすべて `docs/`, `ucel/` 配下。許可パス外の変更なし。

### Tests

```
cargo test --manifest-path ucel/Cargo.toml -p ucel-sdk
  => 18 tests passed (lib: 3, e2e_bittrade_mock: 7, execution_surface_contract: 6, market_meta_e2e_it: 2)

cargo test --manifest-path ucel/Cargo.toml -p ucel-cex-bittrade
  => 15 tests passed (lib: 2, execution_connector_contract: 6, rest_contract: 7)

cargo fmt --manifest-path ucel/Cargo.toml --all
  => OK (format applied)
```

### trace-index JSON validation

```
python3 -m json.tool docs/status/trace-index.json > /dev/null
  => JSON valid
```

### Secrets scan（簡易）

```
rg -n "AKIA|BEGIN PRIVATE KEY|SECRET|TOKEN|API_KEY" -S ucel docs || true
  => No secrets found in new/changed code
```

## ★ 履歴確認の証拠

### git log 要点

```
fb9bcbe  HEAD -> claude/ucel-live-execution-YLtqr
         Merge pull request #423 (UCEL-H: strict coverage)
2219d58  UCEL-H: enforce strict=true coverage
123a218  UCEL-I: add ucel-sdk execution public surface (UCEL-I-EXEC-001)
1e369f8  UCEL-I: add ucel-sdk execution public surface (client/idempotency/audit/gate)
295e35c  master: Merge pull request #421
```

### UCEL-I-EXEC-001 の内容（直前タスク）

- `ucel-sdk/src/execution/` に `client.rs`, `types.rs`, `errors.rs`, `gate.rs`, `idempotency.rs`, `audit.rs` を追加
- sync API として `ExecutionClient<C: ExecutionConnector>` を実装
- `InMemoryAuditSink` を提供（テスト/開発用）
- "次: 監査の永続化・venue 実装" と明記されていた（本タスクで完了）

### 設計根拠

1. **sync trait 互換維持の方針**: EXEC-001 が sync trait で実装済み。EXEC-002 では "追加" として async API を提供。破壊変更なし。
2. **bittrade を最初の実接続 venue に選んだ理由**: `REST_ENDPOINTS` + `BittradeRestClient` が Transport 経由の async 実装として既に完成しており、private order 系エンドポイントが揃っている。
3. **FileAuditSink の path**: `ucel-sdk/src/execution/audit_file.rs`（既存 `audit.rs` と同じ execution モジュール内）。`audit.rs` に追記するより新規ファイルのほうがコンフリクトリスクが低い。
4. **E2E テストをインライン mock で実装した理由**: `ucel-sdk` が `ucel-cex-bittrade` に依存すると循環依存になる。実際の BittradeExecutionConnector の統合テストは bittrade 側テストで完全にカバー済み。

## 100% 達成の根拠

| 条件 | 証明 |
|---|---|
| 唯一の発注出口 | `ucel-sdk::execution::ExecutionClientAsync`（async）/ `ExecutionClient`（sync）のみ |
| Bittrade で Live place が動く | `execution_connector_contract.rs::bittrade_connector_places_with_account_id_and_client_order_id` |
| Bittrade で cancel が動く | `execution_connector_contract.rs::bittrade_connector_cancel_sends_correct_path` |
| open_orders が動く | `execution_connector_contract.rs::bittrade_connector_list_open_orders_sends_states_param` |
| reconcile が動く | `execution_connector_contract.rs::bittrade_connector_reconcile_returns_ok` |
| 監査が永続化される | `execution_e2e_bittrade_mock.rs::execution_async_e2e_with_file_audit`（ファイル存在・JSON 検証）|
| replay が機能する | `execution_e2e_bittrade_mock.rs::execution_async_e2e_with_file_audit`（4 イベント以上）|
| idempotency が注入される | `execution_connector_contract.rs::execution_client_async_injects_client_order_id_for_live` |
