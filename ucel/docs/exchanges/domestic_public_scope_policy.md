# Domestic Public Scope Policy (JP)

## Fixed scope for UCEL-DOMESTIC-PUBLIC-INVENTORY-009A
Domestic venues are pinned to:
- bitbank
- bitflyer
- coincheck
- gmocoin
- bittrade
- sbivc

## Evidence sources
- workspace membership: `ucel/Cargo.toml`
- registry registration: `ucel/crates/ucel-registry/src/hub/registry.rs`
- exchange catalogs: `docs/exchanges/<venue>/catalog.json`
- coverage evidence: `ucel/coverage/<venue>.yaml`
- policy evidence: `ucel/docs/policies/coverage_policy.md`, `ucel/docs/policies/venue_access_policy.md`

## Public detection rule
An endpoint/channel is treated as public if either:
- catalog `visibility == "public"`, or
- catalog visibility is omitted but id contains `.public.` (and not `.private.`)

This rule exists because several catalogs omit explicit `visibility` fields while still encoding public/private via id naming.
