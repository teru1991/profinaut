# Diff Notes (bithumb)

- Confirmed date: 2026-02-21

## P0
- Auth signature/timestamp/header の公式突合が未確定（実装修正前提）。
  - URL: https://apidocs.bithumb.com/
- WS keepalive/reconnect/sequence 復元ルールの確定記述が不足。
  - URL: https://apidocs.bithumb.com/reference/WEBSOCKET

## P1
- Rate limit 適用単位（IP/Account/Key）と retry/backoff 指針の明示不足。
  - URL: https://apidocs.bithumb.com/changelog/%EC%97%85%EB%8D%B0%EC%9D%B4%ED%8A%B8-public-websocket-%EB%8D%B0%EC%9D%B4%ED%84%B0-%EC%9A%94%EC%B2%AD-%EC%88%98-%EC%A0%9C%ED%95%9Crate-limit-%EC%A0%81%EC%9A%A9-%EC%95%88%EB%82%B4
- 数量・価格精度/丸め規則の明示不足。
  - URL: https://apidocs.bithumb.com/reference/%EA%B8%B0%EB%B3%B8-%EC%A0%95%EB%B3%B4

## P2
- 旧版/非推奨の区分と changelog 導線の整理余地。
  - URL: https://apidocs.bithumb.com/changelog
