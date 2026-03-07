# IR Inventory SSOT v1

This spec defines machine-readable inventory fields for JP/US IR source universe.

## Inventory object
- `version`
- `markets[]`
- `sources[]`
- `identities[]`
- `documents[]`

## source required fields
- `market`, `source_family`, `source_id`, `source_kind`
- `access_policy_class`, `access_patterns[]`
- `issuer_identity_kind[]`
- `document_family[]`
- `artifact_kind[]`
- `current_repo_status`
- `evidence_files[]`, `evidence_kinds[]`, `notes`

## identity required fields
- `market`, `identity_kind`, `source_id`, `canonical_role`
- `evidence_files[]`, `notes`

## document required fields
- `market`, `source_id`, `document_family`, `artifact_kind`
- `issuer_identity_kind`, `access_pattern`, `access_policy_class`
- `current_repo_status`, `evidence_files[]`, `notes`

## Fail conditions
- Evidence-backed source missing in inventory.
- Taxonomy class/value missing or duplicated ambiguously.
- Excluded source marked as implementation target.
