# UCEL Support Bundle Spec v1

Support bundle output is JSON with deterministic top-level keys:
- `metadata` (`version`, `run_id`, `timestamp_unix`)
- `ssot` (`coverage_hash`, `rules_version`)
- `transport` (state/counters/limits)
- `hub` (venue summary/capabilities counts)
- `wal` (recent segment metadata and latency stats)
- `errors` (recent classified error summary)

## Redaction / prohibited fields
The following keys or equivalent secret materials MUST NOT appear:
- `api_key`
- `api_secret`
- `Authorization`
- `Cookie`
- `signature`
- `query_secret`
- `body_secret`

Raw frame payloads are prohibited in support bundles.

## API contract
SDK exposes a single entrypoint:
- `ucel_sdk::support_bundle::generate_support_bundle`
