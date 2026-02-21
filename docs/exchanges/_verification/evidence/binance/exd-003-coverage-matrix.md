# Binance EXD-003 Coverage Matrix

- confirmed_date: 2026-02-21
- basis:
  - docs/exchanges/binance/rest-api.md
  - docs/exchanges/binance/websocket.md
  - docs/exchanges/binance/sources.md

## Functional coverage checks
- Market Data: **Yes**
  - ticker/trades/orderbook/klines/exchangeInfo(instruments) を `rest-api.md` で確認。
- Trading: **Yes (partial)**
  - new order / cancel / order status は記載あり。
  - amend / batch は Spot REST catalog で明示的記載が不足（差分項目）。
- Account: **Yes (partial)**
  - balances/account/myTrades/openOrders は記載あり。
  - positions/leverage/margin は Spot 現物の適用範囲外として NA 扱いが必要。
- Transfer: **Partial**
  - user data balance update はあるが、deposit/withdraw/address の REST endpoint catalog 記載が不足。
- WebSocket: **Yes**
  - Public streams / Private userData / WS-API auth handshake / ping-pong 記載あり。
- Rate limit: **Yes (partial)**
  - limits 参照はあるが、weight/IP/API key/ban/backoff を1箇所に集約した規約説明が不足。
- Error model: **Yes (partial)**
  - error-codes 参照はあるが、HTTP vs exchange code と retry方針の実装向け整理が不足。

## Gaps to resolve in next tasks
1. `rest-api.md` に Spot の amend/batch の扱い（対応有無・代替）を明記。
2. Transfer領域（deposit/withdraw/address）を Spot docs の対応範囲として追記/NA整理。
3. Rate limit を実装規約として統合（weight, order count, IP ban, backoff）。
4. Error model を HTTP/コード/リトライ可否の表で正規化。
