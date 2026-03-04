# Rotation & Revocation Runbook (Domain B)

## Principles
- Fail-closed: if audit health down, do not run dangerous ops.
- No plaintext secrets in prod.
- Rotate by key pools; revoke immediately on suspicion.

## Rotation (fileenc)
1) Prepare new key material (out of band).
2) Store into encrypted `.enc` using `scripts/fileenc_tool.py encrypt` with correct context.
3) Update SecretRef references (registry_id/scope) if needed.
4) Run smoke tests in staging.
5) Promote to prod.

## Revocation / Leak suspected
1) Freeze dangerous ops: set policy to deny rotate/withdraw/start_live unless break-glass.
2) Revoke compromised keys at venue/provider.
3) Rotate and redeploy.
4) Generate access review report and attach to incident.

## Break-glass (if absolutely required)
- Must be controlled by policy and must be audited.
- Prefer “deny” unless explicit incident commander approval.
