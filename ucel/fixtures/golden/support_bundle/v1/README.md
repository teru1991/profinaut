# Golden Support Bundle Fixture (v1)

This directory is **read-only** for CI/PR workflows.

## Policy
- Do **not** generate or update golden binaries in automated jobs.
- `bundle_minimal.tar.zst` is optional and must be placed manually when external golden comparison is required.
- Core regression coverage is provided by synthetic bundle bytes generated in tests.

## Optional manual binary
- Path: `ucel/fixtures/golden/support_bundle/v1/bundle_minimal.tar.zst`
- Contents (tar entries):
  1. `manifest.json`
  2. `meta/diag_semver.txt` => `1.0.0\n`
  3. `meta/info.json` => `{"k":"v"}`
  4. `logs/tail.txt` => `hello\n`
- Manifest file list must have correct `size_bytes` and SHA-256 checksums for each payload file.

## Test behavior
- `synth_analyze` always runs in CI and compares analyzer output against `expected.summary.json`.
- `golden_external` runs only when `UCEL_GOLDEN_BUNDLE_DIR` is set.
