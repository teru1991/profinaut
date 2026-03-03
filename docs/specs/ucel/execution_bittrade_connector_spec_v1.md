# UCEL Execution Bittrade Connector Spec v1

- Document ID: UCEL-I-EXEC-BITTRADE-V1
- Status: Canonical / Fixed Contract
- Task: UCEL-I-EXEC-002
- Depends-on: `execution_public_surface_spec_v1.md`, `ucel-cex-bittrade/src/lib.rs`

## Purpose

Bittrade を Execution の "実接続の最初の証明" として採用し、
`place / cancel / open_orders / reconcile` の 4 操作が一貫して動くことをテストで証明する。

---

## Endpoints（SSOT）

REST_ENDPOINTS 定義（`ucel-cex-bittrade/src/lib.rs`）を SSOT とし、
`BittradeExecutionConnector` は以下の endpoint id を使う:

| 操作 | endpoint id | path |
|---|---|---|
| accounts 解決 | `private.rest.account.accounts.get` | `/v1/account/accounts` |
| place | `private.rest.order.place.post` | `/v1/order/orders/place` |
| cancel | `private.rest.order.cancel.post` | `/v1/order/orders/{order-id}/submitcancel` |
| open list | `private.rest.order.list.get` | `/v1/order/orders` |
| order get | `private.rest.order.get` | `/v1/order/orders/{order-id}`（将来 reconcile 強化用） |

---

## Account-ID Resolution

- 初回 `place_order` 時に `accounts.get` を叩き、`data[0].id` を account-id として取得する。
- 取得した account-id は `Mutex<Option<String>>` にキャッシュし、以後再利用する。
- `place` body の `"account-id"` フィールドとして注入する。

---

## Idempotency Mapping

- `OrderRequest.idempotency.0` を body の `"client-order-id"` に転写する（Live 時）。
- SDK 側（`ExecutionClientAsync::place`）で `tags["client_order_id"]` が未指定なら注入される（事故防止）。
- connector は `req.intent.tags["client_order_id"]` を `"client-order-id"` として body に含める。

---

## Order Type Mapping

| OrderSide × OrderType | Bittrade type 文字列 |
|---|---|
| Buy × Limit / PostOnly | `buy-limit` |
| Buy × Market | `buy-market` |
| Sell × Limit / PostOnly | `sell-limit` |
| Sell × Market | `sell-market` |

---

## Response Parsing

### place

```json
{ "status": "ok", "data": "1234567890" }
```

`data` フィールドを `venue_order_id` として返す（数値 or 文字列）。

### cancel

```json
{ "status": "ok", "data": "1234567890" }
```

`status == "ok"` のとき `true` を返す。

### open_orders

```json
{ "status": "ok", "data": [{"id": 555, "symbol": "btcjpy", "type": "buy-limit"}, ...] }
```

`data` 配列から `id`（`venue_order_id`）と `symbol`（大文字化）を抽出する。
`intent_id` / `idempotency` は v1 では `"unknown"` / `random_uuid`（venue 側情報を優先する設計）。

### reconcile（v1 最小実装）

v1 では `ok: true, mismatches: []` を返す。
詳細照合は FileAuditSink と組み合わせて次版で強化する。

---

## Proof

### Contract tests（MockTransport）

`ucel-cex-bittrade/tests/execution_connector_contract.rs`:

| テスト | 検証内容 |
|---|---|
| `bittrade_connector_places_with_account_id_and_client_order_id` | accounts.get → place の順、body に account-id と client-order-id が入る |
| `bittrade_connector_caches_account_id` | 2 回 place しても accounts.get は 1 回のみ |
| `bittrade_connector_cancel_sends_correct_path` | cancel が order-id を path に埋める |
| `bittrade_connector_list_open_orders_sends_states_param` | open_orders が states クエリを送る |
| `bittrade_connector_reconcile_returns_ok` | reconcile が ok=true を返す |
| `execution_client_async_injects_client_order_id_for_live` | SDK が Live で client_order_id を自動注入し body に反映 |

### E2E tests（ucel-sdk 側）

`ucel-sdk/tests/execution_e2e_bittrade_mock.rs`（インライン MockBittradeConnector 使用）:

| テスト | 検証内容 |
|---|---|
| `execution_async_e2e_with_file_audit` | place + cancel + FileAuditSink replay（4 イベント以上） |
| `execution_async_replay_without_audit_returns_error` | audit 未設定で replay がエラー |
| `execution_async_paper_does_not_call_connector` | Paper は venue_order_id=None |
| `execution_async_gate_rejects_limit_without_price` | Limit で price なしを gate が拒否 |
| `file_audit_sink_replay_filters_by_run_id` | run_id フィルタが正しく機能 |
| `file_audit_sink_replay_empty_when_file_missing` | ファイル未存在時は空を返す |
| `execution_async_reconcile_audited` | reconcile が audit に ReconcileResult を書く |

---

## Secrets Policy

`AuditEvent` は `intent.tags` を含むため、`tags` に secrets（API Key, Token 等）を入れてはならない。
これは運用ルールとして強制し、コード側での検査は行わない（v1）。
