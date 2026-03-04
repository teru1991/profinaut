# Runbook: WS Health + Support Bundle Endpoint

1. Query `/healthz` and inspect `status` (`Healthy/Degraded/Unhealthy/Unknown`) and `reasons`.
2. If status is `Degraded`/`Unhealthy`, query `/support_bundle` and archive output.
3. Confirm bundle includes `manifest.json` (`diag_semver` must be present), `health`, `metrics`, `events_tail`, and `rules_snapshot`.
4. Check reconnect and rate-limit counters to identify storm/penalty conditions.
5. Share the secret-free bundle with on-call or vendor support for triage.
