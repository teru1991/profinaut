# sources.md (SSOT Evidence)

Columns:
category | title | url | last_checked_at(YYYY-MM-DD) | notes

| category | title | url | last_checked_at(YYYY-MM-DD) | notes |
|---|---|---|---|---|
| other | Coincheck 公式サイト（JP） | https://coincheck.com/ja/ | 2026-02-19 | 公式トップ。取引所APIドキュメントへの導線あり。 |
| rest | 取引所 API ドキュメント（JP） | https://coincheck.com/ja/documents/exchange/api | 2026-02-19 | REST Public/Private の一次情報。 |
| auth | 認証（ACCESS-KEY / ACCESS-NONCE / ACCESS-SIGNATURE） | https://coincheck.com/ja/documents/exchange/api#auth | 2026-02-19 | Private API 署名方式（HMAC-SHA256）とヘッダ仕様。 |
| rest | Public API セクション | https://coincheck.com/ja/documents/exchange/api#public | 2026-02-19 | ティッカー・約定履歴・板・レート・ステータス。 |
| rest | Private API セクション | https://coincheck.com/ja/documents/exchange/api#private | 2026-02-19 | 注文・残高・送受金・銀行口座・出金。 |
| ws | WebSocket API セクション | https://coincheck.com/ja/documents/exchange/api#websocket | 2026-02-19 | Public/Private WebSocket の導線。 |
| ws | Public WebSocket API | https://coincheck.com/ja/documents/exchange/api#public-channels | 2026-02-19 | wss://ws-api.coincheck.com での購読方式。 |
| ws | Private WebSocket API | https://coincheck.com/ja/documents/exchange/api#private-channels | 2026-02-19 | wss://stream.coincheck.com/private の login/subscribe 仕様。 |
| errors | エラーレスポンス（各 endpoint の success/error 応答） | https://coincheck.com/ja/documents/exchange/api | 2026-02-19 | 各サンプルで success:boolean / error:string 形式を確認。 |
| rate_limit | 注文系リクエスト制限の記載 | https://coincheck.com/ja/documents/exchange/api#order-new | 2026-02-19 | 新規注文・注文詳細に「リクエスト制限について」の節あり。 |
| rate_limit | 注文詳細のリクエスト制限の記載 | https://coincheck.com/ja/documents/exchange/api#order-show | 2026-02-19 | 明示的な数値上限はドキュメント本文未記載。 |
| data | 取引所APIドキュメント（データ配布導線確認） | https://coincheck.com/ja/documents/exchange/api | 2026-02-19 | dump/ファイル配布型データフィードの記載は未確認。 |
