# Egress & Governance Contract (Domain B / Step5)

## Egress
- Any payload sent to external targets (llm/public_http) MUST go through egress_guard.prepare_egress().
- If policy denies target or secret indicators remain -> deny (fail-closed).
- Payload must be redacted JSON only.

## No plaintext secrets in prod
- prod MUST reject env provider and plaintext secret files.
- Secret access must be via SecretRef + registry enforcement.

## Governance
- change_mgmt_policy controlled changes are treated as dangerous ops.
- access_review report is generated monthly and contains no secrets.
