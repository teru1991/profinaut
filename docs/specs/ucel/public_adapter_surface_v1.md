# UCEL Public Adapter Surface v1

This spec defines canonical public market-data operations available through UCEL Hub/Registry/SDK.

## Canonical public REST
- get_ticker
- get_trades
- get_orderbook_snapshot
- get_candles
- list_symbols
- get_market_meta

## Canonical public WebSocket
- subscribe_ticker
- subscribe_trades
- subscribe_orderbook
- subscribe_candles (venue dependent)

## Runtime reason codes
- NotSupported
- SubscriptionRejected
- AckTimeout
- HeartbeatTimeout
- ChecksumMismatch
- GapDetected
- RateLimited
- RetryableTransport

## Policy
Public-only venues are first-class supported for market data even when private auth/execution surfaces are intentionally blocked by policy.
