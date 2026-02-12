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


## Step 8 defaults
1. Reconciliation submissions persist in `reconcile_results` for historical query and auditability.
2. A `MISMATCH` reconciliation status opens a WARNING alert and routes outbound notification via existing router/webhook path.
3. Reconciliation list API supports pagination and basic filters (`instance_id`, `status`).


## Step 9 defaults
1. Cost ingest accepts only `FEE` and `FUNDING` cost types for MVP NetPnL formula compliance.
2. NetPnL summary is derived from latest realized/unrealized metrics and summed cost ledger amounts.
3. NetPnL API supports optional symbol filter and returns UTC timestamped summary payload.


## Step 10 defaults
1. Execution quality ingest stores slippage/latency/fill ratio samples in `execution_quality_ts`.
2. Execution quality summary returns simple averages and sample count for MVP.
3. Symbol filter is optional and list-style aggregations remain admin-protected on analytics endpoints.


## Step 11 defaults
1. Module runs are queued via admin-triggered endpoint and persisted in `module_runs`.
2. Module run status transitions are updated through admin endpoint with optional completion timestamp/summary.
3. Module run list remains paginated and filterable by `module_id` and `status`.


## Step 12 defaults
1. Module runs can be canceled only from non-terminal states (`QUEUED`, `RUNNING`).
2. Module run stats provide lightweight aggregated counts by status for operations visibility.
3. Module run cancellation/stats endpoints are admin-protected and UTC timestamped.


## Step 13 defaults
1. Stuck module run detection only considers non-terminal run statuses (`QUEUED`, `RUNNING`).
2. WARNING alerts for stuck runs are deduplicated by OPEN alert per `module_run` target.
3. Stuck-check endpoint is admin-protected and can optionally notify via existing webhook router.
