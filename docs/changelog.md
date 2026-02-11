# Changelog

## 2026-02-11 — Step 5 (Command E2E + Audit)
- Added persistent command tables via Alembic migration `0002_commands_and_acks`:
  - `commands`
  - `command_acks`
- Implemented command endpoints in dashboard API:
  - `POST /commands` (admin issues command)
  - `GET /commands/pending/{instance_id}` (agent pull)
  - `POST /commands/{command_id}/ack` (agent ack)
- Added audit persistence for command create + ack + module operations.
- Updated SDK runner to default command pull URL to `/commands/pending/{instance_id}` when unset.
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
