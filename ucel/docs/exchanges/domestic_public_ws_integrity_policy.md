# Domestic Public WS Integrity Policy

## Integrity modes
- none: no sequence/checksum checks.
- snapshot_only: full-book replacement only.
- sequence_only: monotonic sequence validation and gap detection.
- checksum_only: checksum validation only.
- sequence_and_checksum: both monotonic sequence and checksum validation.

## Required checks
- crossed-book detection (`best_bid > best_ask`) -> checksum mismatch.
- negative quantity detection -> checksum mismatch.
- duplicate or stale sequence detection -> gap detected.
- checksum mismatch detection -> checksum mismatch.

## Failure mapping
- gap detected -> deadletter `GapDetected`.
- checksum mismatch -> deadletter `ChecksumMismatch`.
- heartbeat timeout -> deadletter `HeartbeatTimeout`.
- transport close -> deadletter `TransportClosed`.
