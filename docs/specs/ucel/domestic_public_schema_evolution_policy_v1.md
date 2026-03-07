# Domestic Public Schema Evolution Policy v1

## Canonical surfaces
- Breaking changes are forbidden on canonical core/extended response contracts.
- Field removal, semantic rename, or enum narrowing is breaking.
- Additive optional fields are non-breaking.

## Vendor public extensions
- Schema version follows `major.minor.patch`.
- `major`: breaking layout/type/rule changes.
- `minor`: additive fields or additive enum variants.
- `patch`: typo/doc/metadata-only adjustments.

## Runtime mode policy
- Runtime mode/key removal or rename is breaking.
- Runtime mode additive behavior with backward-compatible default is non-breaking.
- Missing schema version metadata is treated as gate failure.

## Enforcement
- Final compat tests validate required schema/runtime policy documents and route/inventory coherence.
- `partial/not_implemented` is not allowed at final gate.
