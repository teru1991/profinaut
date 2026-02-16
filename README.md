# Profinaut V2.5+ — Multi-Exchange / Multi-Language Bot Management Dashboard

## Overview
Profinaut is a contracts-first bot management platform focused on safety, auditability, and extensibility for multi-exchange operations.

Current delivered baseline is tracked in:
- Roadmap (step progress): [`docs/roadmap.md`](docs/roadmap.md)
- Changelog (chronological releases): [`docs/changelog.md`](docs/changelog.md)

## Repo layout
- `contracts/` — OpenAPI and JSON Schema source-of-truth.
- `services/` — backend services.
- `apps/` — frontend applications.
- `sdk/` — agent SDKs.
- `docs/` — specs, runbooks, status, and workplans.
- `scripts/` — local dev/test helper scripts.

## Quick start
### Windows 11 + Docker Desktop
1. Copy environment file:
   ```powershell
   Copy-Item .env.example .env
   ```
2. Start local stack:
   ```powershell
   ./scripts/dev.ps1
   ```
3. Run checks:
   ```powershell
   ./scripts/test.ps1
   ```

### Cross-platform npm shortcuts
```bash
npm run dev
npm run test
npm run migrate
npm run web:dev
npm run web:build
npm run test:sdk-python
```

## Key docs
- Docs index: [`docs/README.md`](docs/README.md)
- Roadmap: [`docs/roadmap.md`](docs/roadmap.md)
- Changelog: [`docs/changelog.md`](docs/changelog.md)
- Ultimate Gold progress: [`docs/status/ultimate-gold-progress-check.md`](docs/status/ultimate-gold-progress-check.md)
- Ultimate Gold feature catalog: [`docs/workplan/ultimate-gold-implementation-feature-list.md`](docs/workplan/ultimate-gold-implementation-feature-list.md)
- Parallel task safety spec: [`docs/specs/parallel-task-safety.md`](docs/specs/parallel-task-safety.md)

## Development rules
- Follow one-scope-per-PR and dependency-safe delivery.
- Keep contracts additive-only when contract changes are required.
- Prefer documentation hubs and links over duplicated narrative blocks.

See: [`docs/specs/parallel-task-safety.md`](docs/specs/parallel-task-safety.md)

## Security
- Admin authentication uses `X-Admin-Token` from `.env`.
- Frontend accesses backend through server-side token injection route.
- Dashboard never stores exchange API keys.
