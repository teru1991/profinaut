# PR Features Report (AUD-PR-010)

## Scope
- Task: `audit-pr-features`
- Goal: enumerate implemented features from merged PRs on the default branch.

## Default branch determination (required evidence)
```bash
$ git branch --list master main
  master

$ git log --oneline -5 master
74b8f8f (master) Merge pull request #55 from teru1991/copilot/consolidate-routes-markets
f5f1a92 Initial plan
3c32f35 修正
564a686 Merge pull request #54 from teru1991/codex/verify-current-repo-progress-and-specifications-ee5h5e
74ff4e5 Merge branch 'master' into codex/verify-current-repo-progress-and-specifications-ee5h5e
```

Default branch used for this audit: **master**.

## Implemented features from merged PRs (with git evidence)

### PR #1 — Scaffold monorepo baseline
- Merge commit: `f901b88`
- PR head commit: `c295c1c`
- Feature implemented: initial monorepo scaffolding (API/worker/contracts/tests/docker baseline).
- Evidence (changed paths):
  - `.github/workflows/ci.yml`
  - `Dockerfile.api`
  - `Dockerfile.worker`
  - `alembic/versions/001_initial_schema.py`
  - `contracts/schemas.py`
  - `dashboard_api/main.py`
  - `tests/test_api.py`
  - `worker/main.py`

### PR #20 — Initialize project structure and control-plane API baseline
- Merge commit: `1220c68`
- PR head commit: `c4237b3`
- Feature implemented: expanded repository structure and initial control-plane/dashboard-api shape.
- Evidence (changed paths):
  - `contracts/openapi/control-plane.v1.yaml`
  - `services/dashboard-api/app/main.py`
  - `services/dashboard-api/app/schemas.py`
  - `services/dashboard-api/tests/test_api.py`
  - `docs/changelog.md`

### PR #29 — Add execution contracts
- Merge commit: `41662f8`
- PR head commit: `f39607e`
- Feature implemented: execution order/fill contract schemas and OpenAPI integration.
- Evidence (changed paths):
  - `contracts/openapi/control-plane.v1.yaml`
  - `contracts/schemas/execution/order_intent.schema.json`
  - `contracts/schemas/execution/order.schema.json`
  - `contracts/schemas/execution/fill.schema.json`

### PR #30 — Implement paper execution service
- Merge commit: `bb258d9`
- PR head commit: `f577180`
- Feature implemented: paper execution service with API, storage, and tests.
- Evidence (changed paths):
  - `services/execution/app/main.py`
  - `services/execution/app/schemas.py`
  - `services/execution/app/storage.py`
  - `services/execution/tests/test_api.py`

### PR #31 — Create simple market-making bot
- Merge commit: `747acd1`
- PR head commit: `95e5420`
- Feature implemented: `simple_mm` bot implementation and documentation.
- Evidence (changed paths):
  - `bots/simple_mm/main.py`
  - `bots/simple_mm/test_main.py`
  - `docs/specs/simple-bot.md`

### PR #32 — Stabilize bots endpoint
- Merge commit: `66b713f`
- PR head commit: `1123d06`
- Feature implemented: improved control-plane bots endpoint behavior and models.
- Evidence (changed paths):
  - `services/dashboard-api/app/main.py`
  - `services/dashboard-api/app/models.py`
  - `services/dashboard-api/tests/test_api.py`

### PR #33 — Harden bots status page UX
- Merge commit: `1ffd987`
- PR head commit: `f03196c`
- Feature implemented: defensive status rendering on bots UI.
- Evidence (changed paths):
  - `apps/web/components/BotsTable.tsx`
  - `docs/specs/ui-bots.md`

### PR #34 — Refine paper execution service
- Merge commit: `a765bd5`
- PR head commit: `2ce924b`
- Feature implemented: execution service schema/API/test hardening.
- Evidence (changed paths):
  - `services/execution/app/main.py`
  - `services/execution/app/schemas.py`
  - `services/execution/tests/test_api.py`
  - `services/execution/tests/test_logging.py`

### PR #35 — Create simple E2E paper bot flow
- Merge commit: `e4803f4`
- PR head commit: `8a9e438`
- Feature implemented: E2E flow logic for `simple_mm` paper trading.
- Evidence (changed paths):
  - `bots/simple_mm/main.py`
  - `bots/simple_mm/test_main.py`
  - `docs/specs/simple-bot.md`

### PR #36 — Add market data ticker page
- Merge commit: `58a731b`
- PR head commit: `b914d91`
- Feature implemented: web market ticker page and API proxy route.
- Evidence (changed paths):
  - `apps/web/app/market/page.tsx`
  - `apps/web/app/market/TickerCard.tsx`
  - `apps/web/app/api/ticker/route.ts`

### PR #37 — Add GMO live execution support
- Merge commit: `e8a5856`
- PR head commit: `8e0034f`
- Feature implemented: live execution integration for GMO.
- Evidence (changed paths):
  - `services/execution/app/live.py`
  - `services/execution/app/main.py`
  - `services/execution/app/config.py`
  - `services/execution/tests/test_api.py`

### PR #38 — Implement simple E2E bot safeguards/observability
- Merge commit: `003b1df`
- PR head commit: `aea7507`
- Feature implemented: safer order gating and improved bot observability.
- Evidence (changed paths):
  - `bots/simple_mm/main.py`
  - `bots/simple_mm/test_main.py`
  - `docs/specs/simple-bot.md`

### PR #47 — Add markets page and navigation path
- Merge commit: `c576968`
- PR head commit: `1c46015`
- Feature implemented: markets page plus ticker latest proxy route.
- Evidence (changed paths):
  - `apps/web/app/markets/page.tsx`
  - `apps/web/app/api/markets/ticker/latest/route.ts`

### PR #49 — Add API status summary endpoint
- Merge commit: `fd834b1`
- PR head commit: `9c1060f`
- Feature implemented: status summary in dashboard API.
- Evidence (changed paths):
  - `services/dashboard-api/app/main.py`
  - `services/dashboard-api/app/config.py`
  - `services/dashboard-api/app/schemas.py`
  - `services/dashboard-api/tests/test_api.py`

### PR #51 — Add status ribbon widget in web UI
- Merge commit: `6816080`
- PR head commit: `573c649`
- Feature implemented: status ribbon component and summary polling route.
- Evidence (changed paths):
  - `apps/web/components/StatusRibbon.tsx`
  - `apps/web/app/api/status/summary/route.ts`
  - `apps/web/app/layout.tsx`
  - `apps/web/app/globals.css`

## Notes
- Evidence was collected from local git history on `master` using merge commit inspection and changed-path diffs between merge-base and PR head commits.
- Merge PRs that appear operational/non-feature (e.g., `verify-current...`, `precheck...`) are intentionally omitted from the implemented-feature list.
