# Observability Contracts (C SSOT entrypoint)

This directory defines the contract entrypoint for C-domain observability responses used by runtime endpoints and CI gates.

## Schemas

- `docs/contracts/observability/correlation.schema.json`
- `docs/contracts/observability/healthz.schema.json`
- `docs/contracts/observability/capabilities.schema.json`
- `docs/contracts/observability/log_event.schema.json`

## Versioning rules

- `schema_version` values are immutable contract identifiers.
- Backward-compatible additive changes are allowed for the same schema major (`*.v1`).
- Removing required keys or changing existing key types is a breaking change and must fail contract tests until a new schema version is introduced.

## C-0 principle

- Missing signal must be represented as `UNKNOWN`.
- Degraded behavior must be represented as `DEGRADED`.
- Not-implemented capabilities must use `NOT_IMPLEMENTED` with reasons instead of reporting success.

## Policy linkage

- Capability/health payloads should avoid sensitive keys and align with `docs/policy/forbidden_keys.toml`.

## Structured logging contract (C-2)

- Structured logs MUST conform to `docs/contracts/observability/log_event.schema.json`.
- Required keys: `schema_version`, `ts`, `level`, `msg`, `logger`, `service`, `op`, `run_id`, `instance_id`.
- In STRICT mode (`PROFINAUT_OBS_LOG_STRICT=1`), missing required keys are treated as errors in tests/CI.
- Forbidden keys from `docs/policy/forbidden_keys.toml` are masked in log `fields` payloads.

## Data classification and redaction

- Data classification SSOT: `docs/specs/observability/data_classification.md`
- Runtime redaction policy: `docs/policy/redaction.toml`
- Forbidden key source: `docs/policy/forbidden_keys.toml`
