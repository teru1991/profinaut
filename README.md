# Profinaut V2.5+ — Multi-Exchange / Multi-Language Bot Management Dashboard

Step 6 delivers notification routing (severity skeleton) and heartbeat-loss CRITICAL alerts with outbound Discord webhook notifications.

## What is included
- Contracts SSOT with OpenAPI + JSON Schemas (`contracts/`).
- Backend core service at `services/dashboard-api`.
- Frontend app at `apps/web` with navigation skeleton and bots polling.
- Python Agent SDK MVP at `sdk/python`.
- Command E2E flow + audit persistence.
- Notification router Phase 1:
  - severity routing skeleton: INFO/WARNING/CRITICAL/AUDIT
  - heartbeat-loss checker endpoint: `POST /alerts/heartbeat-check`
  - outbound Discord webhook notifications (mock-tested)

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
