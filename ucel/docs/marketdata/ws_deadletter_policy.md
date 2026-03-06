# WS Deadletter Policy

Streams are deadlettered for:
- policy blocked
- repeated auth/ack/heartbeat failures beyond retry budget
- checksum mismatch beyond retry budget
- unrecoverable parse/spec violation

Deadletter records must include failure class and last known lifecycle state.
