# Profinaut V2.5+ — Multi-Exchange / Multi-Language Bot Management Dashboard

Step 20 delivers resource telemetry ingest and latest resource analytics on top of indices and module-run analytics extensions.

## What is included
- Contracts SSOT with OpenAPI + JSON Schemas (`contracts/`).
- Backend core service at `services/dashboard-api`.
- Frontend app at `apps/web` with navigation skeleton and bots polling.
- Python Agent SDK MVP at `sdk/python`.
- Command E2E flow + audit persistence.
- Notification router Phase 1 (Discord webhook outbound).
- Metrics/exposure foundation:
  - `POST /ingest/metrics`
  - `POST /ingest/positions`
  - `GET /portfolio/exposure`
  - Portfolio UI polling and rendering exposure summary
- Reconciliation persistence + alerting:
  - `POST /reconcile`
  - `GET /reconcile/results`
  - WARNING alert + outbound webhook routing on `MISMATCH`
- NetPnL analytics extension:
  - `POST /ingest/costs`
  - `GET /analytics/net-pnl`
  - Formula: `realized + unrealized - fees + funding`
- Execution quality extension:
  - `POST /ingest/execution-quality`
  - `GET /analytics/execution-quality`
  - Averages: slippage (bps), latency (ms), fill ratio
- Resource telemetry extension:
  - `POST /ingest/resource`
  - `GET /analytics/resource/latest`
  - Latest CPU/memory telemetry summary
- Markets/indices extension:
  - `POST /ingest/indices`
  - `GET /analytics/indices/latest`
  - Latest per-index values with optional filter
- Module execution controls:
  - `POST /modules/{module_id}/run`
  - `PATCH /module-runs/{run_id}`
  - `GET /module-runs` (paginated/filterable)
- Module run ops extension:
  - `POST /module-runs/{run_id}/cancel`
  - `GET /module-runs/stats`
- Module run alerting extension:
  - `POST /alerts/module-runs/stuck-check`
  - WARNING alerts + outbound webhook routing (deduplicated)
- Advanced analytics extension:
  - `GET /analytics/equity-drawdown`
  - max/current drawdown summary from equity time-series
- Module run analytics extension:
  - `GET /analytics/module-runs/performance`
  - completion rate + avg/p95 duration summary
  - `GET /analytics/module-runs/failure-rate`
  - FAILED ratio over recent completed runs (windowed)
  - `GET /analytics/module-runs/throughput`
  - run throughput over recent time window
  - `GET /analytics/module-runs/active-age`
  - active run count + oldest/average active duration

## Quick start (Windows 11 + Docker Desktop)
1. Copy environment file:
   ```powershell
   Copy-Item .env.example .env
   ```
2. Start stack:
   ```powershell
   ./scripts/dev.ps1
   ```
3. Run all checks:
   ```powershell
   ./scripts/test.ps1
   ```

## Cross-platform npm commands
```bash
npm run dev
npm run test
npm run migrate
npm run web:dev
npm run web:build
npm run test:sdk-python
```

## Security baseline
- Dashboard uses replaceable local admin auth header: `X-Admin-Token` from `.env`.
- Frontend calls backend via Next API route with server-side token injection.
- Dashboard never stores exchange API keys.
