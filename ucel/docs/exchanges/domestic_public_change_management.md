# Domestic Public Change Management

## In-scope venues (fixed)
- bitbank
- bitflyer
- coincheck
- gmocoin
- bittrade
- sbivc

## Required update order for any public API change
1. Update SSOT specs (`docs/specs/ucel/domestic_public_*`).
2. Update `jp_public_inventory.json` and validate schema.
3. Update `jp_public_inventory.lock.json` (counts + stable identifiers).
4. Update compatibility docs:
   - `domestic_public_final_support_report.md`
   - `domestic_public_compat_matrix.md`
   - this file
5. Add/update fixture goldens under `ucel/fixtures/domestic_public_goldens/`.
6. Run domestic compatibility tests and workspace tests.

## Drift policy
- Venue add/delete/rename is always breaking unless inventory+lock+docs+tests updated together.
- Route mismatch, docs mismatch, or missing fixtures are strict CI failures.
- CI never regenerates goldens; it only compares existing artifacts.
