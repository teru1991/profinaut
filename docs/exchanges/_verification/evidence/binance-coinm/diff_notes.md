# Diff Notes (binance-coinm)

- Confirmed date: 2026-02-21

## P0
- Auth signature/timestamp/header の公式突合が未確定（実装修正前提）。
  - URL: https://developers.binance.com/docs/derivatives/coin-margined-futures
- WS keepalive/reconnect/sequence 復元ルールの確定記述が不足。
  - URL: https://developers.binance.com/docs/derivatives/coin-margined-futures/websocket-market-streams

## P1
- Rate limit 適用単位（IP/Account/Key）と retry/backoff 指針の明示不足。
  - URL: https://developers.binance.com/docs/derivatives/coin-margined-futures
- 数量・価格精度/丸め規則の明示不足。
  - URL: https://developers.binance.com/docs/derivatives/coin-margined-futures/rest-api

## P2
- 旧版/非推奨の区分と changelog 導線の整理余地。
  - URL: https://developers.binance.com/docs/derivatives/change-log
