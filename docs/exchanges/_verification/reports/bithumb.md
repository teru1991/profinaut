# bithumb Verification Report

## 1) Official Sources（公式導線の説明・最終URL・確認日）
- Confirmed date: 2026-02-21
- 公式導線（一次情報）: `https://apidocs.bithumb.com/` を起点に、同一公式ドキュメント配下の REST/WS/Auth/Rate/Error/Changelog を参照。
- REST: https://apidocs.bithumb.com/reference/%EA%B8%B0%EB%B3%B8-%EC%A0%95%EB%B3%B4
- WebSocket: https://apidocs.bithumb.com/reference/WEBSOCKET
- Auth: https://apidocs.bithumb.com/
- Rate Limit: https://apidocs.bithumb.com/changelog/%EC%97%85%EB%8D%B0%EC%9D%B4%ED%8A%B8-public-websocket-%EB%8D%B0%EC%9D%B4%ED%84%B0-%EC%9A%94%EC%B2%AD-%EC%88%98-%EC%A0%9C%ED%95%9Crate-limit-%EC%A0%81%EC%9A%A9-%EC%95%88%EB%82%B4
- Error model: https://apidocs.bithumb.com/
- Changelog: https://apidocs.bithumb.com/changelog

## 2) Inventory（docs/exchanges内の対象ファイル一覧）
- Directory: `docs/exchanges/bithumb`
- File-to-source mapping:
| File | Primary official URL |
|---|---|
| `CHANGELOG.md` | https://apidocs.bithumb.com/changelog |
| `README.md` | https://apidocs.bithumb.com/ |
| `catalog.json` | https://apidocs.bithumb.com/ |
| `data.md` | https://apidocs.bithumb.com/reference/%EA%B8%B0%EB%B3%B8-%EC%A0%95%EB%B3%B4 |
| `diffs.md` | https://apidocs.bithumb.com/ |
| `fix.md` | https://apidocs.bithumb.com/ |
| `rest-api.md` | https://apidocs.bithumb.com/reference/%EA%B8%B0%EB%B3%B8-%EC%A0%95%EB%B3%B4 |
| `sources.md` | https://apidocs.bithumb.com/ |
| `templates.md` | https://apidocs.bithumb.com/ |
| `websocket.md` | https://apidocs.bithumb.com/reference/WEBSOCKET |

## 3) Coverage（領域ごとYes/No/NA）
| Domain | Yes/No/NA | Explanation | Evidence URL |
|---|---|---|---|
| Version/Lifecycle | No | docs/exchanges 側で現行版/非推奨の明確区別が不足。 | https://apidocs.bithumb.com/changelog |
| Authentication (P0) | No | 署名ベース文字列（method/path/query/body）と timestamp 単位の公式突合が未完了。 | https://apidocs.bithumb.com/ |
| Rate Limit | No | IP/Account/Key 単位と Retry-After の記載整合を要確認。 | https://apidocs.bithumb.com/changelog/%EC%97%85%EB%8D%B0%EC%9D%B4%ED%8A%B8-public-websocket-%EB%8D%B0%EC%9D%B4%ED%84%B0-%EC%9A%94%EC%B2%AD-%EC%88%98-%EC%A0%9C%ED%95%9Crate-limit-%EC%A0%81%EC%9A%A9-%EC%95%88%EB%82%B4 |
| Market Data (P0) | Yes | Market Data 記載ファイルは存在するが、返却フィールド/sequence の正誤検証を継続。 | https://apidocs.bithumb.com/reference/%EA%B8%B0%EB%B3%B8-%EC%A0%95%EB%B3%B4 |
| Trading (P0) | Yes | Trading 系エンドポイントの棚卸しは可能、必須/任意/丸め規則の突合が残る。 | https://apidocs.bithumb.com/reference/%EA%B8%B0%EB%B3%B8-%EC%A0%95%EB%B3%B4 |
| Account (P0) | Yes | balances/positions の項目は参照可能、定義整合は追加検証要。 | https://apidocs.bithumb.com/reference/%EA%B8%B0%EB%B3%B8-%EC%A0%95%EB%B3%B4 |
| Transfer | NA | 取引所によって提供有無が異なるため、該当時のみ EXD-C で実証予定。 | https://apidocs.bithumb.com/reference/%EA%B8%B0%EB%B3%B8-%EC%A0%95%EB%B3%B4 |
| WebSocket (P0) | Yes | WS接続/購読系の資料はあるが、認証ハンドシェイク/再接続の精査が必要。 | https://apidocs.bithumb.com/reference/WEBSOCKET |
| Error model | No | HTTP と独自エラーコードの対応・再試行可否分類が未確定。 | https://apidocs.bithumb.com/ |

## 4) Auth（署名ベース文字列の完全記述、サンプル一致）
- Current state: **No**（署名方式の完全突合未確定）
- Pending verification points:
  - signature payload に `method + path + query + body` をどう連結するか
  - query ソート規則、body canonicalization、改行/区切り
  - timestamp 単位（秒/ms）と許容 skew
  - エンコード（hex/base64）
- Evidence URL: https://apidocs.bithumb.com/
- Fix target files:
  - `docs/exchanges/bithumb/rest-api.md`
  - `docs/exchanges/bithumb/websocket.md`（存在時）

## 5) P0 Endpoints（仕様の要点と差分）
- ticker/orderbook/trades/klines:
  - 差分種別: **不足**（返却フィールド定義・sequence処理・欠損時挙動の明示が不足）
  - 根拠: https://apidocs.bithumb.com/reference/%EA%B8%B0%EB%B3%B8-%EC%A0%95%EB%B3%B4
  - 修正先: `docs/exchanges/bithumb/data.md`, `docs/exchanges/bithumb/rest-api.md`
- order create/cancel/status:
  - 差分種別: **不足**（必須/任意、timeInForce/postOnly/reduceOnly、部分約定時挙動）
  - 根拠: https://apidocs.bithumb.com/reference/%EA%B8%B0%EB%B3%B8-%EC%A0%95%EB%B3%B4
  - 修正先: `docs/exchanges/bithumb/rest-api.md`
- balances/positions:
  - 差分種別: **不足**（available/locked 定義、デリバの証拠金/平均建値）
  - 根拠: https://apidocs.bithumb.com/reference/%EA%B8%B0%EB%B3%B8-%EC%A0%95%EB%B3%B4
  - 修正先: `docs/exchanges/bithumb/rest-api.md`
- WS public/private:
  - 差分種別: **不足**（subscribe payload・ping/pong・reconnect/resubscribe）
  - 根拠: https://apidocs.bithumb.com/reference/WEBSOCKET
  - 修正先: `docs/exchanges/bithumb/websocket.md`

## 6) Rate limit / Error model
- Rate limit: **No**（weight制/頻度制/IP-or-Account単位の明示が不足）
  - Evidence: https://apidocs.bithumb.com/changelog/%EC%97%85%EB%8D%B0%EC%9D%B4%ED%8A%B8-public-websocket-%EB%8D%B0%EC%9D%B4%ED%84%B0-%EC%9A%94%EC%B2%AD-%EC%88%98-%EC%A0%9C%ED%95%9Crate-limit-%EC%A0%81%EC%9A%A9-%EC%95%88%EB%82%B4
- Error model: **No**（HTTP status と取引所独自コードの対応、retryable判定が不足）
  - Evidence: https://apidocs.bithumb.com/

## 7) Findings（P0/P1/P2、根拠URL、修正先ファイル）
- P0-1: 認証署名仕様（ベース文字列・timestamp・header）を公式と1対1で固定化できていない。
  - Evidence: https://apidocs.bithumb.com/
  - Fix: `docs/exchanges/bithumb/rest-api.md`, `docs/exchanges/bithumb/websocket.md`
- P0-2: WS 維持運用（heartbeat/reconnect/sequence復元）の確定記述が不足。
  - Evidence: https://apidocs.bithumb.com/reference/WEBSOCKET
  - Fix: `docs/exchanges/bithumb/websocket.md`
- P1-1: Rate limit と retry/backoff 指針の統一表現が不足。
  - Evidence: https://apidocs.bithumb.com/changelog/%EC%97%85%EB%8D%B0%EC%9D%B4%ED%8A%B8-public-websocket-%EB%8D%B0%EC%9D%B4%ED%84%B0-%EC%9A%94%EC%B2%AD-%EC%88%98-%EC%A0%9C%ED%95%9Crate-limit-%EC%A0%81%EC%9A%A9-%EC%95%88%EB%82%B4
  - Fix: `docs/exchanges/bithumb/rest-api.md`, `docs/exchanges/bithumb/README.md`
- P1-2: 注文・数量・価格の精度/丸め規則の明確化が不足。
  - Evidence: https://apidocs.bithumb.com/reference/%EA%B8%B0%EB%B3%B8-%EC%A0%95%EB%B3%B4
  - Fix: `docs/exchanges/bithumb/rest-api.md`, `docs/exchanges/bithumb/data.md`
- P2-1: 版管理（現行/旧版/非推奨）と changelog 関連リンクの導線改善余地。
  - Evidence: https://apidocs.bithumb.com/changelog
  - Fix: `docs/exchanges/bithumb/README.md`, `docs/exchanges/bithumb/sources.md`

## 8) Smoke Test（実施した/できない理由、ログへのリンクパス）
- REST: 未達（ネットワーク制約）: <urlopen error [Errno -5] No address associated with hostname>
  - Log: `docs/exchanges/_verification/evidence/bithumb/smoke_rest.txt`
- WS: 未実施（sandboxで外部WS疎通不可）
  - Log: `docs/exchanges/_verification/evidence/bithumb/smoke_ws.txt`
- Private(read-only): 未実施（鍵未設定・秘匿方針）

## 9) Fix Plan（修正方針）
- C例外解除条件: 外部ネットワーク疎通可能な環境で Public REST/WS の再スモークを実行。
- docs本体のP0修正は issues_summary の修正先ファイルに沿って継続。

## 10) Status（A完了/B完了/C完了、確定/例外）
- A完了 / B完了 / C例外（ネットワーク制約によりスモーク未達）
