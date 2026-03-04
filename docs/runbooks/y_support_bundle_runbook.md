# Y Runbook: Support Bundle / Triage / Export

## Principles
- Bundles are **secret-free** (fail-closed redaction).
- Analyze is read-only and safe.
- Export (external sharing) requires Break-Glass: TTL + reason + approvals, and is always encrypted.
- All actions write audit events (chain-hash) to `diagnostics_audit.jsonl`.

## Commands
### Analyze (offline)
- `UCEL_DIAG_AUDIT_PATH=diagnostics_audit.jsonl ucel-diag analyze --input <bundle.tar.zst> --output summary.json`

### Export (external)
- Requires approvals (default 2): configure `UCEL_DIAG_BG_APPROVALS_REQUIRED` when policy differs.
- `UCEL_DIAG_AUDIT_PATH=diagnostics_audit.jsonl ucel-diag export --input <bundle.tar.zst> --recipient-pubkey <ref> --output <bundle.enc.json> --ttl-minutes 60 --reason "<why>" --approval <token1> --approval <token2>`

## Audit
- Audit file is JSONL. Each line contains `prev_hash_hex` / `this_hash_hex` for tamper-evident chaining.
- Audit records action, actor, result, and break-glass metadata without bundle plaintext.

## Notes
- Current implementation encrypts with XChaCha20-Poly1305 and writes encrypted JSON payload.
- `recipient_pubkey` is recorded as audit/export reference in this minimal path; operational key exchange is out-of-band.
- If policy requires strict recipient public-key envelope encryption, upgrade to age/recipient-based encryption.
