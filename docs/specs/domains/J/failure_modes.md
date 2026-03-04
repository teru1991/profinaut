# J Failure Modes (SSOT)

## Principle
- Unknown / missing / invalid => fail-close (DENY/CANCEL_ONLY/HALT)
- Audit chain broken => HALT (latch)
- Lease missing/expired => block live operations
- Observability missing (required metrics/log keys missing) => fail-close

## Typical failure modes
1) SSOT parse failure (YAML invalid / missing file)
   - Expected action: fail-close (HALT)
2) Required inputs missing (metrics/clock/audit/lease/deps)
   - Expected action: fail-close (HALT)
3) Audit chain continuity broken
   - Expected action: HALT (latch)
4) Lease expired or renewer unhealthy
   - Expected action: DENY/CANCEL_ONLY for live ops
5) Dependency SLO breach (exchange API down / DB unavailable)
   - Expected action: DEGRADED or CANCEL_ONLY (per dependency_slo.yml)
6) Quiet hours restrictions
   - Expected action: DENY for restricted ops
