# Analyzer Policy

Analyzer output must include:
- `summary.json` equivalent data (generator/build/runtime/hash visibility)
- compatibility status
- drift findings (`info`/`warn`/`error`)

Analyzer must fail-closed on unsupported major semver and malformed manifest.
