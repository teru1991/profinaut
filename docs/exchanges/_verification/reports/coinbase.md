# coinbase Verification Report

## 1) Official Sources（公式導線の説明・最終URL・確認日）
- Confirmed date: 2026-02-21
- 公式導線（一次情報）: `https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview` を起点に、同一公式ドキュメント配下の REST/WS/Auth/Rate/Error/Changelog を参照。
- REST: https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/introduction
- WebSocket: https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/guides/websocket
- Auth: https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview
- Rate Limit: https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview
- Error model: https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview
- Changelog: https://docs.cdp.coinbase.com/coinbase-app/introduction/changelog

## 2) Inventory（docs/exchanges内の対象ファイル一覧）
- Directory: `docs/exchanges/coinbase`
- File-to-source mapping:
| File | Primary official URL |
|---|---|
| `CHANGELOG.md` | https://docs.cdp.coinbase.com/coinbase-app/introduction/changelog |
| `README.md` | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview |
| `catalog.json` | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview |
| `data.md` | https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/introduction |
| `diffs.md` | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview |
| `fix.md` | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview |
| `rest-api.md` | https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/introduction |
| `sources.md` | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview |
| `templates.md` | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview |
| `websocket.md` | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/guides/websocket |

## 3) Coverage（領域ごとYes/No/NA）
| Domain | Yes/No/NA | Explanation | Evidence URL |
|---|---|---|---|
| Version/Lifecycle | No | docs/exchanges 側で現行版/非推奨の明確区別が不足。 | https://docs.cdp.coinbase.com/coinbase-app/introduction/changelog |
| Authentication (P0) | No | 署名ベース文字列（method/path/query/body）と timestamp 単位の公式突合が未完了。 | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview |
| Rate Limit | No | IP/Account/Key 単位と Retry-After の記載整合を要確認。 | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview |
| Market Data (P0) | Yes | Market Data 記載ファイルは存在するが、返却フィールド/sequence の正誤検証を継続。 | https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/introduction |
| Trading (P0) | Yes | Trading 系エンドポイントの棚卸しは可能、必須/任意/丸め規則の突合が残る。 | https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/introduction |
| Account (P0) | Yes | balances/positions の項目は参照可能、定義整合は追加検証要。 | https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/introduction |
| Transfer | NA | 取引所によって提供有無が異なるため、該当時のみ EXD-C で実証予定。 | https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/introduction |
| WebSocket (P0) | Yes | WS接続/購読系の資料はあるが、認証ハンドシェイク/再接続の精査が必要。 | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/guides/websocket |
| Error model | No | HTTP と独自エラーコードの対応・再試行可否分類が未確定。 | https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview |

## 4) Auth（署名ベース文字列の完全記述、サンプル一致）
- Current state: **No**（署名方式の完全突合未確定）
- Pending verification points:
  - signature payload に `method + path + query + body` をどう連結するか
  - query ソート規則、body canonicalization、改行/区切り
  - timestamp 単位（秒/ms）と許容 skew
  - エンコード（hex/base64）
- Evidence URL: https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview
- Fix target files:
  - `docs/exchanges/coinbase/rest-api.md`
  - `docs/exchanges/coinbase/websocket.md`（存在時）

## 5) P0 Endpoints（仕様の要点と差分）
- ticker/orderbook/trades/klines:
  - 差分種別: **不足**（返却フィールド定義・sequence処理・欠損時挙動の明示が不足）
  - 根拠: https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/introduction
  - 修正先: `docs/exchanges/coinbase/data.md`, `docs/exchanges/coinbase/rest-api.md`
- order create/cancel/status:
  - 差分種別: **不足**（必須/任意、timeInForce/postOnly/reduceOnly、部分約定時挙動）
  - 根拠: https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/introduction
  - 修正先: `docs/exchanges/coinbase/rest-api.md`
- balances/positions:
  - 差分種別: **不足**（available/locked 定義、デリバの証拠金/平均建値）
  - 根拠: https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/introduction
  - 修正先: `docs/exchanges/coinbase/rest-api.md`
- WS public/private:
  - 差分種別: **不足**（subscribe payload・ping/pong・reconnect/resubscribe）
  - 根拠: https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/guides/websocket
  - 修正先: `docs/exchanges/coinbase/websocket.md`

## 6) Rate limit / Error model
- Rate limit: **No**（weight制/頻度制/IP-or-Account単位の明示が不足）
  - Evidence: https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview
- Error model: **No**（HTTP status と取引所独自コードの対応、retryable判定が不足）
  - Evidence: https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview

## 7) Findings（P0/P1/P2、根拠URL、修正先ファイル）
- P0-1: 認証署名仕様（ベース文字列・timestamp・header）を公式と1対1で固定化できていない。
  - Evidence: https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview
  - Fix: `docs/exchanges/coinbase/rest-api.md`, `docs/exchanges/coinbase/websocket.md`
- P0-2: WS 維持運用（heartbeat/reconnect/sequence復元）の確定記述が不足。
  - Evidence: https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/guides/websocket
  - Fix: `docs/exchanges/coinbase/websocket.md`
- P1-1: Rate limit と retry/backoff 指針の統一表現が不足。
  - Evidence: https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview
  - Fix: `docs/exchanges/coinbase/rest-api.md`, `docs/exchanges/coinbase/README.md`
- P1-2: 注文・数量・価格の精度/丸め規則の明確化が不足。
  - Evidence: https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/introduction
  - Fix: `docs/exchanges/coinbase/rest-api.md`, `docs/exchanges/coinbase/data.md`
- P2-1: 版管理（現行/旧版/非推奨）と changelog 関連リンクの導線改善余地。
  - Evidence: https://docs.cdp.coinbase.com/coinbase-app/introduction/changelog
  - Fix: `docs/exchanges/coinbase/README.md`, `docs/exchanges/coinbase/sources.md`

## 8) Smoke Test（実施した/できない理由、ログへのリンクパス）
- EXD-C で実施予定（Public REST/WS 優先）。

## 9) Fix Plan（修正方針）
- EXD-C で P0（Auth/WS/Trading/Account）から順に docs 本体を修正し、証拠ログを保存。

## 10) Status（A完了/B完了/C完了、確定/例外）
- A完了 / B完了 / C未着手
