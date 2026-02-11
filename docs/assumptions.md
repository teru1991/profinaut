# Assumptions

## Step 0 defaults
1. Development starts on Windows 11 with Docker Desktop installed and WSL2 backend enabled.
2. Team members on macOS/Linux can use npm scripts that proxy to Docker Compose commands.
3. Step 0 creates runnable infrastructure placeholders only; application runtime implementation starts in Step 1 and Step 2.
4. PostgreSQL is the sole control-plane/state database for MVP.
5. Time values across backend storage will be UTC ISO-8601 once API contracts are introduced in Step 1.
6. Admin authentication in MVP uses `X-Admin-Token` sourced from `.env` (implemented in Step 2).
7. Discord support before Step 7 is outbound webhook notifications only.

## Step 1 defaults
1. OpenAPI uses version `1.0.0` as the first SSOT baseline for V2.5+ bootstrap.
2. Contract validation in CI uses Redocly CLI (OpenAPI lint) and Python `jsonschema` (schema correctness).
3. Contract files are additive and backward-compatible at this stage; no breaking changes are introduced.
4. List endpoints include pagination parameters (`page`, `page_size`) in the OpenAPI contract baseline.

## Step 2 defaults
1. Backend uses synchronous SQLAlchemy sessions for MVP simplicity.
2. Heartbeat endpoint allows unauthenticated ingest in Step 2; admin endpoints require `X-Admin-Token`.
3. `POST /ingest/heartbeat` auto-creates missing `bots` and `instances` records for onboarding.
4. Module registry CRUD is implemented on core table `modules`; execution scheduling starts in later steps.

## Step 3 defaults
1. Frontend uses Next.js App Router + TypeScript.
2. Bots polling is implemented through Next API route proxy to avoid hardcoding token in browser code.
3. Navigation skeleton pages are intentionally thin until functional modules are implemented in later steps.

## Step 4 defaults
1. Python SDK command source can be HTTP pull or local file queue for development/testing.
2. Dead-man switch defaults to `SAFE_MODE` when control plane is unreachable past timeout; `FLATTEN` is configurable.
3. SDK command processor is strict on idempotency and TTL and returns `REJECTED_DUPLICATE` / `REJECTED_EXPIRED`.

## Step 5 defaults
1. Command records persist in `commands`, and acknowledgements persist in `command_acks`.
2. Audit logs capture command creation and ack events from all control paths.
3. Agent command pull uses unauthenticated pending-command endpoint keyed by `instance_id` in Step 5; hardening can be added in later steps.

## Step 6 defaults
1. Notification router includes severity skeleton for INFO/WARNING/CRITICAL/AUDIT.
2. Heartbeat-loss detection is triggered via API endpoint and raises CRITICAL alerts.
3. Discord webhook notifications are outbound-only and optional (no two-way interactions before Step 7).

## Step 7 defaults
1. Time-series metrics foundation is implemented in `metrics_ts` with control-plane summary queries from Postgres for MVP.
2. Current positions are stored in `positions_current` and aggregated by symbol for exposure summary.
3. Portfolio UI reads exposure summary through a Next server route proxy.
