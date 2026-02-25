# Versioning Policy

## Contracts and Core Specs
- `docs/contracts/*.schema.json` and `docs/specs/ucel/` follow SemVer.
- Breaking changes MUST bump the major version.
- New fields with defaults MAY bump the minor version.

## Policy, Plan, Runbook
- `docs/policy/`, `docs/plans/`, `docs/runbooks/` can change without bumping contract/core spec versions.
- They MUST remain consistent with contracts and core specs.

## Implementation Notes
- `docs/context/notes/` are non-normative and have no versioning requirement.
