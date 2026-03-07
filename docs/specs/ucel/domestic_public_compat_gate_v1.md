# Domestic Public Compatibility Gate v1

This specification defines the final CI gate for the domestic public API series (009A-009F).

## Gate checks
1. Workspace scope: domestic venues are fixed to `bitbank`, `bitflyer`, `coincheck`, `gmocoin`, `bittrade`, `sbivc`.
2. Inventory completeness: `jp_public_inventory.json` must remain schema-valid and contain unique stable identifiers (`venue|api_kind|public_id`).
3. Route reachability: every inventory entry (except explicit `not_supported`) must resolve to a registered route/channel.
4. Canonical/extension consistency: `surface_class` and SDK/registry hub surfaces must remain aligned.
5. Docs/matrix consistency: final support report, compat matrix, and change management must match inventory counts.
6. Fixture/golden coverage: required golden families must exist and are read-only in CI.
7. Schema/runtime evolution: extension schema and runtime modes follow non-breaking policy.
8. Venue drift: add/delete/rename in workspace crates must fail unless inventory/lock/docs are updated together.

## Lock policy
- `ucel/coverage_v2/domestic_public/jp_public_inventory.lock.json` is the pinned final snapshot.
- Any intentional inventory change must update lock + docs + tests in the same PR.
- Count-only matches are insufficient: stable identifiers are also compared.

## Fail rules
- Missing route for any supported inventory entry.
- Any mismatch between inventory and lock summary/identifiers.
- Any mismatch between inventory summary and final compat docs.
- Missing golden files for any required family.
- `partial` or `not_implemented` status remains in domestic public inventory.
