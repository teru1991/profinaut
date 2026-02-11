# Profinaut V2.5+ — Multi-Exchange / Multi-Language Bot Management Dashboard

Step 5 delivers command system end-to-end flow and audit persistence.

## What is included
- Contracts SSOT with OpenAPI + JSON Schemas (`contracts/`).
- Backend core service at `services/dashboard-api`.
- Frontend app at `apps/web` with navigation skeleton and bots polling.
- Python Agent SDK MVP at `sdk/python`.
- Command E2E flow:
  - admin creates command in API (`POST /commands`)
  - agent pulls pending command (`GET /instances/{instance_id}/commands/pending`)
  - agent sends ack (`POST /commands/{command_id}/ack`)
  - backend persists command + ack and writes audit logs.

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
