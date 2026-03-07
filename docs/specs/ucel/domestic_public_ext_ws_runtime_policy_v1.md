# domestic_public_ext_ws_runtime_policy_v1

Runtime policy for vendor public WS extension:
- readiness_mode: explicit_ack | implicit_observation | immediate_active
- integrity_mode: none | snapshot_only | sequence_only | checksum_only | sequence_and_checksum
- resume_mode: resubscribe_only | resnapshot_then_resubscribe | deadletter

Rules:
- inventory vendor WS entries must have one runtime triple.
- `resnapshot_then_resubscribe` requires non-none integrity mode.
- runtime mode mismatch between docs/code is CI failure.
