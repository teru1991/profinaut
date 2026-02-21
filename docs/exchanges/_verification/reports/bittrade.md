# bittrade Verification Report

## 1) Official Sources（公式導線の説明・最終URL・確認日）
- Confirmed date: 2026-02-21
- 公式導線（一次情報）: `https://www.bittrade.co.jp/` を起点に、同一公式ドキュメント配下の REST/WS/Auth/Rate/Error/Changelog を参照。
- REST: https://api-doc.bittrade.co.jp/#rest-api
- WebSocket: https://api-doc.bittrade.co.jp/#websocket-public
- Auth: https://www.bittrade.co.jp/
- Rate Limit: https://www.bittrade.co.jp/
- Error model: https://www.bittrade.co.jp/
- Changelog: https://www.bittrade.co.jp/

## 2) Inventory（docs/exchanges内の対象ファイル一覧）
- Directory: `docs/exchanges/bittrade`
- File-to-source mapping:
| File | Primary official URL |
|---|---|
| `CHANGELOG.md` | https://www.bittrade.co.jp/ |
| `README.md` | https://www.bittrade.co.jp/ |
| `catalog.json` | https://www.bittrade.co.jp/ |
| `data.md` | https://api-doc.bittrade.co.jp/#rest-api |
| `rest-api.md` | https://api-doc.bittrade.co.jp/#rest-api |
| `sources.md` | https://www.bittrade.co.jp/ |
| `templates.md` | https://www.bittrade.co.jp/ |
| `websocket.md` | https://api-doc.bittrade.co.jp/#websocket-public |

## 3) Coverage（領域ごとYes/No/NA）
| Domain | Yes/No/NA | Explanation | Evidence URL |
|---|---|---|---|
| Version/Lifecycle | No | docs/exchanges 側で現行版/非推奨の明確区別が不足。 | https://www.bittrade.co.jp/ |
| Authentication (P0) | No | 署名ベース文字列（method/path/query/body）と timestamp 単位の公式突合が未完了。 | https://www.bittrade.co.jp/ |
| Rate Limit | No | IP/Account/Key 単位と Retry-After の記載整合を要確認。 | https://www.bittrade.co.jp/ |
| Market Data (P0) | Yes | Market Data 記載ファイルは存在するが、返却フィールド/sequence の正誤検証を継続。 | https://api-doc.bittrade.co.jp/#rest-api |
| Trading (P0) | Yes | Trading 系エンドポイントの棚卸しは可能、必須/任意/丸め規則の突合が残る。 | https://api-doc.bittrade.co.jp/#rest-api |
| Account (P0) | Yes | balances/positions の項目は参照可能、定義整合は追加検証要。 | https://api-doc.bittrade.co.jp/#rest-api |
| Transfer | NA | 取引所によって提供有無が異なるため、該当時のみ EXD-C で実証予定。 | https://api-doc.bittrade.co.jp/#rest-api |
| WebSocket (P0) | Yes | WS接続/購読系の資料はあるが、認証ハンドシェイク/再接続の精査が必要。 | https://api-doc.bittrade.co.jp/#websocket-public |
| Error model | No | HTTP と独自エラーコードの対応・再試行可否分類が未確定。 | https://www.bittrade.co.jp/ |

## 4) Auth（署名ベース文字列の完全記述、サンプル一致）
- Current state: **No**（署名方式の完全突合未確定）
- Pending verification points:
  - signature payload に `method + path + query + body` をどう連結するか
  - query ソート規則、body canonicalization、改行/区切り
  - timestamp 単位（秒/ms）と許容 skew
  - エンコード（hex/base64）
- Evidence URL: https://www.bittrade.co.jp/
- Fix target files:
  - `docs/exchanges/bittrade/rest-api.md`
  - `docs/exchanges/bittrade/websocket.md`（存在時）

## 5) P0 Endpoints（仕様の要点と差分）
- ticker/orderbook/trades/klines:
  - 差分種別: **不足**（返却フィールド定義・sequence処理・欠損時挙動の明示が不足）
  - 根拠: https://api-doc.bittrade.co.jp/#rest-api
  - 修正先: `docs/exchanges/bittrade/data.md`, `docs/exchanges/bittrade/rest-api.md`
- order create/cancel/status:
  - 差分種別: **不足**（必須/任意、timeInForce/postOnly/reduceOnly、部分約定時挙動）
  - 根拠: https://api-doc.bittrade.co.jp/#rest-api
  - 修正先: `docs/exchanges/bittrade/rest-api.md`
- balances/positions:
  - 差分種別: **不足**（available/locked 定義、デリバの証拠金/平均建値）
  - 根拠: https://api-doc.bittrade.co.jp/#rest-api
  - 修正先: `docs/exchanges/bittrade/rest-api.md`
- WS public/private:
  - 差分種別: **不足**（subscribe payload・ping/pong・reconnect/resubscribe）
  - 根拠: https://api-doc.bittrade.co.jp/#websocket-public
  - 修正先: `docs/exchanges/bittrade/websocket.md`

## 6) Rate limit / Error model
- Rate limit: **No**（weight制/頻度制/IP-or-Account単位の明示が不足）
  - Evidence: https://www.bittrade.co.jp/
- Error model: **No**（HTTP status と取引所独自コードの対応、retryable判定が不足）
  - Evidence: https://www.bittrade.co.jp/

## 7) Findings（P0/P1/P2、根拠URL、修正先ファイル）
- P0-1: 認証署名仕様（ベース文字列・timestamp・header）を公式と1対1で固定化できていない。
  - Evidence: https://www.bittrade.co.jp/
  - Fix: `docs/exchanges/bittrade/rest-api.md`, `docs/exchanges/bittrade/websocket.md`
- P0-2: WS 維持運用（heartbeat/reconnect/sequence復元）の確定記述が不足。
  - Evidence: https://api-doc.bittrade.co.jp/#websocket-public
  - Fix: `docs/exchanges/bittrade/websocket.md`
- P1-1: Rate limit と retry/backoff 指針の統一表現が不足。
  - Evidence: https://www.bittrade.co.jp/
  - Fix: `docs/exchanges/bittrade/rest-api.md`, `docs/exchanges/bittrade/README.md`
- P1-2: 注文・数量・価格の精度/丸め規則の明確化が不足。
  - Evidence: https://api-doc.bittrade.co.jp/#rest-api
  - Fix: `docs/exchanges/bittrade/rest-api.md`, `docs/exchanges/bittrade/data.md`
- P2-1: 版管理（現行/旧版/非推奨）と changelog 関連リンクの導線改善余地。
  - Evidence: https://www.bittrade.co.jp/
  - Fix: `docs/exchanges/bittrade/README.md`, `docs/exchanges/bittrade/sources.md`

## 8) Smoke Test（実施した/できない理由、ログへのリンクパス）
- REST: 未実施（公開REST endpoint不明）
  - Log: `docs/exchanges/_verification/evidence/bittrade/smoke_rest.txt`
- WS: 未実施（sandboxで外部WS疎通不可）
  - Log: `docs/exchanges/_verification/evidence/bittrade/smoke_ws.txt`
- Private(read-only): 未実施（鍵未設定・秘匿方針）

## 9) Fix Plan（修正方針）
- C例外解除条件: 外部ネットワーク疎通可能な環境で Public REST/WS の再スモークを実行。
- docs本体のP0修正は issues_summary の修正先ファイルに沿って継続。

## 10) Status（A完了/B完了/C完了、確定/例外）
- A完了 / B完了 / C例外（公開endpoint不明によりスモーク未実施）
