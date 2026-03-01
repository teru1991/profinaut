# Runbook: WS Reconnect Storm

1. Confirm reconnect_attempts/reconnect_failure counters increase.
2. Check breaker state and reconnect backoff behavior.
3. Validate upstream status and DNS/TLS reachability.
4. If upstream degraded, keep breaker open and reduce reconnect pressure.
5. Collect support bundle for escalation.
