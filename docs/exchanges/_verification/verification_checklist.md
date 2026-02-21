# Verification Checklist (Common)

各項目を取引所別レポートの Coverage/Auth/P0/Findings に Yes/No/NA + 根拠URL付きで記録する。

## 1. Version / Lifecycle
- [ ] API version (v1/v2/v3) が docs/exchanges 記載と一致
- [ ] 現行/旧版/非推奨の区分が明記

## 2. Authentication (P0)
- [ ] 認証方式（HMAC/RSA/ED25519/JWT）
- [ ] 必須ヘッダ（API-KEY/SIGN/TIMESTAMP/PASSPHRASE/RECV-WINDOW 等）
- [ ] timestamp 単位（秒/ミリ秒）
- [ ] 署名ベース文字列（method/path/query/body/区切り）
- [ ] エンコード（hex/base64）
- [ ] 公式サンプル一致
- [ ] 認証エラー体系

## 3. Rate Limit
- [ ] 単位（req/s, req/min, weight 等）
- [ ] 公開/認証別
- [ ] 適用単位（IP/Account/Key）
- [ ] Retry-After / backoff 指針

## 4. Market Data (P0)
- [ ] ticker
- [ ] orderbook（snapshot/incremental/sequence）
- [ ] trades
- [ ] klines/candles

## 5. Trading (P0)
- [ ] 新規注文（type/TIF/postOnly/reduceOnly）
- [ ] 取消（単一/一括）
- [ ] 注文照会（open/status/fills）
- [ ] 数量・価格精度と丸め

## 6. Account (P0)
- [ ] balances
- [ ] positions（デリバティブ取引所）

## 7. Transfer
- [ ] deposit/withdraw（該当時）

## 8. WebSocket (P0)
- [ ] 接続URL / ping-pong / keepalive
- [ ] subscribe payload
- [ ] public channels
- [ ] private channels
- [ ] 認証ハンドシェイク
- [ ] reconnect/resubscribe/sequence 復元

## 9. Error model
- [ ] HTTP status + exchange固有コード
- [ ] retryable / non-retryable 分類
- [ ] 代表例（認証・レート制限・バリデーション）
