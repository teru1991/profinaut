# SecretRef & Providers Contract (Domain B / Step2)

## Purpose
Unify secret access via `SecretRef` and enforce:
- fail-closed validation
- prod plaintext/env forbidden
- asset registry required
- short TTL cache

## SecretRef format (minimum)
`<scheme>://<path>#<field>?registry_id=<id>&scope=<scope>&version_hint=<v>`

### Required
- scheme: fileenc | vault | env
- path
- field
- registry_id
- scope (structured; contains ':')

### Optional
- version_hint

## Rules
1) Unscoped SecretRef is invalid (missing scope => reject).
2) Registry enforcement:
- registry_id must exist in `docs/policy/asset_registry.json`
- scheme/scope/ttl must satisfy entry
3) Prod restrictions:
- env scheme forbidden
- plaintext json secrets forbidden (fileenc provider only accepts `.enc` in prod; `.json` is dev-only until Step3 completes encryption)
4) Fail-closed:
- any parse/validation/provider misconfig => raise categorized error; never emit secret material in messages.

## Tests
See `tests/test_secretref_and_provider.py`
