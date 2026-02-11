# Profinaut V2.5+ — Multi-Exchange / Multi-Language Bot Management Dashboard

Step 7 delivers metrics + positions + exposure foundation and portfolio exposure UI.

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
