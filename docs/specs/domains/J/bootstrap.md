# Bootstrap Self-Check (SSOT)

At startup (and periodically), the system MUST perform self-check:
1) SSOT load/validate
   - All required J SSOT files exist and parse
   - Key uniqueness & references are valid
2) Required inputs availability
   - metrics pipeline alive
   - clock health measurable
   - audit chain head readable
   - safety lease system reachable (even if lease not granted yet)
   - dependency health probes operational
3) Fail-close behavior
   - If any critical self-check fails => recommended mode CANCEL_ONLY or HALT
4) Evidence
   - self-check results must be emitted with required_log_keys (observability_contract.yml)
