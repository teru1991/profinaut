# UI Audit: Current Dashboard vs Spec (Bots Status / Degraded / Kill Switch)
Date: 2026-02-13
Scope: ui-audit-no-change (docs only)
Code changes: NONE

## 1. Executive Summary (Decision-Ready)
- Can current UI be used as-is? **Partial**
- Must-fix items (minimum):
  1) Add required bot-state fields (`state`, `degraded`, `degraded_reason`) to both API payload usage and UI rendering.
  2) Add explicit degraded-state UX (banner/badge + stale heartbeat policy handling).
  3) Keep kill-switch interactions read-only in this phase (no command/kill actions from UI).
- Nice-to-have (defer):
  - Standardize `last_seen` formatting (absolute UTC + relative time) for faster operator triage.
  - Add bot detail route (`/bots/:id`) once fields are aligned.
- Recommended next MRU tasks (IDs + scope proposals):
  - **T191 (ui-bots-fields-align):** Align `/bots` table + types to include `state/degraded/degraded_reason/last_seen` and consistent enum mapping.
  - **T192 (ui-killswitch-readonly):** Add kill-switch display panel with read-only state, no action wiring.
  - **T193 (ui-degraded-banner-unify)** (if needed): Introduce shared degraded banner/chip component for list/detail surfaces.

## 2. Inventory: Pages / Routes / Data Sources
| Area | Route | Component/Page | Data Source (API / mock) | Polling/WS | Notes |
|---|---|---|---|---|---|
| Bots | `/bots` | `app/bots/page.tsx` -> `components/BotsTable.tsx` | `GET /api/bots` (Next route proxy to dashboard API `/bots`) | Polling 5s | Only implemented bot ops surface today. |
| Bot detail | `/bots/:id` | Not implemented | N/A | N/A | No detail route/page/component found in web app. |
| Runs | N/A | Not implemented in UI | Backend has module run endpoints, but no web route | N/A | No `app/runs` route in `apps/web/app`. |
| Commands | N/A | Not implemented in UI | Backend has `/commands` endpoints, but no web route | N/A | No command controls currently exposed in web UI. |
| Kill switch | N/A | Not implemented in UI | Backend kill-switch path not proxied in current web app | N/A | No kill-switch panel/button currently present in `apps/web`. |

## 3. Spec Targets (What we require)
### 3.1 Bot status fields (minimum)
Required fields:
- `bot_id`
- `state` (enum)
- `degraded` (bool)
- `degraded_reason` (string?)
- `last_seen` (timestamp)

Optional:
- capabilities
- version/build
- tags/labels

### 3.2 Degraded display rules (minimum)
- If `degraded=true` => UI must show clear degraded banner/status.
- If `last_seen` stale => treat as degraded (or "unknown") depending on policy.
- No destructive controls shown/used when degraded (read-only ok).

### 3.3 Kill switch (read-only in UI for now)
- UI may display kill switch state.
- UI must NOT send kill/execute commands in this phase.

## 4. Gap Table: Current vs Spec
| Spec Item | Current UI behavior | Evidence (route/component/API) | Gap | Severity | Minimal fix idea | Candidate task |
|---|---|---|---|---|---|---|
| `state` shown | Shows `status` text in badge; no explicit `state` enum field | `/bots` -> `BotsTable.tsx` renders `row.status` | Spec name/semantics mismatch and no normalized mapping | High | Add `state` field in response typing + enum-to-label mapping | T191 |
| degraded banner/status | No degraded indicator or banner in list/detail | `BotsTable.tsx` has no degraded fields/UI; no detail route exists | Not implemented | High | Add `degraded` boolean UI chip/banner and shared presentation | T193/T191 |
| `degraded_reason` | Not rendered anywhere | `BotRow` type lacks this field; table has no column/tooltip | Missing operator context | Medium | Add optional reason column/tooltip in bots list + detail | T191 |
| `last_seen` policy | Displays raw `last_seen` string only | `BotsTable.tsx` column "Last Seen (UTC)" prints value directly | No stale-policy classification and no formatting standard | Medium | Apply stale-threshold rule and dual format (relative + UTC) | T191 |
| kill switch read-only | No kill-switch UI currently | No `/kill-switch` route/component in `apps/web`; no related nav entry | Spec item absent (safe-by-absence, but not visible) | Medium | Add read-only kill-switch status panel; keep actions disabled/unavailable | T192 |
| no command/kill actions | UI currently does not expose command execution controls | No commands route/components in `apps/web`; only bots/portfolio APIs proxied | Meets current phase constraint | Low | Preserve no-action posture while adding read-only observability | T192 |

## 5. Proposed Minimal Follow-up Plan (MRU)
### T191 ui-bots-fields-align (apps/web/** only)
- Goal: Align bots list (+ future detail) fields to spec.
- Minimal acceptance:
  - `state/degraded/degraded_reason/last_seen` visible in bots UI.
  - Degraded display consistent for list cells and summary states.
  - Bot row typing updated to contract fields and enum mapping documented.
- Non-goals:
  - No backend changes.
  - No control actions.

### T192 ui-killswitch-readonly (apps/web/** only)
- Goal: Ensure kill switch panel is read-only.
- Minimal acceptance:
  - UI displays kill switch state/message.
  - No POST/command action is triggered from UI.
  - Any future controls remain disabled with explanatory text.

### T193 ui-degraded-banner-unify (apps/web/** only) (optional)
- Goal: Unified degraded banner/component for degraded state.
- Minimal acceptance:
  - Shared component used wherever degraded is represented.
  - Consistent copy/style for degraded + stale heartbeat.

## 6. Notes / Risks
- Spec interpretation risk:
  - Current implementation uses `status` naming; spec requires `state`. Need a clear mapping contract to avoid drift.
- Potential blocker (backend/contracts):
  - Current `/bots` response shape consumed by web does not include `degraded`/`degraded_reason`; if backend does not emit these yet, UI alignment depends on contract/API update.
- Current safety posture:
  - No command or kill-switch actions are wired in the UI today, which is compatible with read-only phase requirements.
