# UCEL Private WS Surface v1

## Scope
JP resident policy 前提で private WebSocket を UCEL 共通面へ提供する。
private WS は Policy Gate + Auth Core を通過できる場合のみ接続される。

## Canonical channels
- balances
- orders
- fills
- positions
- session

## Lifecycle
- Connecting
- Authenticating
- Authenticated
- Subscribing
- Active
- ReauthPending
- ResubscribePending
- Failed
- Deadlettered

## ACK mode
- `explicit_ack`: auth/sub ack frame を待つ
- `implicit_observation`: 有効 private event の初回観測で Active 化
- `none`: private WS 未提供または policy blocked

## Auth patterns
- login frame
- token/bootstrap 連携
- signed subscribe
- session expiry / refresh / reauth

## Canonical private events
- CanonicalBalanceEvent
- CanonicalOrderEvent
- CanonicalFillEvent
- CanonicalPositionEvent
- CanonicalSessionEvent

## Reject normalization
- auth_failed
- entitlement_denied
- session_expired
- ack_timeout
- reauth_required
- subscription_rejected
- gap_detected
- transport_closed

## Retry / reconnect
- reconnect は許可
- private write 相当の副作用再送は禁止
- resubscribe は idempotent 前提
