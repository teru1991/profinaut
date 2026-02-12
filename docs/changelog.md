# Changelog

## 2026-02-11 — Step 19 (Indices ingest + latest analytics)
- Added `POST /ingest/indices` for index time-series ingestion.
- Added `GET /analytics/indices/latest` for latest per-index values with optional filter.
- Added automated API test for index ingest and latest-summary behavior.
- Added `scripts/scaffold_step19.ps1`.
- Bumped OpenAPI contract version to `1.16.0`.

## 2026-02-11 — Step 18 (Module run active-age analytics)
- Added `GET /analytics/module-runs/active-age` for active-run age visibility.
- Added summary fields: `active_runs`, `oldest_active_seconds`, `avg_active_seconds`.
- Added automated API test for active-run age summary behavior.
- Added `scripts/scaffold_step18.ps1`.
- Bumped OpenAPI contract version to `1.15.0`.

## 2026-02-11 — Step 17 (Module run throughput analytics)
- Added `GET /analytics/module-runs/throughput` for windowed run-throughput analytics.
- Added summary fields: `window_hours`, `total_runs`, `runs_per_hour`.
- Added automated API test for deterministic throughput calculation.
- Added `scripts/scaffold_step17.ps1`.
- Bumped OpenAPI contract version to `1.14.0`.

## 2026-02-11 — Step 16 (Module run failure-rate analytics)
- Added `GET /analytics/module-runs/failure-rate` for windowed FAILED-ratio analytics.
- Added summary fields: `total_completed`, `failed_runs`, `failure_rate`, `window_size_used`.
- Added automated API test for deterministic failure-rate calculation.
- Added `scripts/scaffold_step16.ps1`.
- Bumped OpenAPI contract version to `1.13.0`.

## 2026-02-11 — Step 15 (Module run performance analytics)
- Added `GET /analytics/module-runs/performance` for completion-rate and runtime duration analytics.
- Added summary fields: `total_runs`, `completed_runs`, `success_rate`, `avg_duration_seconds`, `p95_duration_seconds`.
- Added automated API test for deterministic duration and success-rate calculations.
- Added `scripts/scaffold_step15.ps1`.
- Bumped OpenAPI contract version to `1.12.0`.

## 2026-02-11 — Step 14 (Advanced analytics: equity drawdown)
- Added `GET /analytics/equity-drawdown` to summarize drawdown statistics from equity metrics.
- Added drawdown response schema with max/current drawdown fields.
- Added automated API test for drawdown calculation.
- Added `scripts/scaffold_step14.ps1`.
- Bumped OpenAPI contract version to `1.11.0`.

## 2026-02-11 — Step 13 (Stuck module-run alerting)
- Added `POST /alerts/module-runs/stuck-check` to detect stale `QUEUED`/`RUNNING` module runs.
- Added WARNING alert creation and webhook routing for newly detected stuck runs.
- Added dedup behavior to avoid duplicate OPEN alerts for the same module run.
- Added automated API test for stuck-run detection, notification, and dedup behavior.
- Added `scripts/scaffold_step13.ps1`.
- Bumped OpenAPI contract version to `1.10.0`.

## 2026-02-11 — Step 12 (Module run cancel + stats)
- Added `POST /module-runs/{run_id}/cancel` to cancel non-terminal module runs.
- Added `GET /module-runs/stats` for aggregated module run status counts.
- Added audit logging for module run cancellation.
- Added automated API test for run cancellation and stats summary.
- Added `scripts/scaffold_step12.ps1`.
- Bumped OpenAPI contract version to `1.9.0`.

## 2026-02-11 — Step 11 (Module execution controls)
- Added `POST /modules/{module_id}/run` to queue manual module runs.
- Added `PATCH /module-runs/{run_id}` to update module run status and summary.
- Added audit logging for module run trigger/update actions.
- Added automated API test for trigger -> update -> filtered list flow.
- Added `scripts/scaffold_step11.ps1`.
- Bumped OpenAPI contract version to `1.8.0`.

## 2026-02-11 — Step 10 (Execution quality telemetry)
- Added `execution_quality_ts` persistence via Alembic migration `0007_execution_quality_ts`.
- Added ingest endpoint `POST /ingest/execution-quality`.
- Added analytics endpoint `GET /analytics/execution-quality` returning average slippage/latency/fill-ratio and sample count.
- Added automated API test for execution quality ingest and summary aggregation.
- Added `scripts/scaffold_step10.ps1`.
- Bumped OpenAPI contract version to `1.7.0`.

## 2026-02-11 — Step 9 (NetPnL extension)
- Added `cost_ledger` persistence via Alembic migration `0006_cost_ledger`.
- Added ingest endpoint `POST /ingest/costs` for `FEE`/`FUNDING` cost entries.
- Added analytics endpoint `GET /analytics/net-pnl` implementing MVP formula:
  - `NetPnL = realized + unrealized - fees + funding`
- Added automated API test for NetPnL formula validation with symbol filtering.
- Added `scripts/scaffold_step9.ps1`.
- Bumped OpenAPI contract version to `1.6.0`.

## 2026-02-11 — Step 8 (Reconciliation persistence + mismatch alerts)
- Added `reconcile_results` persistence via Alembic migration `0005_reconcile_results`.
- Upgraded `POST /reconcile` to typed/persistent behavior with instance validation.
- Added `GET /reconcile/results` with pagination and filters (`instance_id`, `status`).
- Added mismatch alert routing: `MISMATCH` reconciliation creates WARNING alert and triggers outbound webhook notification when configured.
- Added automated API test for reconciliation persistence/filtering and mismatch notification behavior.
- Added `scripts/scaffold_step8.ps1`.
- Bumped OpenAPI contract version to `1.5.0`.

## 2026-02-11 — Step 7 (Metrics/positions/exposure foundation)
- Added metrics and positions persistence via Alembic migration `0004_metrics_positions`:
  - `metrics_ts`
  - `positions_current`
- Added ingestion endpoints:
  - `POST /ingest/metrics`
  - `POST /ingest/positions`
- Added exposure summary endpoint:
  - `GET /portfolio/exposure`
  - returns total net/gross exposure, per-symbol breakdown, and key metrics
- Updated Portfolio UI page to poll and render exposure summary.
- Added Next API proxy route for portfolio exposure.
- Added API tests for metrics/positions ingestion + exposure aggregation behavior.
- Added `scripts/scaffold_step7.ps1`.
- Bumped OpenAPI contract version to `1.4.0`.

## 2026-02-11 — Step 6 (Notification Router + Discord webhook)
- Added notification router in dashboard API with severity skeleton:
  - INFO
  - WARNING
  - CRITICAL
  - AUDIT
- Added `alerts` persistence model and Alembic migration `0003_alerts`.
- Implemented `POST /alerts/heartbeat-check` to detect stale heartbeats and create CRITICAL alerts.
- Added Discord webhook send path for routed alerts (outbound only).
- Added test coverage with mocked webhook for heartbeat-loss alerting and dedup behavior.
- Added `scripts/scaffold_step6.ps1`.
- Updated OpenAPI contract to `1.3.0`.

## 2026-02-11 — Step 5 (Command E2E + Audit)
- Added persistent command tables via Alembic migration `0002_commands_and_acks`:
  - `commands`
  - `command_acks`
- Implemented command endpoints in dashboard API:
  - `POST /commands` (admin issues command)
  - `GET /instances/{instance_id}/commands/pending` (agent pull)
  - `POST /commands/{command_id}/ack` (agent ack)
- Added audit persistence for command create + ack + module operations.
- Updated SDK runner to default command pull URL to `/instances/{instance_id}/commands/pending` when unset.
- Added API tests for command end-to-end flow, audit log persistence, and expired-rejection ack path.
- Added `scripts/scaffold_step5.ps1`.
- Bumped OpenAPI contract version to `1.2.0` and added pending command path.

## 2026-02-11 — Step 4 (Python Agent SDK MVP)
- Added Python agent SDK under `sdk/python`.
- Implemented core SDK components:
  - `Command` + `CommandAck` models
  - `CommandProcessor` with idempotency (`command_id`) + TTL (`expires_at`) handling
  - `DeadmanSwitch` fallback logic (`SAFE_MODE` default, `FLATTEN` optional)
  - control-plane HTTP client for heartbeat + ACK
  - command source abstractions (HTTP pull and local file queue)
  - runtime loop and executable `run_agent.py`
- Added SDK test suite covering:
  - expired command rejection
  - duplicate command rejection
  - dead-man switch timeout behavior
  - runtime heartbeat + ACK processing
- Added `scripts/scaffold_step4.ps1`.
- Updated root npm scripts and CI to run SDK tests.

## 2026-02-11 — Step 3 (Frontend skeleton + bots polling)
- Added Next.js TypeScript frontend scaffold under `apps/web`.
- Implemented stable navigation shell and skeleton routes:
  - `/dashboard`
  - `/bots`
  - `/portfolio`
  - `/markets`
  - `/analytics`
  - `/datasets`
  - `/admin/modules`
- Implemented Bots page polling every 5 seconds with table columns: status, last_seen, version.
- Added Next API routes:
  - `GET /api/bots` proxying to dashboard-api with `X-Admin-Token` from env
  - `GET /api/healthz`
- Updated Docker Compose web service to run Next.js dev server.
- Added web npm scripts (`web:dev`, `web:build`) in root `package.json`.

## 2026-02-11 — Step 2 (Backend core + auth MVP + health)
- Added FastAPI backend service scaffold in `services/dashboard-api`.
- Added SQLAlchemy models and initial Alembic migration for:
  - `bots`
  - `instances`
  - `bot_status`
  - `audit_logs`
  - `modules`
  - `module_runs`
- Implemented endpoints:
  - `GET /healthz`
  - `POST /ingest/heartbeat` (upsert)
  - `GET /bots` (paginated, admin token)
  - module registry CRUD (`GET/POST/GET:id/DELETE /modules`, admin token)
- Added pytest coverage for health, heartbeat upsert, auth checks, and module CRUD.
- Updated docker compose to run API service with migrations and uvicorn.
- Updated scripts and npm commands for migration and API tests.
- Updated CI to run backend tests in addition to contracts.

## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
- Added OpenAPI contract at `contracts/openapi/control-plane.v1.yaml`.
- Added JSON Schemas:
  - `heartbeat.schema.json`
  - `command.schema.json`
  - `ack.schema.json`
  - `reconcile.schema.json`
  - `audit.schema.json`
  - `module.schema.json`
  - `module_run.schema.json`
- Added contract validation scripts:
  - `scripts/validate_contracts.ps1`
  - `scripts/validate_json_schemas.py`
- Updated npm scripts to include `contracts:lint` and made `test` run contract checks.
- Updated CI workflow to enforce contract linting/validation.
- Updated README with contract SSOT and validation commands.

## 2026-02-11 — Step 0 (Project Initialization)
- Added required monorepo directory structure for contracts, services, SDKs, apps, infra, scripts, and docs.
- Added Docker Compose baseline with PostgreSQL and placeholders for dashboard API and web app.
- Added `.env.example` with core, auth, database, service port, and webhook placeholders.
- Added PowerShell scripts: `scripts/dev.ps1`, `scripts/test.ps1`, `scripts/migrate.ps1`.
- Added cross-platform npm scripts in `package.json`: `dev`, `test`, `migrate`.
- Added minimal CI skeleton workflow in `.github/workflows/ci.yml`.
- Updated `README.md` with quick start and repository conventions.
