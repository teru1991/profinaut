# AUDIT-DUP-EXCH-000 監査レポート

- Task: `AUDIT-DUP-EXCH-000`
- Scope: `audit-duplicate-exchange-impls-and-docs`
- Execution mode: SINGLE-RUN (audit-only)
- Lock note: `status.json` 上で PR #140 が `LOCK:services-marketdata` を保持しているため、`services/marketdata/**` は read-only スキャンのみ実施。

## Summary

- 重複候補: **5件**
- Severity内訳: **High=2 / Medium=2 / Low=1**
- 主要リスク:
  - GMO/GMOCoin の識別子と責務境界が Python 実装内で分散・重複し、FIX時に片側修正漏れが起きやすい。
  - GMO の operation 記述が docs (`catalog.json`, `rest-api.md`, `websocket.md`) と実装側で二重管理されている。
  - health/capabilities API の実装が重複ファイル化しており、将来の仕様分岐リスクがある。

## A) GitHub / Docs OS 実態チェック（重複PR/分岐）

- `docs/status/status.json` では PR #140(open) と PR #152(closed) が記録されている。
- `open_prs` 配列に `status: closed` の #152 が残っており、「open_prs 名称と実体」のズレがある。
- GitHub API への直接照会 (`curl .../pulls?state=all`) はこの実行環境で空レスポンスとなり、リモート実PRとの差分を機械照合できなかったため、Docs OS記録を一次証跡として扱った。

## B/C/D) Finding 一覧（ID付き）

### FIND-001: GMO/GMOCoin 識別子・境界の二重実装
- 対象: Adapter/Facade (取引所差異吸収)
- Evidence:
  - `main.py` で `_ALLOWED_EXCHANGES = {"gmo"}` と `exchange` 正規化。
  - `raw_ingest.py` で `venue != "gmo"` を別途固定判定。
  - `gmo_ws_connector.py` でも `venue_id: "gmo"` を固定埋め込み。
- Risk: **High**
- Recommendation: **Merge**（exchange identifier/normalization を単一 descriptor に統合）

### FIND-002: GMO ticker/trades/orderbook 系の transport 層が REST/WS で二重導線
- 対象: HTTP client / WS connector
- Evidence:
  - `main.py` 内 `MarketDataPoller` が REST (`_request_json`, `_fetch_gmo_ticker`, `_fetch_gmo_ohlcv`) を実装。
  - `gmo_ws_connector.py` が別コンポーネントとして WS subscribe/ingest を実装。
  - どちらも `ingest_raw_envelope` へ送るが、共通の operation descriptor は見当たらず実装側で重複。
- Risk: **Medium**
- Recommendation: **Move**（transport共通層を抽出し、op/channelマッピングをSSOT化）

### FIND-003: GMO docs catalog の operation 定義が複数ファイルに重複
- 対象: Docs catalog（`catalog.json` / `rest-api.md` / `websocket.md`）
- Evidence:
  - `crypto.public.rest.ticker.get`, `orderbooks.get`, `trades.get` が `catalog.json` と `rest-api.md` の双方に存在。
  - `crypto.public.ws.ticker.update`, `trades.update`, `orderbooks.update` が `catalog.json` と `websocket.md` の双方に存在。
- Risk: **Medium**
- Recommendation: **Keep**（現行構造は維持しつつ、生成元を明示して手編集を最小化）

### FIND-004: allowed_ops/requires_auth 相当の descriptor 導線が分散
- 対象: Config/descriptor
- Evidence:
  - 実装側では `_ALLOWED_EXCHANGES`, `venue==gmo` 判定などが個別実装。
  - docs 側では operation/auth が `catalog.json` で管理。
  - 実装とdocs間を接続する単一 `policy_id` / `allowed_ops` / `requires_auth` 導線は確認できない。
- Risk: **High**
- Recommendation: **Move**（docs catalog と実装バリデーションの接続層を新設）

### FIND-005: health/capabilities API の重複実装（サービス内）
- 対象: API layer
- Evidence:
  - `main.py` に `/healthz` `/capabilities` の実装。
  - `routes/health.py` に同名エンドポイント実装。
  - `main.py` では `raw_ingest_router` は include しているが `health router` は include しておらず、片系統が実質未使用化。
- Risk: **Low**
- Recommendation: **Deprecate**（未使用経路を廃止し単一実装へ統一）

## 次タスク（FIX）への入力

- 優先度順:
  1. FIND-001 / FIND-004（境界・descriptorのSSOT化）
  2. FIND-002（transport層統合）
  3. FIND-003（docs生成運用の固定）
  4. FIND-005（API重複整理）
- 方針分類:
  - additive: descriptor接続層
  - deprecate: 未使用 health router
  - move/merge: GMO adapter/transport の責務再配置

## 実行コマンド（監査）

- `rg -n "(gmo|gmocoin)" services/marketdata docs/exchanges`
- `rg -n "(bybit|binance|bitbank|bitflyer|coincheck)" services/marketdata docs/exchanges`
- `rg -n "subscribe_|fetch_|orderbook|ticker|trades" services/marketdata`
- `rg -n "ws_url|WebSocket|wss://" services/marketdata docs/exchanges`
- `rg -n "allowed_ops|requires_auth|capabilities|policy_id" services/marketdata`
- `rg -n "include_router|routes.health|/capabilities|healthz" services/marketdata/app/main.py services/marketdata/app/routes/health.py`

