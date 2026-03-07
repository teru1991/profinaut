# domestic_public_ws_runtime_v1

Task: UCEL-DOMESTIC-PUBLIC-WS-009C

## Readiness modes
- explicit_ack
- implicit_observation
- immediate_active

## Integrity modes
- none
- snapshot_only
- sequence_only
- checksum_only
- sequence_and_checksum

## Runtime requirements
- subscribe/unsubscribe
- heartbeat and ping/pong
- reconnect with bounded backoff
- resubscribe from active plan
- snapshot bootstrap and integrity checks for orderbook streams
- deadletter reason mapping for ack timeout, heartbeat timeout, gaps, checksum mismatch, and transport close
