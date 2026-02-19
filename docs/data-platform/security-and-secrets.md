# Security and Secrets

## 1. Secrets non-persistence policy
- API key/secret/token/signature は保存禁止
- request/response の raw dump でも秘匿情報を保持しない
- Bronze 以前（collector memory）で scrub 完了させる

## 2. Data classes
- Public market data: bronze/silver/gold に保存可
- Account/private data: 最小限の必要項目のみ保存
- Credentials/secrets: 保存不可、ログ出力不可

## 3. Key management
- 実行時注入（env/secret manager）
- ローテーション前提
- 失効時は fail-closed

## 4. Logging rules
- denylist key/value を強制マスク
- 例外スタックにも credential を残さない

## 5. Access control
- Bronze/Silver/Gold は least privilege
- Serving DB は read/write role 分離
