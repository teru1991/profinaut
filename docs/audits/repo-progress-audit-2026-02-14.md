# Repo Progress Audit (Planner + Repo Auditor)
Date: 2026-02-14
Scope: repository audit and MRU planning (no runtime behavior changes)

## 1) Status summary (facts with evidence)

### A. What the repo currently contains
- Branch / HEAD: `work` @ `58a731b86101cc558f4e095a1d559e85f4cd85ff`.
- Recent commit stream includes merged PRs #32–#37 and latest additions for market ticker UI and GMO live execution support.
- CI workflow is present (`.github/workflows/ci.yml`) with jobs for contracts lint/validation, dashboard API tests, SDK tests, web build, and baseline script checks.
- Source-of-truth/spec docs exist in:
  - `contracts/openapi/control-plane.v1.yaml` and `contracts/schemas/**/*.json`
  - `README.md` (platform scope and included endpoints)
  - `docs/roadmap.md` (Step 0–21 progression)
  - `docs/changelog.md` (step-by-step merged history)
  - `docs/specs/*.md` (domain specs: bots UI, execution, marketdata UI, simple bot)
- Implemented service/app directories:
  - Control-plane backend: `services/dashboard-api`
  - Execution service: `services/execution`
  - Market data service: `services/marketdata-rs`
  - Web UI: `apps/web`
  - Bot/agent runtime: `bots/simple_mm`, `sdk/python`
- Implemented web pages/routes include:
  - Pages: `/dashboard`, `/bots`, `/portfolio`, `/markets`, `/market`, `/analytics`, `/datasets`, `/admin/modules`
  - Next API routes: `/api/bots`, `/api/healthz`, `/api/portfolio/exposure`, `/api/ticker`
- Implemented backend endpoints (non-exhaustive) include:
  - Health/capabilities: `/healthz`, `/capabilities`
  - Control-plane ingest/analytics/ops: `/ingest/*`, `/analytics/*`, `/portfolio/exposure`, `/bots`, `/modules`, `/module-runs`, `/commands`, `/audit/logs`, `/reconcile`
  - Execution: `/execution/order-intents`, `/execution/orders/{order_id}/cancel`
  - Marketdata: `/ticker/latest`, `/healthz`, `/capabilities`

### B. What is merged vs in-flight vs missing
- Merged (verified by roadmap/changelog/checklist and code presence):
  - Step 0–21 are marked completed in `docs/roadmap.md`.
  - Changelog has concrete entries through Step 21.
  - Latest merged commits include market ticker page/proxy and GMO live execution support.
- In-flight / partial:
  - Roadmap Step 22+ is explicitly open.
  - UI audit flags follow-up MRUs T191/T192/T193 as recommendations (bots field alignment, read-only kill switch panel, unified degraded UX).
- Missing / unverifiable:
  - No evidence for a task ID `W2-030` in repository docs/specs/commits (string not found).
  - No local metadata for open PR list from remote provider; only local branch `work` is available.

### C. Blockers / risks
- Contracts-first risk:
  - UI audit notes potential mismatch if `/bots` payload from backend does not expose all required degraded fields expected by UI spec.
- Parallel safety risk:
  - Proposed UI MRUs T191/T192/T193 all target `apps/web/**`; these are **not parallel-safe** unless file-level scopes are disjoint.
- CI observability risk:
  - Local repo contains CI config but no local artifact/log history for recent remote CI failures.
- Environment/testing limitation observed:
  - `scripts/validate_json_schemas.py` failed locally due missing `jsonschema` package in current runtime.

## 2) Concise system spec (derived from repo)

- Overall goal:
  - Contracts-first multi-exchange bot management / trading control plane with safety and observability as defaults.
- High-level architecture:
  1. Control Plane API + Dashboard UI (`services/dashboard-api`, `apps/web`)
  2. Market Data service (`services/marketdata-rs`) and UI ticker monitor (`/market`)
  3. Execution service (`services/execution`) for paper + gated GMO live path
  4. Strategy/runtime side (`bots/simple_mm`, `sdk/python`) with heartbeat/command/dead-man handling
  5. Portfolio + analytics + event/audit logs (`/portfolio/exposure`, `/analytics/*`, `/audit/logs`)
- Key invariants:
  - Contracts are SSOT and validated in CI.
  - Control plane/dashboard do not store exchange API keys.
  - Dead-man / safe defaults block unsafe actions when degraded/unreachable.
  - Capabilities + degraded signaling are explicit (`status`, `degraded_reason`), not silent success.

## 3) Verified MRU roadmap and current progress

### MRU catalog (extracted from roadmap + UI audit)
| MRU ID | Description | Status | Evidence |
|---|---|---|---|
| Step 0–21 | Core platform phases through resource window analytics | DONE | `docs/roadmap.md` checked items + `docs/changelog.md` Step 21 entry + implemented endpoints/code |
| Step 22+ | Additional module expansion/analytics | NOT STARTED (open umbrella) | `docs/roadmap.md` unchecked Step 22+ |
| T191 | UI bots fields align (`state/degraded/degraded_reason/last_seen`) | IN PROGRESS (planned, not merged as dedicated MRU) | `docs/audits/ui-current-vs-spec.md` recommended follow-up; no dedicated task artifact found |
| T192 | UI kill-switch read-only panel | IN PROGRESS (planned, not merged as dedicated MRU) | `docs/audits/ui-current-vs-spec.md` recommended follow-up |
| T193 | Unified degraded banner/chip | NOT STARTED (optional) | `docs/audits/ui-current-vs-spec.md` marks optional; no dedicated task artifact found |
| W2-030 | (claim verification request) | REFUTED/UNVERIFIED | `W2-030` string not found in repository |

### Next MRUs
#### 1) Must-do next (sequential unblockers)

**Task ID:** T191 — `ui-bots-fields-align`
- Depends-on: none (UI-only alignment)
- Allowed paths:
  - `apps/web/app/bots/page.tsx`
  - `apps/web/components/BotsTable.tsx`
  - `apps/web/app/api/bots/route.ts` (only if response normalization needed)
  - `docs/specs/ui-bots.md` (acceptance update only)
- Forbidden paths:
  - `.github/workflows/ci.yml`, `docker-compose.yml`, lockfiles, root `package.json`
  - `contracts/**` (unless dedicated contracts PR)
- Deliverables:
  - Bots UI renders `state`, `degraded`, `degraded_reason`, `last_seen` per spec.
  - Explicit stale/degraded visual treatment.
- Acceptance criteria:
  - `/bots` page visibly shows degraded state and reason when present.
  - No command/kill actions introduced.
  - Web build passes.
- Minimal test:
  - `npm --prefix apps/web run build`

#### 2) Parallel-safe tasks (disjoint files)

**Task ID:** T192 — `ui-killswitch-readonly`
- Depends-on: none (can run parallel with T191 only if files do not overlap)
- Allowed paths:
  - `apps/web/app/dashboard/page.tsx` (or a dedicated readonly panel file under `apps/web/components/`)
  - `apps/web/app/api/healthz/route.ts` and/or new `apps/web/app/api/kill-switch/route.ts` (GET-only)
  - `docs/specs/ui-bots.md` or dedicated UI spec note
- Forbidden paths:
  - Same global forbidden set; additionally avoid files reserved by T191 when parallelized.
- Deliverables:
  - Read-only kill-switch state panel with clear disabled-action posture.
- Acceptance criteria:
  - UI shows kill-switch status and reason message.
  - No POST/action wiring.
- Minimal test:
  - `npm --prefix apps/web run build`

**Task ID:** T060 — `execution-observability-tighten` (new safety MRU)
- Depends-on: none
- Allowed paths:
  - `services/execution/app/main.py`
  - `services/execution/tests/test_main.py`
  - `docs/context/notes/execution-gmo.md` (observability notes)
- Forbidden paths:
  - contracts/workflows/docker/root package/lockfiles
- Deliverables:
  - Structured logs for live-degraded transitions and recovery path verification.
- Acceptance criteria:
  - On 429/timeout, explicit degraded log emitted once per transition.
  - On backoff expiry/recovery, explicit recovery log emitted.
  - Tests cover both transitions.
- Minimal test:
  - `PYTHONPATH=services/execution pytest -q services/execution/tests`

#### 3) Deferred tasks

**Task ID:** T193 — `ui-degraded-banner-unify`
- Depends-on: T191
- Allowed paths:
  - `apps/web/components/**` (shared degraded banner/chip)
  - Relevant route files consuming the component
- Forbidden paths:
  - global forbidden set + avoid T192-owned files if parallel
- Deliverables:
  - Shared degraded component used in bots/list or future detail.
- Acceptance criteria:
  - Consistent style/copy for degraded states.
- Minimal test:
  - `npm --prefix apps/web run build`

## 4) Codex-ready prompts for next tasks

### Prompt for T191 (ui-bots-fields-align)
```text
You are implementing MRU task T191: ui-bots-fields-align.

Scope:
- Align /bots UI rendering to include state, degraded, degraded_reason, and last_seen.

Allowed paths (strict):
- apps/web/app/bots/page.tsx
- apps/web/components/BotsTable.tsx
- apps/web/app/api/bots/route.ts (only if response normalization is required)
- docs/specs/ui-bots.md (small acceptance updates only)

Forbidden paths:
- .github/workflows/ci.yml, docker-compose.yml, lockfiles, root package.json
- contracts/**

Plan:
1) Inspect current Bot row typing and normalize API fields to spec names.
2) Add explicit degraded badge/reason rendering in bots table.
3) Add stale last_seen handling (clear visual indicator).
4) Keep page read-only; no command or kill-switch actions.
5) Update docs/spec acceptance checklist minimally if behavior changes.

Verification:
- Run: npm --prefix apps/web run build
- Confirm /bots renders fields and degraded states without runtime errors.

Safety/observability requirements:
- Do not log secrets.
- Do not silently ignore malformed API rows; show clear fallback text.
- Keep behavior safe-by-default (read-only, non-destructive).
```

### Prompt for T192 (ui-killswitch-readonly)
```text
You are implementing MRU task T192: ui-killswitch-readonly.

Scope:
- Add read-only kill-switch visibility in dashboard web UI.

Allowed paths (strict):
- apps/web/app/dashboard/page.tsx (or a new panel component under apps/web/components)
- apps/web/app/api/kill-switch/route.ts (GET only) and related minimal API proxy wiring
- docs/specs/ui-bots.md or docs/specs/controlplane-bots.md (small read-only UI note)

Forbidden paths:
- .github/workflows/ci.yml, docker-compose.yml, lockfiles, root package.json
- contracts/**
- Any files modified by T191 if tasks run in parallel

Plan:
1) Add GET-only proxy route for kill-switch state from control-plane API.
2) Render a read-only panel showing state/message/updated_at.
3) Explicitly display that actions are disabled in this phase.
4) Ensure no POST wiring exists.

Verification:
- Run: npm --prefix apps/web run build
- Validate UI compiles and shows read-only status panel.

Safety/observability requirements:
- No control actions from UI.
- Clear degraded/error text on proxy failure.
- Never expose secrets in client logs.
```

### Prompt for T060 (execution-observability-tighten)
```text
You are implementing MRU task T060: execution-observability-tighten.

Scope:
- Improve explicit degraded/recovery logs for GMO live execution backoff transitions.

Allowed paths (strict):
- services/execution/app/main.py
- services/execution/tests/test_main.py
- docs/context/notes/execution-gmo.md (observability section updates)

Forbidden paths:
- .github/workflows/ci.yml, docker-compose.yml, lockfiles, root package.json
- contracts/**

Plan:
1) Identify current degrade marker and backoff state transition points.
2) Emit structured log on first transition into degraded mode.
3) Emit structured log when recovery happens (backoff elapsed and request allowed again).
4) Add tests for both log transition behaviors.

Verification:
- Run: PYTHONPATH=services/execution pytest -q services/execution/tests

Safety/observability requirements:
- Never log API keys/secrets.
- Keep live execution disabled by default.
- On errors, return explicit non-200 status with actionable non-sensitive detail.
```
