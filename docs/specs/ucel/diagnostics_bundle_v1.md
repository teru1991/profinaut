# UCEL Diagnostics Bundle v1

## Manifest schema
Bundle must include `manifest` with:
- `diag_semver`
- `generated_at`
- `generator_id`
- `build_info`
- `coverage_hash`
- `coverage_v2_hash`
- `ws_rules_hash`
- `catalog_hash`
- `policy_hash`
- `symbol_meta_hash`
- `execution_surface_hash`
- `runtime_capability_hash`
- `bundle_redaction_version`
- `runtime_capabilities`

## Hash policy
- Deterministic file ordering (lexicographic path order)
- Newline normalization (`\r\n` and `\r` => `\n`)
- SHA-256 canonical hashing
- `unknown` or empty hash values are invalid

## Security
- Bundle must not contain secrets/tokens/private payload/auth headers/signatures.
