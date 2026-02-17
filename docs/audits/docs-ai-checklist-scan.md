# DOC-AI-CHECKLIST Scan

## Command: git status --short
```
?? docs/audits/docs-ai-checklist-scan.md
?? services/marketdata-rs/Cargo.lock
```

## Command: git ls-tree -r --name-only HEAD docs
```
docs/README.md
docs/SSOT/AI_PROMPTS.md
docs/SSOT/README_AI.md
docs/SSOT/TECH_CONTEXT.md
docs/assumptions.md
docs/audits/docs-ai-pack-decision.md
docs/audits/docs-ai-pack-scan.md
docs/audits/docs-ai-prompts-decision.md
docs/audits/docs-ai-prompts-scan.md
docs/audits/docs-audit-report.md
docs/audits/docs-content-overlap.md
docs/audits/docs-os-consolidation-decision.md
docs/audits/docs-os-existing-file-scan.md
docs/audits/docs-rules-unify-decision.md
docs/audits/docs-rules-unify-scan.md
docs/audits/repo-progress-audit-2026-02-14.md
docs/audits/ui-current-vs-spec.md
docs/changelog.md
docs/decisions/decisions.md
docs/handoff/HANDOFF.json
docs/roadmap.md
docs/rules/parallel-development-safety.md
docs/rules/task-generation-policy.md
docs/runbooks/e2e-smoke-runbook.md
docs/runbooks/marketdata-local.md
docs/runbooks/marketdata-replay.md
docs/runbooks/paper_e2e.md
docs/runbooks/reconcile-mismatch-repair.md
docs/runbooks/supply-chain-security.md
docs/specs/controlplane-bots.md
docs/specs/dangerous-ops-confirmation.md
docs/specs/dangerous-ops-taxonomy.md
docs/specs/execution-gmo.md
docs/specs/execution.md
docs/specs/parallel-task-safety.md
docs/specs/simple-bot.md
docs/specs/ui-bots.md
docs/specs/ui-marketdata.md
docs/status/CURRENT_STATUS.md
docs/status/HANDOFF.json
docs/status/decisions.md
docs/status/progress-updates/UG-P0-101.md
docs/status/progress-updates/UG-P0-102.md
docs/status/progress-updates/UG-P0-103.md
docs/status/progress-updates/UG-P0-104.md
docs/status/progress-updates/UG-P0-105.md
docs/status/progress-updates/UG-P0-106.md
docs/status/progress-updates/UG-P0-110.md
docs/status/progress-updates/UG-P0-111.md
docs/status/progress-updates/UG-P0-112.md
docs/status/status.json
docs/status/trace-index.json
docs/status/trace-index.md
docs/status/ultimate-gold-progress-check.md
docs/troubleshooting/bots-502.md
docs/verification/marketdata-data-platform-smoke-results.md
docs/verification/marketdata-data-platform-smoke.md
docs/workplan/ultimate-gold-implementation-feature-list.md
```

## Command: rg -n "checklist|preflight|merge|push|rollback|rebase|conflict|LOCK:|Required Locks|Allowed paths|Forbidden paths|CI|up-to-date|Handoff|HANDOFF|status\.json|trace-index|decision" docs -S
```
docs/decisions/decisions.md:8:- 2026-02-17: stopping requires `docs/handoff/HANDOFF.json` update / Multi-agent and interrupted work need explicit continuity state / No stop/handoff without handoff state refresh.
docs/workplan/ultimate-gold-implementation-feature-list.md:17:- Allowed paths / Forbidden paths
docs/workplan/ultimate-gold-implementation-feature-list.md:30:| Allowed paths | `<path glob list>` |
docs/workplan/ultimate-gold-implementation-feature-list.md:31:| Forbidden paths | `<path glob list>` |
docs/workplan/ultimate-gold-implementation-feature-list.md:33:| Notes/Links | `<docs/decision/pr>` |
docs/workplan/ultimate-gold-implementation-feature-list.md:46:| Allowed paths | `docs/**`, `services/**/metrics/**`, `services/**/alerts/**`, `.github/workflows/**` |
docs/workplan/ultimate-gold-implementation-feature-list.md:47:| Forbidden paths | `contracts/**`, `migrations/**`, `infra/**` |
docs/workplan/ultimate-gold-implementation-feature-list.md:55:| Scope | `decisionlog-assumptions-ssot-governance` |
docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/workplan/ultimate-gold-implementation-feature-list.md:61:| Forbidden paths | `services/**`, `contracts/**` |
docs/workplan/ultimate-gold-implementation-feature-list.md:62:| DoD | `DecisionLog/Assumptionsの更新基準明文化 + PRテンプレ必須化 + CI lint で未記載を失敗化` |
docs/workplan/ultimate-gold-implementation-feature-list.md:74:| Allowed paths | `contracts/**`, `services/**/audit/**`, `docs/**` |
docs/workplan/ultimate-gold-implementation-feature-list.md:75:| Forbidden paths | `migrations/**`, `infra/**` |
docs/workplan/ultimate-gold-implementation-feature-list.md:88:| Allowed paths | `.github/workflows/**`, `scripts/**`, `docs/**` |
docs/workplan/ultimate-gold-implementation-feature-list.md:89:| Forbidden paths | `services/**`, `contracts/**` |
docs/workplan/ultimate-gold-implementation-feature-list.md:102:| Allowed paths | `tests/**`, `services/**/replay/**`, `.github/workflows/**`, `docs/**` |
docs/workplan/ultimate-gold-implementation-feature-list.md:103:| Forbidden paths | `contracts/**`, `infra/**` |
docs/workplan/ultimate-gold-implementation-feature-list.md:104:| DoD | `同一dataset_refでPolicy/OMS/Portfolio一致をCI検証 + clock drift/sequence異常注入テスト + 失敗時SAFE遷移検証` |
docs/workplan/ultimate-gold-implementation-feature-list.md:124:- **UGF-0-017** 互換性統治（additive-only / schema_version / CI破壊検知）
docs/workplan/ultimate-gold-implementation-feature-list.md:133:- **UGF-A-002** 危険領域ロック（contracts/migrations/lockfile/CI/infra）
docs/workplan/ultimate-gold-implementation-feature-list.md:136:- **UGF-A-005** ブランチ保護（PR+CI+up-to-date+直push禁止）
docs/assumptions.md:14:2. Contract validation in CI uses Redocly CLI (OpenAPI lint) and Python `jsonschema` (schema correctness).
docs/roadmap.md:7:- [x] **Step 0**: Project initialization (layout, compose, scripts, docs, CI skeleton).
docs/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/specs/ui-bots.md:212:- CI/CD workflows
docs/specs/parallel-task-safety.md:8:- `docs/rules/task-generation-policy.md` (task generation / Multi-AI / Credit-out / HANDOFF / status / decisions / trace)
docs/specs/simple-bot.md:10:- Logging: lines include `run_id`, `bot_id`, `state`, `decision`, `idempotency_key` (and `order_id` when available)
docs/status/trace-index.json:15:        "docs/audits/docs-os-consolidation-decision.md",
docs/status/trace-index.json:17:        "docs/status/status.json",
docs/status/trace-index.json:18:        "docs/handoff/HANDOFF.json",
docs/status/trace-index.json:19:        "docs/decisions/decisions.md"
docs/status/ultimate-gold-progress-check.md:34:Recent merged PRs:
docs/status/ultimate-gold-progress-check.md:45:| CI | _unlocked_ | - | - | 現在Open PRなし |
docs/status/ultimate-gold-progress-check.md:55:| UG-A | 基盤・運用 | P0 | In Progress | 60% | CI、契約検証、changelog/roadmap運用、段階的ステップ開発 | 危険領域ロックの制度化、branch保護厳格化の文書固定 |
docs/status/ultimate-gold-progress-check.md:117:3. **UG-I**: 障害注入テスト（429/5xx/timeout/WS断）をCIに追加し、SAFE遷移検証を自動化。
docs/status/ultimate-gold-progress-check.md:132:| 2026-02-15 | Repo snapshot + PR/commit evidence refresh (git remote + merge evidence) | Codex |
docs/status/decisions.md:3:Record decision entries here in append-only format.
docs/status/CURRENT_STATUS.md:3:_Last updated: 1970-01-01T00:00:00Z (from `docs/status/status.json`)_
docs/status/CURRENT_STATUS.md:6:> **SSOT is `docs/status/status.json`**.
docs/status/CURRENT_STATUS.md:11:- Active epic: _not specified in `status.json`_
docs/status/CURRENT_STATUS.md:17:- _No open PRs recorded in `status.json`._
docs/status/CURRENT_STATUS.md:21:- _No locks recorded in `status.json`._
docs/status/CURRENT_STATUS.md:25:- _None explicitly recorded in `status.json`._
docs/status/CURRENT_STATUS.md:29:- Update `docs/status/status.json` when a task owner is assigned.
docs/status/CURRENT_STATUS.md:31:- Add progress evidence under `docs/status/progress-updates/` and trace links in `docs/status/trace-index.md`/`.json` as work proceeds.
docs/status/CURRENT_STATUS.md:36:- `docs/status/status.json` (machine-readable SSOT)
docs/status/progress-updates/UG-P0-111.md:7:- Added safety decision criteria for `SAFE_MODE`, `CLOSE_ONLY`, and `FLATTEN`/`HALT` escalation.
docs/status/progress-updates/UG-P0-111.md:9:- Added canary validation success criteria and explicit rollback procedure when mismatch persists.
docs/status/progress-updates/UG-P0-101.md:8:- `policy_decision.schema.json`
docs/status/progress-updates/UG-P0-101.md:9:  - Added PolicyDecision object schema with required fields: `decision`, `reason_code`.
docs/status/progress-updates/UG-P0-101.md:17:  "policy_decision": {
docs/status/progress-updates/UG-P0-101.md:18:    "decision": "BLOCK",
docs/status/progress-updates/UG-P0-110.md:15:   - Backoff/degraded window (`LIVE_DEGRADED`, decision=`THROTTLE`)
docs/status/progress-updates/UG-P0-110.md:20:- `decision`: `ALLOW | BLOCK | THROTTLE | REDUCE_ONLY | CLOSE_ONLY | FLATTEN | HALT`
docs/status/progress-updates/UG-P0-105.md:26:## CI workflow
docs/SSOT/AI_PROMPTS.md:7:- Status SSOT: `docs/status/status.json`
docs/SSOT/AI_PROMPTS.md:8:- Trace SSOT: `docs/status/trace-index.json`
docs/SSOT/AI_PROMPTS.md:16:- Read in strict order: `docs/SSOT/README_AI.md` → `docs/status/status.json` → `docs/handoff/HANDOFF.json` (or `docs/status/HANDOFF.json`) → `docs/status/decisions.md`.
docs/SSOT/AI_PROMPTS.md:17:- Before planning or editing, inspect `docs/status/status.json` keys: `active_task`, `open_prs`, `locks_held`, `owner`, `state`, `last_updated`.
docs/SSOT/AI_PROMPTS.md:18:- If `locks_held` conflicts with intended scope, STOP and return a lock-conflict response.
docs/SSOT/AI_PROMPTS.md:19:- Build a task card that includes: Task ID, Scope, Allowed paths, Forbidden paths, Required LOCKS, Dependencies, Verification steps.
docs/SSOT/AI_PROMPTS.md:20:- Only edit files inside Allowed paths. Do not touch forbidden paths.
docs/SSOT/AI_PROMPTS.md:21:- If pausing/stopping/credit-out before completion, update `docs/handoff/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next` (and sync `docs/status/HANDOFF.json` if that path is being used by current workflow).
docs/SSOT/AI_PROMPTS.md:22:- Any "progress made" claim requires evidence updates in `docs/status/status.json` and/or `docs/status/progress-updates/*`.
docs/SSOT/AI_PROMPTS.md:23:- Record evidence links in `docs/status/trace-index.json` (SSOT) and mirror to markdown index if required.
docs/SSOT/AI_PROMPTS.md:24:- If uncertain, add/update assumptions or decisions via the existing mechanism (`docs/assumptions.md`, `docs/status/decisions.md`) before irreversible changes.
docs/SSOT/AI_PROMPTS.md:32:- Read order: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
docs/SSOT/AI_PROMPTS.md:34:- Include Allowed/Forbidden paths in every task card.
docs/SSOT/AI_PROMPTS.md:36:- Ensure each task has DoD, test/verification commands, and rollback notes.
docs/SSOT/AI_PROMPTS.md:37:- If assumptions are needed, record them in assumptions/decisions flow before execution.
docs/SSOT/AI_PROMPTS.md:38:- Do not claim progress unless `status.json`/progress-updates are updated.
docs/SSOT/AI_PROMPTS.md:42:- Read order: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
docs/SSOT/AI_PROMPTS.md:43:- Implement only within Allowed paths and declared scope.
docs/SSOT/AI_PROMPTS.md:45:- If lock conflict/forbidden touch is needed, stop and return for replanning.
docs/SSOT/AI_PROMPTS.md:47:- On stop/credit-out, update HANDOFF required fields before exit.
docs/SSOT/AI_PROMPTS.md:48:- Any progress statement must be backed by `status.json` or progress update entries.
docs/SSOT/AI_PROMPTS.md:52:- Read order: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
docs/SSOT/AI_PROMPTS.md:55:- If evidence is missing from `status.json`/progress updates/trace index, fail verification.
docs/SSOT/AI_PROMPTS.md:56:- If uncertain behavior exists, request assumptions/decisions update before pass.
docs/SSOT/AI_PROMPTS.md:61:- Read order: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
docs/SSOT/AI_PROMPTS.md:62:- Review PR for SSOT integrity, lock safety, and rollback risk.
docs/SSOT/AI_PROMPTS.md:63:- Reject if LOCK conflicts were ignored or forbidden paths changed.
docs/SSOT/AI_PROMPTS.md:65:- Verify progress claims have concrete evidence updates (`status.json`, progress updates, trace index).
docs/SSOT/AI_PROMPTS.md:66:- Flag decision gaps and require explicit decision log updates when ambiguity remains.
docs/SSOT/AI_PROMPTS.md:74:Read `docs/SSOT/README_AI.md`, then `docs/status/status.json`, `docs/handoff/HANDOFF.json` (or `docs/status/HANDOFF.json`), then `docs/status/decisions.md`; treat `status.json` as SSOT and check `active_task/open_prs/locks_held` before action, declare Allowed/Forbidden paths and Required LOCKS in your task card, stop on LOCK conflict, record trace links in `docs/status/trace-index.json`, and if stopping early update HANDOFF with `what_done/what_next/errors/commands_next`.
docs/SSOT/AI_PROMPTS.md:78:Follow OS read order `README_AI → status.json → HANDOFF → decisions`; use `status.json` as authoritative runtime state (`active_task`, `open_prs`, `locks_held`), operate only in Allowed paths, stop and return on LOCK conflict, require evidence updates for any progress claim (`status.json` or progress updates), write trace evidence to `docs/status/trace-index.json`, and update HANDOFF fields before any pause/credit-out.
docs/SSOT/AI_PROMPTS.md:82:Start by reading `docs/SSOT/README_AI.md`, `docs/status/status.json`, `docs/handoff/HANDOFF.json`/`docs/status/HANDOFF.json`, and `docs/status/decisions.md`; then create a task card with scope + Allowed/Forbidden + LOCKS, execute only safe paths, halt on lock collisions, ensure progress claims are backed by status/progress updates, keep trace links in `docs/status/trace-index.json`, and always write HANDOFF before stopping mid-task.
docs/SSOT/AI_PROMPTS.md:90:- Read: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
docs/SSOT/AI_PROMPTS.md:93:- Stop if LOCK conflict exists.
docs/SSOT/AI_PROMPTS.md:95:- If uncertain, write assumption/decision updates first.
docs/SSOT/AI_PROMPTS.md:96:- Update evidence: `status.json` or `progress-updates` + `trace-index.json`.
docs/SSOT/AI_PROMPTS.md:97:- If pausing, update HANDOFF required keys.
docs/SSOT/AI_PROMPTS.md:102:- Validate active ownership and lock status in `status.json`.
docs/SSOT/AI_PROMPTS.md:105:- Fail fast on lock collisions or missing decision authority.
docs/SSOT/AI_PROMPTS.md:107:- Record trace links in `trace-index.json`.
docs/SSOT/AI_PROMPTS.md:108:- On credit-out, update HANDOFF with actionable next commands.
docs/SSOT/AI_PROMPTS.md:114:- Generate execution checklist with Allowed/Forbidden and LOCK requirements.
docs/SSOT/AI_PROMPTS.md:115:- If lock conflict appears, return STOP + replan message.
docs/SSOT/AI_PROMPTS.md:119:- On interruption, complete HANDOFF fields before ending response.
docs/rules/task-generation-policy.md:10:2. `docs/status/status.json`
docs/rules/task-generation-policy.md:11:3. `docs/status/HANDOFF.json`
docs/rules/task-generation-policy.md:12:4. `docs/status/decisions.md`
docs/rules/task-generation-policy.md:19:- Active task ownership and lifecycle are tracked in `docs/status/status.json`.
docs/rules/task-generation-policy.md:32:- `Required Locks`
docs/rules/task-generation-policy.md:33:- Allowed paths (`ONLY`)
docs/rules/task-generation-policy.md:34:- Forbidden paths (`MUST NOT touch`)
docs/rules/task-generation-policy.md:40:## 4. Multi-AI / Credit-out / Handoff protocol
docs/rules/task-generation-policy.md:45:- Canonical runtime status must be updated through `docs/status/status.json` only.
docs/rules/task-generation-policy.md:46:- Handoff responsibility is explicit: the current owner must leave resumable state.
docs/rules/task-generation-policy.md:52:### 4.3 Mandatory HANDOFF update on stop/switch
docs/rules/task-generation-policy.md:54:Before stopping or switching owner, update `docs/status/HANDOFF.json` with all of:
docs/rules/task-generation-policy.md:66:  - an update to `docs/status/status.json`, **or**
docs/rules/task-generation-policy.md:72:- Use `docs/status/decisions.md` to lock assumptions and decisions that affect ongoing work.
docs/rules/task-generation-policy.md:73:- If work depends on a decision, reference its decision entry ID/date in task notes.
docs/rules/task-generation-policy.md:74:- Do not silently override a recorded decision; record superseding decision explicitly.
docs/rules/task-generation-policy.md:78:- `docs/status/trace-index.md` is the SSOT for trace links (PRs, commits, issues, run artifacts).
docs/rules/task-generation-policy.md:80:- Other docs may include convenience links, but must treat trace-index as canonical.
docs/SSOT/README_AI.md:9:3. Runtime status SSOT: `docs/status/status.json`
docs/SSOT/README_AI.md:12:6. Handoff state: `docs/status/HANDOFF.json` and `docs/handoff/HANDOFF.json`
docs/SSOT/README_AI.md:13:7. Decision baseline: `docs/status/decisions.md`
docs/SSOT/README_AI.md:14:8. Trace SSOT: `docs/status/trace-index.md` and `docs/status/trace-index.json`
docs/SSOT/README_AI.md:23:- **Status SSOT:** `docs/status/status.json`
docs/SSOT/README_AI.md:33:> Read in order: `docs/SSOT/README_AI.md` → `docs/status/status.json` → `docs/handoff/HANDOFF.json` (or `docs/status/HANDOFF.json`) → `docs/status/decisions.md`.
docs/SSOT/README_AI.md:34:> Treat `docs/status/status.json` as SSOT and check `active_task`, `open_prs`, `locks_held` before planning.
docs/SSOT/README_AI.md:35:> If stopping/pause/credit-out before completion, you MUST update `docs/handoff/HANDOFF.json` (and keep `docs/status/HANDOFF.json` aligned if used).
docs/SSOT/README_AI.md:36:> Use `docs/status/trace-index.json` as trace link SSOT.
docs/SSOT/README_AI.md:37:> In task cards always declare Allowed/Forbidden paths and Required LOCKS; if LOCK conflicts exist, stop and return.
docs/SSOT/README_AI.md:41:- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/status/HANDOFF.json` and `docs/handoff/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
docs/SSOT/README_AI.md:42:- **Single active task:** only one active task is tracked in `docs/status/status.json`.
docs/SSOT/README_AI.md:43:- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
docs/SSOT/README_AI.md:44:- **LOCK areas:** lock-sensitive areas include contracts/ci/infra/lockfiles and shared-docs (`docs/SSOT/**`, `docs/rules/**`, docs hubs). Declare `Required Locks` in task cards.
docs/SSOT/README_AI.md:45:- **Trace SSOT:** `docs/status/trace-index.md` / `docs/status/trace-index.json` are the canonical place for PR/commit/evidence links.
docs/SSOT/README_AI.md:46:- **Decisions are binding:** use `docs/status/decisions.md` to fix assumptions and supersede only via explicit new decision entries.
docs/SSOT/TECH_CONTEXT.md:12:- Runtime status SSOT: `docs/status/status.json`
docs/SSOT/TECH_CONTEXT.md:13:- Trace index: `docs/status/trace-index.md` and `docs/status/trace-index.json`
docs/SSOT/TECH_CONTEXT.md:14:- Decisions baseline: `docs/status/decisions.md` (and `docs/decisions/decisions.md` if referenced by task)
docs/SSOT/TECH_CONTEXT.md:15:- Handoff state: `docs/status/HANDOFF.json` (legacy/alt path may exist at `docs/handoff/HANDOFF.json`)
docs/SSOT/TECH_CONTEXT.md:42:- When in doubt, follow linked SSOT/authoritative docs and record decisions in `docs/status/decisions.md`.
docs/rules/parallel-development-safety.md:7:Prevent fatal merge/process accidents during concurrent delivery by enforcing clear scope boundaries and lock declarations.
docs/rules/parallel-development-safety.md:18:- If task B depends on task A, B stays Draft until A merges.
docs/rules/parallel-development-safety.md:45:### 3.3 Required Locks declaration
docs/rules/parallel-development-safety.md:47:Task cards must declare `Required Locks` explicitly. Examples:
docs/rules/parallel-development-safety.md:49:- `Required Locks: docs/SSOT/**`
docs/rules/parallel-development-safety.md:50:- `Required Locks: none` (only when truly lock-free)
docs/rules/parallel-development-safety.md:66:## 6. Quick checklist
docs/rules/parallel-development-safety.md:69:- [ ] Allowed/Forbidden paths respected
docs/rules/parallel-development-safety.md:71:- [ ] Required Locks declared
docs/rules/parallel-development-safety.md:73:- [ ] Revert/rollback path is clear
docs/runbooks/supply-chain-security.md:56:- Re-commit and push the PR branch.
docs/runbooks/supply-chain-security.md:69:If approval is not present, the workflow failure is blocking and must be fixed before merge.
docs/runbooks/e2e-smoke-runbook.md:60:## Live migration checklist (paper -> live)
docs/runbooks/e2e-smoke-runbook.md:74:### 2) Safety checklist before live
docs/runbooks/reconcile-mismatch-repair.md:67:   - Typical causes: parallel stream merge race, inconsistent ordering key/time source.
docs/runbooks/reconcile-mismatch-repair.md:73:## 3) Immediate safety actions (decision criteria)
docs/runbooks/reconcile-mismatch-repair.md:101:- `FLATTEN`: emergency exposure neutralization.
docs/runbooks/reconcile-mismatch-repair.md:140:## 5) Canary-first restoration and rollback
docs/runbooks/reconcile-mismatch-repair.md:169:## 6) Operator checklist
docs/handoff/HANDOFF.json:8:    "On next docs task, update status.json and trace-index.json first.",
docs/handoff/HANDOFF.json:16:    "cat docs/status/status.json",
docs/handoff/HANDOFF.json:17:    "cat docs/status/trace-index.json"
docs/changelog.md:167:- Updated root npm scripts and CI to run SDK tests.
docs/changelog.md:203:- Updated CI to run backend tests in addition to contracts.
docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/changelog.md:219:- Updated CI workflow to enforce contract linting/validation.
docs/changelog.md:228:- Added minimal CI skeleton workflow in `.github/workflows/ci.yml`.
docs/audits/repo-progress-audit-2026-02-14.md:9:- Recent commit stream includes merged PRs #32–#37 and latest additions for market ticker UI and GMO live execution support.
docs/audits/repo-progress-audit-2026-02-14.md:10:- CI workflow is present (`.github/workflows/ci.yml`) with jobs for contracts lint/validation, dashboard API tests, SDK tests, web build, and baseline script checks.
docs/audits/repo-progress-audit-2026-02-14.md:15:  - `docs/changelog.md` (step-by-step merged history)
docs/audits/repo-progress-audit-2026-02-14.md:32:### B. What is merged vs in-flight vs missing
docs/audits/repo-progress-audit-2026-02-14.md:33:- Merged (verified by roadmap/changelog/checklist and code presence):
docs/audits/repo-progress-audit-2026-02-14.md:36:  - Latest merged commits include market ticker page/proxy and GMO live execution support.
docs/audits/repo-progress-audit-2026-02-14.md:49:- CI observability risk:
docs/audits/repo-progress-audit-2026-02-14.md:50:  - Local repo contains CI config but no local artifact/log history for recent remote CI failures.
docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/repo-progress-audit-2026-02-14.md:77:| T191 | UI bots fields align (`state/degraded/degraded_reason/last_seen`) | IN PROGRESS (planned, not merged as dedicated MRU) | `docs/audits/ui-current-vs-spec.md` recommended follow-up; no dedicated task artifact found |
docs/audits/repo-progress-audit-2026-02-14.md:78:| T192 | UI kill-switch read-only panel | IN PROGRESS (planned, not merged as dedicated MRU) | `docs/audits/ui-current-vs-spec.md` recommended follow-up |
docs/audits/repo-progress-audit-2026-02-14.md:87:- Allowed paths:
docs/audits/repo-progress-audit-2026-02-14.md:92:- Forbidden paths:
docs/audits/repo-progress-audit-2026-02-14.md:109:- Allowed paths:
docs/audits/repo-progress-audit-2026-02-14.md:113:- Forbidden paths:
docs/audits/repo-progress-audit-2026-02-14.md:125:- Allowed paths:
docs/audits/repo-progress-audit-2026-02-14.md:129:- Forbidden paths:
docs/audits/repo-progress-audit-2026-02-14.md:144:- Allowed paths:
docs/audits/repo-progress-audit-2026-02-14.md:147:- Forbidden paths:
docs/audits/repo-progress-audit-2026-02-14.md:165:Allowed paths (strict):
docs/audits/repo-progress-audit-2026-02-14.md:171:Forbidden paths:
docs/audits/repo-progress-audit-2026-02-14.md:180:5) Update docs/spec acceptance checklist minimally if behavior changes.
docs/audits/repo-progress-audit-2026-02-14.md:199:Allowed paths (strict):
docs/audits/repo-progress-audit-2026-02-14.md:204:Forbidden paths:
docs/audits/repo-progress-audit-2026-02-14.md:232:Allowed paths (strict):
docs/audits/repo-progress-audit-2026-02-14.md:237:Forbidden paths:
docs/audits/docs-os-existing-file-scan.md:51:## Command: rg -n "README_AI|SSOT|handoff|HANDOFF|decision|decisions|status\.json|trace-index|LOCKS|glossary|canonical|North Star" docs -S
docs/audits/docs-os-existing-file-scan.md:53:docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/audits/docs-os-existing-file-scan.md:61:docs/specs/simple-bot.md:10:- Logging: lines include `run_id`, `bot_id`, `state`, `decision`, `idempotency_key` (and `order_id` when available)
docs/audits/docs-os-existing-file-scan.md:81:docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/docs-os-existing-file-scan.md:82:docs/runbooks/reconcile-mismatch-repair.md:73:## 3) Immediate safety actions (decision criteria)
docs/audits/docs-os-existing-file-scan.md:85:docs/status/progress-updates/UG-P0-110.md:15:   - Backoff/degraded window (`LIVE_DEGRADED`, decision=`THROTTLE`)
docs/audits/docs-os-existing-file-scan.md:86:docs/status/progress-updates/UG-P0-110.md:20:- `decision`: `ALLOW | BLOCK | THROTTLE | REDUCE_ONLY | CLOSE_ONLY | FLATTEN | HALT`
docs/audits/docs-os-existing-file-scan.md:87:docs/status/progress-updates/UG-P0-101.md:8:- `policy_decision.schema.json`
docs/audits/docs-os-existing-file-scan.md:88:docs/status/progress-updates/UG-P0-101.md:9:  - Added PolicyDecision object schema with required fields: `decision`, `reason_code`.
docs/audits/docs-os-existing-file-scan.md:89:docs/status/progress-updates/UG-P0-101.md:17:  "policy_decision": {
docs/audits/docs-os-existing-file-scan.md:90:docs/status/progress-updates/UG-P0-101.md:18:    "decision": "BLOCK",
docs/audits/docs-os-existing-file-scan.md:91:docs/status/progress-updates/UG-P0-111.md:7:- Added safety decision criteria for `SAFE_MODE`, `CLOSE_ONLY`, and `FLATTEN`/`HALT` escalation.
docs/audits/docs-os-existing-file-scan.md:94:docs/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/audits/docs-os-existing-file-scan.md:95:docs/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/audits/docs-os-existing-file-scan.md:97:docs/workplan/ultimate-gold-implementation-feature-list.md:33:| Notes/Links | `<docs/decision/pr>` |
docs/audits/docs-os-existing-file-scan.md:99:docs/workplan/ultimate-gold-implementation-feature-list.md:55:| Scope | `decisionlog-assumptions-ssot-governance` |
docs/audits/docs-os-existing-file-scan.md:100:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/audits/docs-os-existing-file-scan.md:105:## Command: find docs -maxdepth 4 -type f \( -iname "*readme*" -o -iname "*ssot*" -o -iname "*handoff*" -o -iname "*decision*" -o -iname "*status*.json" -o -iname "*trace*" -o -iname "*locks*" -o -iname "*glossary*" \) -print
docs/audits/docs-os-consolidation-decision.md:6:| Machine-readable current status | No direct candidate found in `docs/**` (`status.json` not present) | `docs/status/status.json` | **Keep (new canonical)** | Add a dedicated structured status file for active epic/task, locks, next actions, and PR state. |
docs/audits/docs-os-consolidation-decision.md:7:| Handoff protocol | No direct candidate found in `docs/**` (`HANDOFF.json` not present) | `docs/handoff/HANDOFF.json` | **Keep (new canonical)** | Add explicit stop/credit-out handoff file with required fields for multi-AI continuity. |
docs/audits/docs-os-consolidation-decision.md:8:| Decision log | No direct candidate found in `docs/**` (`docs/decisions/*.md` absent). Related mentions in `docs/changelog.md` and workplan notes. | `docs/decisions/decisions.md` | **Merge** | Keep changelog/workplan for their original purpose, and establish one decision log canonical path for short decision entries. |
docs/audits/docs-os-consolidation-decision.md:9:| Traceability index | No direct candidate found in `docs/**` (`trace-index.json` absent) | `docs/status/trace-index.json` | **Keep (new canonical)** | Introduce one machine-readable trace SSOT for req↔progress↔PR/commit/spec/runbook/verification links. |
docs/audits/docs-os-consolidation-decision.md:12:- No existing same-role canonical files were found for `status.json`, `HANDOFF.json`, `decisions.md`, or `trace-index.json`, so no stub replacement was required.
docs/audits/docs-audit-report.md:88:  - MRU概念（`1PR=1scope`, Depends-on, Allowed/Forbidden paths, DoD）が `workplan` に整理済み。
docs/audits/docs-audit-report.md:216:  - CI/手動運用で参照先が異なるため、想定読者（SRE/開発/QA）を明記する。
docs/audits/docs-audit-report.md:269:rg -i "status|progress|workplan|roadmap|spec|design|decision|handoff|mru" -n docs
docs/audits/docs-content-overlap.md:111:Allowed paths:
docs/audits/docs-content-overlap.md:114:Forbidden paths:
docs/audits/docs-rules-unify-decision.md:6:- Reason: existing rule content is concentrated in `docs/specs/parallel-task-safety.md`; new required operational topics (Multi-AI / Credit-out / HANDOFF / status / trace / decisions / LOCK policy) should be explicit and discoverable under a dedicated operations-rule path.
docs/audits/docs-rules-unify-decision.md:11:| Topic | Existing doc(s) | Proposed canonical doc | Action (keep/merge/stub) | Notes |
docs/audits/docs-rules-unify-decision.md:13:| Parallel safety / 1PR=1scope / Allowed/Forbidden | `docs/specs/parallel-task-safety.md` | `docs/rules/parallel-development-safety.md` | merge + stub old | Preserve prior safety intent and checklist; move to rules namespace. |
docs/audits/docs-rules-unify-decision.md:14:| Multi-AI / Credit-out / Handoff | `docs/specs/parallel-task-safety.md` (partial task-generation section only) | `docs/rules/task-generation-policy.md` | merge + extend | Add stop protocol + mandatory HANDOFF update semantics. |
docs/audits/docs-rules-unify-decision.md:15:| LOCK policy | `docs/specs/parallel-task-safety.md` (LOCK/semi-LOCK partial) | `docs/rules/parallel-development-safety.md` | merge + extend | Add explicit lock areas incl. shared-docs and Required Locks declaration. |
docs/audits/docs-rules-unify-decision.md:16:| Decision log policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Standardize decision fixation via `docs/status/decisions.md`. |
docs/audits/docs-rules-unify-decision.md:17:| Traceability policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Set `docs/status/trace-index.md` as trace SSOT, avoid scattered links. |
docs/audits/docs-rules-unify-decision.md:18:| Task card required fields | `docs/specs/parallel-task-safety.md` section 4.1 | `docs/rules/task-generation-policy.md` | merge + extend | Keep minimum fields and add multi-AI runtime governance fields. |
docs/audits/docs-ai-prompts-decision.md:3:| Topic | Existing candidate(s) | Canonical location | Action (keep/merge/stub) | Notes |
docs/audits/docs-ai-prompts-decision.md:5:| Magic prompt (must-read OS) | `docs/SSOT/README_AI.md` mandatory entrypoint; no dedicated magic-prompt block found | `docs/SSOT/README_AI.md` | merge | Keep single canonical onboarding page and add concise universal magic prompt there. |
docs/audits/docs-ai-prompts-decision.md:7:| Stop/Credit-out/Handoff text | Existing stop protocol in `docs/SSOT/README_AI.md` | `docs/SSOT/README_AI.md` + mirrored in `docs/SSOT/AI_PROMPTS.md` | merge | Keep wording consistent with OS: stop/pause/handoff requires HANDOFF update before exit. |
docs/audits/docs-ai-prompts-decision.md:11:- Canonical prompt locations are fixed to two non-conflicting roles:
docs/audits/docs-ai-pack-decision.md:3:| Artifact | Existing candidate(s) | Proposed canonical path | Action (keep/merge/stub) | Notes |
docs/audits/docs-ai-pack-decision.md:5:| AI entry (constitution) | `docs/SSOT/README_AI.md` | `docs/SSOT/README_AI.md` | keep/merge | Must remain canonical AI entrypoint. Add links to new non-SSOT context files. |
docs/audits/docs-ai-pack-decision.md:6:| Human-readable current status | `docs/status/status.json` (machine-readable SSOT), no existing markdown summary found | `docs/status/CURRENT_STATUS.md` | create/merge | New file is a thin, human-readable summary only. Explicitly states SSOT is `docs/status/status.json`. |
docs/audits/docs-ai-pack-decision.md:7:| Tech context (links + constraints) | No dedicated tech-context hub found in `docs/` | `docs/SSOT/TECH_CONTEXT.md` | create/merge | Links-only navigation hub to existing specs/runbooks/status artifacts; no full-schema duplication. |
docs/audits/docs-ai-pack-decision.md:13:- Treat `docs/status/CURRENT_STATUS.md` as non-SSOT summary derived from `docs/status/status.json`.
docs/audits/docs-ai-pack-scan.md:16:docs/audits/docs-os-consolidation-decision.md
docs/audits/docs-ai-pack-scan.md:18:docs/audits/docs-rules-unify-decision.md
docs/audits/docs-ai-pack-scan.md:23:docs/decisions/decisions.md
docs/audits/docs-ai-pack-scan.md:24:docs/handoff/HANDOFF.json
docs/audits/docs-ai-pack-scan.md:43:docs/status/HANDOFF.json
docs/audits/docs-ai-pack-scan.md:44:docs/status/decisions.md
docs/audits/docs-ai-pack-scan.md:54:docs/status/status.json
docs/audits/docs-ai-pack-scan.md:55:docs/status/trace-index.json
docs/audits/docs-ai-pack-scan.md:56:docs/status/trace-index.md
docs/audits/docs-ai-pack-scan.md:64:## Command: rg -n "CURRENT_STATUS|AI_READ_ME|TECH_CONTEXT|README_AI|SSOT|status\.json|trace-index|handoff|HANDOFF|decisions|LOCKS|glossary|context pack" docs -S
docs/audits/docs-ai-pack-scan.md:66:docs/decisions/decisions.md:6:- 2026-02-17: contracts additive-only (see contracts policy SSOT where applicable) / Prevents breaking downstream consumers / Contract evolution must preserve compatibility.
docs/audits/docs-ai-pack-scan.md:67:docs/decisions/decisions.md:7:- 2026-02-17: docs canonical入口 is `docs/SSOT/README_AI.md` / Single entrypoint prevents SSOT ambiguity / AI and human operators start from one canonical path.
docs/audits/docs-ai-pack-scan.md:68:docs/decisions/decisions.md:8:- 2026-02-17: stopping requires `docs/handoff/HANDOFF.json` update / Multi-agent and interrupted work need explicit continuity state / No stop/handoff without handoff state refresh.
docs/audits/docs-ai-pack-scan.md:70:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/audits/docs-ai-pack-scan.md:76:docs/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/audits/docs-ai-pack-scan.md:77:docs/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/audits/docs-ai-pack-scan.md:78:docs/specs/parallel-task-safety.md:8:- `docs/rules/task-generation-policy.md` (task generation / Multi-AI / Credit-out / HANDOFF / status / decisions / trace)
docs/audits/docs-ai-pack-scan.md:80:docs/status/trace-index.md:1:# Trace Index (SSOT)
docs/audits/docs-ai-pack-scan.md:81:docs/status/trace-index.json:16:        "docs/SSOT/README_AI.md",
docs/audits/docs-ai-pack-scan.md:82:docs/status/trace-index.json:17:        "docs/status/status.json",
docs/audits/docs-ai-pack-scan.md:83:docs/status/trace-index.json:18:        "docs/handoff/HANDOFF.json",
docs/audits/docs-ai-pack-scan.md:84:docs/status/trace-index.json:19:        "docs/decisions/decisions.md"
docs/audits/docs-ai-pack-scan.md:87:docs/status/decisions.md:1:# Decisions Log (SSOT)
docs/audits/docs-ai-pack-scan.md:90:docs/SSOT/README_AI.md:8:2. Runtime status: `docs/status/status.json`
docs/audits/docs-ai-pack-scan.md:91:docs/SSOT/README_AI.md:9:3. Handoff state: `docs/status/HANDOFF.json`
docs/audits/docs-ai-pack-scan.md:92:docs/SSOT/README_AI.md:10:4. Decision baseline: `docs/status/decisions.md`
docs/audits/docs-ai-pack-scan.md:94:docs/SSOT/README_AI.md:17:- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/status/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
docs/audits/docs-ai-pack-scan.md:95:docs/SSOT/README_AI.md:18:- **Single active task:** only one active task is tracked in `docs/status/status.json`.
docs/audits/docs-ai-pack-scan.md:96:docs/SSOT/README_AI.md:19:- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
docs/audits/docs-ai-pack-scan.md:97:docs/SSOT/README_AI.md:20:- **LOCK areas:** lock-sensitive areas include contracts/ci/infra/lockfiles and shared-docs (`docs/SSOT/**`, `docs/rules/**`, docs hubs). Declare `Required Locks` in task cards.
docs/audits/docs-ai-pack-scan.md:98:docs/SSOT/README_AI.md:21:- **Trace SSOT:** `docs/status/trace-index.md` is the canonical place for PR/commit/evidence links.
docs/audits/docs-ai-pack-scan.md:99:docs/SSOT/README_AI.md:22:- **Decisions are binding:** use `docs/status/decisions.md` to fix assumptions and supersede only via explicit new decision entries.
docs/audits/docs-ai-pack-scan.md:102:docs/rules/task-generation-policy.md:10:2. `docs/status/status.json`
docs/audits/docs-ai-pack-scan.md:103:docs/rules/task-generation-policy.md:11:3. `docs/status/HANDOFF.json`
docs/audits/docs-ai-pack-scan.md:104:docs/rules/task-generation-policy.md:12:4. `docs/status/decisions.md`
docs/audits/docs-ai-pack-scan.md:105:docs/rules/task-generation-policy.md:19:- Active task ownership and lifecycle are tracked in `docs/status/status.json`.
docs/audits/docs-ai-pack-scan.md:106:docs/rules/task-generation-policy.md:45:- Canonical runtime status must be updated through `docs/status/status.json` only.
docs/audits/docs-ai-pack-scan.md:108:docs/rules/task-generation-policy.md:52:### 4.3 Mandatory HANDOFF update on stop/switch
docs/audits/docs-ai-pack-scan.md:109:docs/rules/task-generation-policy.md:54:Before stopping or switching owner, update `docs/status/HANDOFF.json` with all of:
docs/audits/docs-ai-pack-scan.md:110:docs/rules/task-generation-policy.md:66:  - an update to `docs/status/status.json`, **or**
docs/audits/docs-ai-pack-scan.md:111:docs/rules/task-generation-policy.md:72:- Use `docs/status/decisions.md` to lock assumptions and decisions that affect ongoing work.
docs/audits/docs-ai-pack-scan.md:112:docs/rules/task-generation-policy.md:78:- `docs/status/trace-index.md` is the SSOT for trace links (PRs, commits, issues, run artifacts).
docs/audits/docs-ai-pack-scan.md:113:docs/rules/task-generation-policy.md:80:- Other docs may include convenience links, but must treat trace-index as canonical.
docs/audits/docs-ai-pack-scan.md:116:docs/rules/parallel-development-safety.md:49:- `Required Locks: docs/SSOT/**`
docs/audits/docs-ai-pack-scan.md:117:docs/handoff/HANDOFF.json:8:    "On next docs task, update status.json and trace-index.json first.",
docs/audits/docs-ai-pack-scan.md:118:docs/handoff/HANDOFF.json:9:    "If stopping mid-task, update this handoff file before exit."
docs/audits/docs-ai-pack-scan.md:119:docs/handoff/HANDOFF.json:16:    "cat docs/status/status.json",
docs/audits/docs-ai-pack-scan.md:120:docs/handoff/HANDOFF.json:17:    "cat docs/status/trace-index.json"
docs/audits/docs-ai-pack-scan.md:121:docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/audits/docs-ai-pack-scan.md:123:docs/audits/docs-os-existing-file-scan.md:51:## Command: rg -n "README_AI|SSOT|handoff|HANDOFF|decision|decisions|status\.json|trace-index|LOCKS|glossary|canonical|North Star" docs -S
docs/audits/docs-ai-pack-scan.md:124:docs/audits/docs-os-existing-file-scan.md:53:docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/audits/docs-ai-pack-scan.md:134:docs/audits/docs-os-existing-file-scan.md:81:docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/docs-ai-pack-scan.md:138:docs/audits/docs-os-existing-file-scan.md:94:docs/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/audits/docs-ai-pack-scan.md:139:docs/audits/docs-os-existing-file-scan.md:95:docs/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/audits/docs-ai-pack-scan.md:142:docs/audits/docs-os-existing-file-scan.md:100:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/audits/docs-ai-pack-scan.md:144:docs/audits/docs-os-existing-file-scan.md:105:## Command: find docs -maxdepth 4 -type f \( -iname "*readme*" -o -iname "*ssot*" -o -iname "*handoff*" -o -iname "*decision*" -o -iname "*status*.json" -o -iname "*trace*" -o -iname "*locks*" -o -iname "*glossary*" \) -print
docs/audits/docs-ai-pack-scan.md:155:docs/audits/docs-audit-report.md:269:rg -i "status|progress|workplan|roadmap|spec|design|decision|handoff|mru" -n docs
docs/audits/docs-ai-pack-scan.md:156:docs/audits/docs-rules-unify-decision.md:5:- Chosen approach: **Option A** (`docs/rules/` as rules SSOT namespace).
docs/audits/docs-ai-pack-scan.md:157:docs/audits/docs-rules-unify-decision.md:6:- Reason: existing rule content is concentrated in `docs/specs/parallel-task-safety.md`; new required operational topics (Multi-AI / Credit-out / HANDOFF / status / trace / decisions / LOCK policy) should be explicit and discoverable under a dedicated operations-rule path.
docs/audits/docs-ai-pack-scan.md:158:docs/audits/docs-rules-unify-decision.md:14:| Multi-AI / Credit-out / Handoff | `docs/specs/parallel-task-safety.md` (partial task-generation section only) | `docs/rules/task-generation-policy.md` | merge + extend | Add stop protocol + mandatory HANDOFF update semantics. |
docs/audits/docs-ai-pack-scan.md:159:docs/audits/docs-rules-unify-decision.md:16:| Decision log policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Standardize decision fixation via `docs/status/decisions.md`. |
docs/audits/docs-ai-pack-scan.md:160:docs/audits/docs-rules-unify-decision.md:17:| Traceability policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Set `docs/status/trace-index.md` as trace SSOT, avoid scattered links. |
docs/audits/docs-ai-pack-scan.md:161:docs/audits/docs-rules-unify-decision.md:22:- `docs/SSOT/README_AI.md` is created/updated as must-read entrypoint for AI operators.
docs/audits/docs-ai-pack-scan.md:162:docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/docs-ai-pack-scan.md:163:docs/audits/docs-os-consolidation-decision.md:5:| SSOT entry for Docs Development OS | `docs/README.md` (general docs index), `docs/specs/parallel-task-safety.md` (parallel safety SSOT note), `docs/audits/docs-content-overlap.md` (canonicalization guidance) | `docs/SSOT/README_AI.md` | **Merge** | Existing docs already describe partial SSOT/canonical concepts, but no single AI-first entrypoint exists. Create one canonical entrance and link all SSOT roles there. |
docs/audits/docs-ai-pack-scan.md:164:docs/audits/docs-os-consolidation-decision.md:6:| Machine-readable current status | No direct candidate found in `docs/**` (`status.json` not present) | `docs/status/status.json` | **Keep (new canonical)** | Add a dedicated structured status file for active epic/task, locks, next actions, and PR state. |
docs/audits/docs-ai-pack-scan.md:165:docs/audits/docs-os-consolidation-decision.md:7:| Handoff protocol | No direct candidate found in `docs/**` (`HANDOFF.json` not present) | `docs/handoff/HANDOFF.json` | **Keep (new canonical)** | Add explicit stop/credit-out handoff file with required fields for multi-AI continuity. |
docs/audits/docs-ai-pack-scan.md:166:docs/audits/docs-os-consolidation-decision.md:8:| Decision log | No direct candidate found in `docs/**` (`docs/decisions/*.md` absent). Related mentions in `docs/changelog.md` and workplan notes. | `docs/decisions/decisions.md` | **Merge** | Keep changelog/workplan for their original purpose, and establish one decision log canonical path for short decision entries. |
docs/audits/docs-ai-pack-scan.md:167:docs/audits/docs-os-consolidation-decision.md:9:| Traceability index | No direct candidate found in `docs/**` (`trace-index.json` absent) | `docs/status/trace-index.json` | **Keep (new canonical)** | Introduce one machine-readable trace SSOT for req↔progress↔PR/commit/spec/runbook/verification links. |
docs/audits/docs-ai-pack-scan.md:168:docs/audits/docs-os-consolidation-decision.md:12:- No existing same-role canonical files were found for `status.json`, `HANDOFF.json`, `decisions.md`, or `trace-index.json`, so no stub replacement was required.
docs/audits/docs-ai-pack-scan.md:169:docs/audits/docs-os-consolidation-decision.md:13:- `docs/README.md` remains a general docs hub; `docs/SSOT/README_AI.md` becomes the canonical AI operating entrypoint and references role-specific SSOT paths.
docs/audits/docs-ai-pack-scan.md:177:docs/audits/docs-rules-unify-scan.md:52:## 3) `rg -n "parallel-task-safety|タスク生成|task generation|Codexアジャイル|致命的事故|guardrail|LOCK:|handoff|HANDOFF|status\.json|trace-index|decisions|SSOT|README_AI" docs -S`
docs/audits/docs-ai-pack-scan.md:178:docs/audits/docs-rules-unify-scan.md:55:docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/audits/docs-ai-pack-scan.md:181:docs/audits/docs-rules-unify-scan.md:58:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/audits/docs-ai-pack-scan.md:183:docs/audits/docs-rules-unify-scan.md:62:docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/docs-ai-pack-scan.md:193:docs/audits/docs-rules-unify-scan.md:72:docs/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/audits/docs-ai-pack-scan.md:194:docs/audits/docs-rules-unify-scan.md:73:docs/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/audits/docs-ai-pack-scan.md:198:docs/audits/docs-rules-unify-scan.md:87:  -iname "*handoff*" -o -iname "*decision*" -o -iname "*status*.json" -o -iname "*trace*" \
docs/audits/docs-ai-pack-scan.md:204:docs/status/status.json
docs/audits/docs-rules-unify-scan.md:52:## 3) `rg -n "parallel-task-safety|タスク生成|task generation|Codexアジャイル|致命的事故|guardrail|LOCK:|handoff|HANDOFF|status\.json|trace-index|decisions|SSOT|README_AI" docs -S`
docs/audits/docs-rules-unify-scan.md:55:docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/audits/docs-rules-unify-scan.md:58:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/audits/docs-rules-unify-scan.md:62:docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/docs-rules-unify-scan.md:72:docs/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/audits/docs-rules-unify-scan.md:73:docs/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/audits/docs-rules-unify-scan.md:87:  -iname "*handoff*" -o -iname "*decision*" -o -iname "*status*.json" -o -iname "*trace*" \
docs/audits/docs-ai-prompts-scan.md:15:docs/audits/docs-ai-pack-decision.md
docs/audits/docs-ai-prompts-scan.md:19:docs/audits/docs-os-consolidation-decision.md
docs/audits/docs-ai-prompts-scan.md:21:docs/audits/docs-rules-unify-decision.md
docs/audits/docs-ai-prompts-scan.md:26:docs/decisions/decisions.md
docs/audits/docs-ai-prompts-scan.md:27:docs/handoff/HANDOFF.json
docs/audits/docs-ai-prompts-scan.md:47:docs/status/HANDOFF.json
docs/audits/docs-ai-prompts-scan.md:48:docs/status/decisions.md
docs/audits/docs-ai-prompts-scan.md:58:docs/status/status.json
docs/audits/docs-ai-prompts-scan.md:59:docs/status/trace-index.json
docs/audits/docs-ai-prompts-scan.md:60:docs/status/trace-index.md
docs/audits/docs-ai-prompts-scan.md:68:## Command: rg -n "magic prompt|プロンプト|prompt template|Planner|Builder|Verifier|Reviewer|Copilot|Claude|Gemini|ChatGPT|README_AI|SSOT|handoff|HANDOFF|status\.json|trace-index|LOCK" docs -S
docs/audits/docs-ai-prompts-scan.md:70:docs/decisions/decisions.md:6:- 2026-02-17: contracts additive-only (see contracts policy SSOT where applicable) / Prevents breaking downstream consumers / Contract evolution must preserve compatibility.
docs/audits/docs-ai-prompts-scan.md:71:docs/decisions/decisions.md:7:- 2026-02-17: docs canonical入口 is `docs/SSOT/README_AI.md` / Single entrypoint prevents SSOT ambiguity / AI and human operators start from one canonical path.
docs/audits/docs-ai-prompts-scan.md:72:docs/decisions/decisions.md:8:- 2026-02-17: stopping requires `docs/handoff/HANDOFF.json` update / Multi-agent and interrupted work need explicit continuity state / No stop/handoff without handoff state refresh.
docs/audits/docs-ai-prompts-scan.md:82:docs/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/audits/docs-ai-prompts-scan.md:83:docs/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/audits/docs-ai-prompts-scan.md:84:docs/specs/parallel-task-safety.md:8:- `docs/rules/task-generation-policy.md` (task generation / Multi-AI / Credit-out / HANDOFF / status / decisions / trace)
docs/audits/docs-ai-prompts-scan.md:86:docs/status/trace-index.md:1:# Trace Index (SSOT)
docs/audits/docs-ai-prompts-scan.md:87:docs/status/trace-index.json:16:        "docs/SSOT/README_AI.md",
docs/audits/docs-ai-prompts-scan.md:88:docs/status/trace-index.json:17:        "docs/status/status.json",
docs/audits/docs-ai-prompts-scan.md:89:docs/status/trace-index.json:18:        "docs/handoff/HANDOFF.json",
docs/audits/docs-ai-prompts-scan.md:95:docs/status/decisions.md:1:# Decisions Log (SSOT)
docs/audits/docs-ai-prompts-scan.md:96:docs/status/CURRENT_STATUS.md:3:_Last updated: 1970-01-01T00:00:00Z (from `docs/status/status.json`)_
docs/audits/docs-ai-prompts-scan.md:97:docs/status/CURRENT_STATUS.md:6:> **SSOT is `docs/status/status.json`**.
docs/audits/docs-ai-prompts-scan.md:98:docs/status/CURRENT_STATUS.md:11:- Active epic: _not specified in `status.json`_
docs/audits/docs-ai-prompts-scan.md:99:docs/status/CURRENT_STATUS.md:17:- _No open PRs recorded in `status.json`._
docs/audits/docs-ai-prompts-scan.md:100:docs/status/CURRENT_STATUS.md:21:- _No locks recorded in `status.json`._
docs/audits/docs-ai-prompts-scan.md:101:docs/status/CURRENT_STATUS.md:25:- _None explicitly recorded in `status.json`._
docs/audits/docs-ai-prompts-scan.md:102:docs/status/CURRENT_STATUS.md:29:- Update `docs/status/status.json` when a task owner is assigned.
docs/audits/docs-ai-prompts-scan.md:103:docs/status/CURRENT_STATUS.md:31:- Add progress evidence under `docs/status/progress-updates/` and trace links in `docs/status/trace-index.md`/`.json` as work proceeds.
docs/audits/docs-ai-prompts-scan.md:104:docs/status/CURRENT_STATUS.md:36:- `docs/status/status.json` (machine-readable SSOT)
docs/audits/docs-ai-prompts-scan.md:105:docs/status/progress-updates/UG-P0-101.md:18:    "decision": "BLOCK",
docs/audits/docs-ai-prompts-scan.md:109:docs/status/progress-updates/UG-P0-110.md:20:- `decision`: `ALLOW | BLOCK | THROTTLE | REDUCE_ONLY | CLOSE_ONLY | FLATTEN | HALT`
docs/audits/docs-ai-prompts-scan.md:112:docs/rules/task-generation-policy.md:10:2. `docs/status/status.json`
docs/audits/docs-ai-prompts-scan.md:113:docs/rules/task-generation-policy.md:11:3. `docs/status/HANDOFF.json`
docs/audits/docs-ai-prompts-scan.md:114:docs/rules/task-generation-policy.md:19:- Active task ownership and lifecycle are tracked in `docs/status/status.json`.
docs/audits/docs-ai-prompts-scan.md:116:docs/rules/task-generation-policy.md:45:- Canonical runtime status must be updated through `docs/status/status.json` only.
docs/audits/docs-ai-prompts-scan.md:118:docs/rules/task-generation-policy.md:52:### 4.3 Mandatory HANDOFF update on stop/switch
docs/audits/docs-ai-prompts-scan.md:119:docs/rules/task-generation-policy.md:54:Before stopping or switching owner, update `docs/status/HANDOFF.json` with all of:
docs/audits/docs-ai-prompts-scan.md:120:docs/rules/task-generation-policy.md:66:  - an update to `docs/status/status.json`, **or**
docs/audits/docs-ai-prompts-scan.md:121:docs/rules/task-generation-policy.md:78:- `docs/status/trace-index.md` is the SSOT for trace links (PRs, commits, issues, run artifacts).
docs/audits/docs-ai-prompts-scan.md:122:docs/rules/task-generation-policy.md:80:- Other docs may include convenience links, but must treat trace-index as canonical.
docs/audits/docs-ai-prompts-scan.md:129:docs/rules/parallel-development-safety.md:49:- `Required Locks: docs/SSOT/**`
docs/audits/docs-ai-prompts-scan.md:134:docs/SSOT/README_AI.md:9:3. Runtime status SSOT: `docs/status/status.json`
docs/audits/docs-ai-prompts-scan.md:136:docs/SSOT/README_AI.md:11:5. Handoff state: `docs/status/HANDOFF.json`
docs/audits/docs-ai-prompts-scan.md:137:docs/SSOT/README_AI.md:13:7. Trace SSOT: `docs/status/trace-index.md`
docs/audits/docs-ai-prompts-scan.md:141:docs/SSOT/README_AI.md:22:- **Status SSOT:** `docs/status/status.json`
docs/audits/docs-ai-prompts-scan.md:144:docs/SSOT/README_AI.md:29:- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/status/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
docs/audits/docs-ai-prompts-scan.md:145:docs/SSOT/README_AI.md:30:- **Single active task:** only one active task is tracked in `docs/status/status.json`.
docs/audits/docs-ai-prompts-scan.md:146:docs/SSOT/README_AI.md:31:- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
docs/audits/docs-ai-prompts-scan.md:147:docs/SSOT/README_AI.md:32:- **LOCK areas:** lock-sensitive areas include contracts/ci/infra/lockfiles and shared-docs (`docs/SSOT/**`, `docs/rules/**`, docs hubs). Declare `Required Locks` in task cards.
docs/audits/docs-ai-prompts-scan.md:148:docs/SSOT/README_AI.md:33:- **Trace SSOT:** `docs/status/trace-index.md` is the canonical place for PR/commit/evidence links.
docs/audits/docs-ai-prompts-scan.md:151:docs/SSOT/TECH_CONTEXT.md:12:- Runtime status SSOT: `docs/status/status.json`
docs/audits/docs-ai-prompts-scan.md:152:docs/SSOT/TECH_CONTEXT.md:13:- Trace index: `docs/status/trace-index.md` and `docs/status/trace-index.json`
docs/audits/docs-ai-prompts-scan.md:153:docs/SSOT/TECH_CONTEXT.md:15:- Handoff state: `docs/status/HANDOFF.json` (legacy/alt path may exist at `docs/handoff/HANDOFF.json`)
docs/audits/docs-ai-prompts-scan.md:154:docs/SSOT/TECH_CONTEXT.md:42:- When in doubt, follow linked SSOT/authoritative docs and record decisions in `docs/status/decisions.md`.
docs/audits/docs-ai-prompts-scan.md:155:docs/handoff/HANDOFF.json:8:    "On next docs task, update status.json and trace-index.json first.",
docs/audits/docs-ai-prompts-scan.md:156:docs/handoff/HANDOFF.json:9:    "If stopping mid-task, update this handoff file before exit."
docs/audits/docs-ai-prompts-scan.md:157:docs/handoff/HANDOFF.json:16:    "cat docs/status/status.json",
docs/audits/docs-ai-prompts-scan.md:158:docs/handoff/HANDOFF.json:17:    "cat docs/status/trace-index.json"
docs/audits/docs-ai-prompts-scan.md:159:docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/audits/docs-ai-prompts-scan.md:161:docs/audits/docs-os-existing-file-scan.md:51:## Command: rg -n "README_AI|SSOT|handoff|HANDOFF|decision|decisions|status\.json|trace-index|LOCKS|glossary|canonical|North Star" docs -S
docs/audits/docs-ai-prompts-scan.md:162:docs/audits/docs-os-existing-file-scan.md:53:docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/audits/docs-ai-prompts-scan.md:172:docs/audits/docs-os-existing-file-scan.md:81:docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/docs-ai-prompts-scan.md:174:docs/audits/docs-os-existing-file-scan.md:86:docs/status/progress-updates/UG-P0-110.md:20:- `decision`: `ALLOW | BLOCK | THROTTLE | REDUCE_ONLY | CLOSE_ONLY | FLATTEN | HALT`
docs/audits/docs-ai-prompts-scan.md:175:docs/audits/docs-os-existing-file-scan.md:90:docs/status/progress-updates/UG-P0-101.md:18:    "decision": "BLOCK",
docs/audits/docs-ai-prompts-scan.md:178:docs/audits/docs-os-existing-file-scan.md:94:docs/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/audits/docs-ai-prompts-scan.md:179:docs/audits/docs-os-existing-file-scan.md:95:docs/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/audits/docs-ai-prompts-scan.md:183:docs/audits/docs-os-existing-file-scan.md:105:## Command: find docs -maxdepth 4 -type f \( -iname "*readme*" -o -iname "*ssot*" -o -iname "*handoff*" -o -iname "*decision*" -o -iname "*status*.json" -o -iname "*trace*" -o -iname "*locks*" -o -iname "*glossary*" \) -print
docs/audits/docs-ai-prompts-scan.md:194:docs/audits/docs-audit-report.md:269:rg -i "status|progress|workplan|roadmap|spec|design|decision|handoff|mru" -n docs
docs/audits/docs-ai-prompts-scan.md:195:docs/audits/docs-rules-unify-decision.md:5:- Chosen approach: **Option A** (`docs/rules/` as rules SSOT namespace).
docs/audits/docs-ai-prompts-scan.md:196:docs/audits/docs-rules-unify-decision.md:6:- Reason: existing rule content is concentrated in `docs/specs/parallel-task-safety.md`; new required operational topics (Multi-AI / Credit-out / HANDOFF / status / trace / decisions / LOCK policy) should be explicit and discoverable under a dedicated operations-rule path.
docs/audits/docs-ai-prompts-scan.md:197:docs/audits/docs-rules-unify-decision.md:14:| Multi-AI / Credit-out / Handoff | `docs/specs/parallel-task-safety.md` (partial task-generation section only) | `docs/rules/task-generation-policy.md` | merge + extend | Add stop protocol + mandatory HANDOFF update semantics. |
docs/audits/docs-ai-prompts-scan.md:198:docs/audits/docs-rules-unify-decision.md:15:| LOCK policy | `docs/specs/parallel-task-safety.md` (LOCK/semi-LOCK partial) | `docs/rules/parallel-development-safety.md` | merge + extend | Add explicit lock areas incl. shared-docs and Required Locks declaration. |
docs/audits/docs-ai-prompts-scan.md:199:docs/audits/docs-rules-unify-decision.md:16:| Decision log policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Standardize decision fixation via `docs/status/decisions.md`. |
docs/audits/docs-ai-prompts-scan.md:200:docs/audits/docs-rules-unify-decision.md:17:| Traceability policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Set `docs/status/trace-index.md` as trace SSOT, avoid scattered links. |
docs/audits/docs-ai-prompts-scan.md:201:docs/audits/docs-rules-unify-decision.md:22:- `docs/SSOT/README_AI.md` is created/updated as must-read entrypoint for AI operators.
docs/audits/docs-ai-prompts-scan.md:202:docs/audits/docs-ai-pack-decision.md:5:| AI entry (constitution) | `docs/SSOT/README_AI.md` | `docs/SSOT/README_AI.md` | keep/merge | Must remain canonical AI entrypoint. Add links to new non-SSOT context files. |
docs/audits/docs-ai-prompts-scan.md:203:docs/audits/docs-ai-pack-decision.md:6:| Human-readable current status | `docs/status/status.json` (machine-readable SSOT), no existing markdown summary found | `docs/status/CURRENT_STATUS.md` | create/merge | New file is a thin, human-readable summary only. Explicitly states SSOT is `docs/status/status.json`. |
docs/audits/docs-ai-prompts-scan.md:204:docs/audits/docs-ai-pack-decision.md:7:| Tech context (links + constraints) | No dedicated tech-context hub found in `docs/` | `docs/SSOT/TECH_CONTEXT.md` | create/merge | Links-only navigation hub to existing specs/runbooks/status artifacts; no full-schema duplication. |
docs/audits/docs-ai-prompts-scan.md:205:docs/audits/docs-ai-pack-decision.md:8:| Any root-level AI files | None detected | none | avoid | Do not create root-level `AI_READ_ME.md` or equivalent SSOT duplicates. |
docs/audits/docs-ai-prompts-scan.md:206:docs/audits/docs-ai-pack-decision.md:12:- Keep `docs/SSOT/README_AI.md` as the single AI constitution entrypoint.
docs/audits/docs-ai-prompts-scan.md:207:docs/audits/docs-ai-pack-decision.md:13:- Treat `docs/status/CURRENT_STATUS.md` as non-SSOT summary derived from `docs/status/status.json`.
docs/audits/docs-ai-prompts-scan.md:208:docs/audits/docs-ai-pack-decision.md:14:- Keep `docs/SSOT/TECH_CONTEXT.md` as a link hub (not a new canonical source of detailed specs).
docs/audits/docs-ai-prompts-scan.md:210:docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/docs-ai-prompts-scan.md:211:docs/audits/docs-os-consolidation-decision.md:5:| SSOT entry for Docs Development OS | `docs/README.md` (general docs index), `docs/specs/parallel-task-safety.md` (parallel safety SSOT note), `docs/audits/docs-content-overlap.md` (canonicalization guidance) | `docs/SSOT/README_AI.md` | **Merge** | Existing docs already describe partial SSOT/canonical concepts, but no single AI-first entrypoint exists. Create one canonical entrance and link all SSOT roles there. |
docs/audits/docs-ai-prompts-scan.md:212:docs/audits/docs-os-consolidation-decision.md:6:| Machine-readable current status | No direct candidate found in `docs/**` (`status.json` not present) | `docs/status/status.json` | **Keep (new canonical)** | Add a dedicated structured status file for active epic/task, locks, next actions, and PR state. |
docs/audits/docs-ai-prompts-scan.md:213:docs/audits/docs-os-consolidation-decision.md:7:| Handoff protocol | No direct candidate found in `docs/**` (`HANDOFF.json` not present) | `docs/handoff/HANDOFF.json` | **Keep (new canonical)** | Add explicit stop/credit-out handoff file with required fields for multi-AI continuity. |
docs/audits/docs-ai-prompts-scan.md:214:docs/audits/docs-os-consolidation-decision.md:9:| Traceability index | No direct candidate found in `docs/**` (`trace-index.json` absent) | `docs/status/trace-index.json` | **Keep (new canonical)** | Introduce one machine-readable trace SSOT for req↔progress↔PR/commit/spec/runbook/verification links. |
docs/audits/docs-ai-prompts-scan.md:215:docs/audits/docs-os-consolidation-decision.md:12:- No existing same-role canonical files were found for `status.json`, `HANDOFF.json`, `decisions.md`, or `trace-index.json`, so no stub replacement was required.
docs/audits/docs-ai-prompts-scan.md:216:docs/audits/docs-os-consolidation-decision.md:13:- `docs/README.md` remains a general docs hub; `docs/SSOT/README_AI.md` becomes the canonical AI operating entrypoint and references role-specific SSOT paths.
docs/audits/docs-ai-prompts-scan.md:225:docs/audits/docs-ai-pack-scan.md:24:docs/handoff/HANDOFF.json
docs/audits/docs-ai-prompts-scan.md:226:docs/audits/docs-ai-pack-scan.md:43:docs/status/HANDOFF.json
docs/audits/docs-ai-prompts-scan.md:227:docs/audits/docs-ai-pack-scan.md:54:docs/status/status.json
docs/audits/docs-ai-prompts-scan.md:228:docs/audits/docs-ai-pack-scan.md:55:docs/status/trace-index.json
docs/audits/docs-ai-prompts-scan.md:229:docs/audits/docs-ai-pack-scan.md:56:docs/status/trace-index.md
docs/audits/docs-ai-prompts-scan.md:230:docs/audits/docs-ai-pack-scan.md:64:## Command: rg -n "CURRENT_STATUS|AI_READ_ME|TECH_CONTEXT|README_AI|SSOT|status\.json|trace-index|handoff|HANDOFF|decisions|LOCKS|glossary|context pack" docs -S
docs/audits/docs-ai-prompts-scan.md:231:docs/audits/docs-ai-pack-scan.md:66:docs/decisions/decisions.md:6:- 2026-02-17: contracts additive-only (see contracts policy SSOT where applicable) / Prevents breaking downstream consumers / Contract evolution must preserve compatibility.
docs/audits/docs-ai-prompts-scan.md:232:docs/audits/docs-ai-pack-scan.md:67:docs/decisions/decisions.md:7:- 2026-02-17: docs canonical入口 is `docs/SSOT/README_AI.md` / Single entrypoint prevents SSOT ambiguity / AI and human operators start from one canonical path.
docs/audits/docs-ai-prompts-scan.md:233:docs/audits/docs-ai-pack-scan.md:68:docs/decisions/decisions.md:8:- 2026-02-17: stopping requires `docs/handoff/HANDOFF.json` update / Multi-agent and interrupted work need explicit continuity state / No stop/handoff without handoff state refresh.
docs/audits/docs-ai-prompts-scan.md:240:docs/audits/docs-ai-pack-scan.md:76:docs/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/audits/docs-ai-prompts-scan.md:241:docs/audits/docs-ai-pack-scan.md:77:docs/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/audits/docs-ai-prompts-scan.md:242:docs/audits/docs-ai-pack-scan.md:78:docs/specs/parallel-task-safety.md:8:- `docs/rules/task-generation-policy.md` (task generation / Multi-AI / Credit-out / HANDOFF / status / decisions / trace)
docs/audits/docs-ai-prompts-scan.md:244:docs/audits/docs-ai-pack-scan.md:80:docs/status/trace-index.md:1:# Trace Index (SSOT)
docs/audits/docs-ai-prompts-scan.md:245:docs/audits/docs-ai-pack-scan.md:81:docs/status/trace-index.json:16:        "docs/SSOT/README_AI.md",
docs/audits/docs-ai-prompts-scan.md:246:docs/audits/docs-ai-pack-scan.md:82:docs/status/trace-index.json:17:        "docs/status/status.json",
docs/audits/docs-ai-prompts-scan.md:247:docs/audits/docs-ai-pack-scan.md:83:docs/status/trace-index.json:18:        "docs/handoff/HANDOFF.json",
docs/audits/docs-ai-prompts-scan.md:248:docs/audits/docs-ai-pack-scan.md:84:docs/status/trace-index.json:19:        "docs/decisions/decisions.md"
docs/audits/docs-ai-prompts-scan.md:251:docs/audits/docs-ai-pack-scan.md:87:docs/status/decisions.md:1:# Decisions Log (SSOT)
docs/audits/docs-ai-prompts-scan.md:254:docs/audits/docs-ai-pack-scan.md:90:docs/SSOT/README_AI.md:8:2. Runtime status: `docs/status/status.json`
docs/audits/docs-ai-prompts-scan.md:255:docs/audits/docs-ai-pack-scan.md:91:docs/SSOT/README_AI.md:9:3. Handoff state: `docs/status/HANDOFF.json`
docs/audits/docs-ai-prompts-scan.md:256:docs/audits/docs-ai-pack-scan.md:92:docs/SSOT/README_AI.md:10:4. Decision baseline: `docs/status/decisions.md`
docs/audits/docs-ai-prompts-scan.md:258:docs/audits/docs-ai-pack-scan.md:94:docs/SSOT/README_AI.md:17:- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/status/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
docs/audits/docs-ai-prompts-scan.md:259:docs/audits/docs-ai-pack-scan.md:95:docs/SSOT/README_AI.md:18:- **Single active task:** only one active task is tracked in `docs/status/status.json`.
docs/audits/docs-ai-prompts-scan.md:260:docs/audits/docs-ai-pack-scan.md:96:docs/SSOT/README_AI.md:19:- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
docs/audits/docs-ai-prompts-scan.md:261:docs/audits/docs-ai-pack-scan.md:97:docs/SSOT/README_AI.md:20:- **LOCK areas:** lock-sensitive areas include contracts/ci/infra/lockfiles and shared-docs (`docs/SSOT/**`, `docs/rules/**`, docs hubs). Declare `Required Locks` in task cards.
docs/audits/docs-ai-prompts-scan.md:262:docs/audits/docs-ai-pack-scan.md:98:docs/SSOT/README_AI.md:21:- **Trace SSOT:** `docs/status/trace-index.md` is the canonical place for PR/commit/evidence links.
docs/audits/docs-ai-prompts-scan.md:263:docs/audits/docs-ai-pack-scan.md:99:docs/SSOT/README_AI.md:22:- **Decisions are binding:** use `docs/status/decisions.md` to fix assumptions and supersede only via explicit new decision entries.
docs/audits/docs-ai-prompts-scan.md:266:docs/audits/docs-ai-pack-scan.md:102:docs/rules/task-generation-policy.md:10:2. `docs/status/status.json`
docs/audits/docs-ai-prompts-scan.md:267:docs/audits/docs-ai-pack-scan.md:103:docs/rules/task-generation-policy.md:11:3. `docs/status/HANDOFF.json`
docs/audits/docs-ai-prompts-scan.md:268:docs/audits/docs-ai-pack-scan.md:105:docs/rules/task-generation-policy.md:19:- Active task ownership and lifecycle are tracked in `docs/status/status.json`.
docs/audits/docs-ai-prompts-scan.md:269:docs/audits/docs-ai-pack-scan.md:106:docs/rules/task-generation-policy.md:45:- Canonical runtime status must be updated through `docs/status/status.json` only.
docs/audits/docs-ai-prompts-scan.md:271:docs/audits/docs-ai-pack-scan.md:108:docs/rules/task-generation-policy.md:52:### 4.3 Mandatory HANDOFF update on stop/switch
docs/audits/docs-ai-prompts-scan.md:272:docs/audits/docs-ai-pack-scan.md:109:docs/rules/task-generation-policy.md:54:Before stopping or switching owner, update `docs/status/HANDOFF.json` with all of:
docs/audits/docs-ai-prompts-scan.md:273:docs/audits/docs-ai-pack-scan.md:110:docs/rules/task-generation-policy.md:66:  - an update to `docs/status/status.json`, **or**
docs/audits/docs-ai-prompts-scan.md:274:docs/audits/docs-ai-pack-scan.md:112:docs/rules/task-generation-policy.md:78:- `docs/status/trace-index.md` is the SSOT for trace links (PRs, commits, issues, run artifacts).
docs/audits/docs-ai-prompts-scan.md:275:docs/audits/docs-ai-pack-scan.md:113:docs/rules/task-generation-policy.md:80:- Other docs may include convenience links, but must treat trace-index as canonical.
docs/audits/docs-ai-prompts-scan.md:278:docs/audits/docs-ai-pack-scan.md:116:docs/rules/parallel-development-safety.md:49:- `Required Locks: docs/SSOT/**`
docs/audits/docs-ai-prompts-scan.md:279:docs/audits/docs-ai-pack-scan.md:117:docs/handoff/HANDOFF.json:8:    "On next docs task, update status.json and trace-index.json first.",
docs/audits/docs-ai-prompts-scan.md:280:docs/audits/docs-ai-pack-scan.md:118:docs/handoff/HANDOFF.json:9:    "If stopping mid-task, update this handoff file before exit."
docs/audits/docs-ai-prompts-scan.md:281:docs/audits/docs-ai-pack-scan.md:119:docs/handoff/HANDOFF.json:16:    "cat docs/status/status.json",
docs/audits/docs-ai-prompts-scan.md:282:docs/audits/docs-ai-pack-scan.md:120:docs/handoff/HANDOFF.json:17:    "cat docs/status/trace-index.json"
docs/audits/docs-ai-prompts-scan.md:283:docs/audits/docs-ai-pack-scan.md:121:docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/audits/docs-ai-prompts-scan.md:285:docs/audits/docs-ai-pack-scan.md:123:docs/audits/docs-os-existing-file-scan.md:51:## Command: rg -n "README_AI|SSOT|handoff|HANDOFF|decision|decisions|status\.json|trace-index|LOCKS|glossary|canonical|North Star" docs -S
docs/audits/docs-ai-prompts-scan.md:286:docs/audits/docs-ai-pack-scan.md:124:docs/audits/docs-os-existing-file-scan.md:53:docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/audits/docs-ai-prompts-scan.md:296:docs/audits/docs-ai-pack-scan.md:134:docs/audits/docs-os-existing-file-scan.md:81:docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/docs-ai-prompts-scan.md:300:docs/audits/docs-ai-pack-scan.md:138:docs/audits/docs-os-existing-file-scan.md:94:docs/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/audits/docs-ai-prompts-scan.md:301:docs/audits/docs-ai-pack-scan.md:139:docs/audits/docs-os-existing-file-scan.md:95:docs/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/audits/docs-ai-prompts-scan.md:305:docs/audits/docs-ai-pack-scan.md:144:docs/audits/docs-os-existing-file-scan.md:105:## Command: find docs -maxdepth 4 -type f \( -iname "*readme*" -o -iname "*ssot*" -o -iname "*handoff*" -o -iname "*decision*" -o -iname "*status*.json" -o -iname "*trace*" -o -iname "*locks*" -o -iname "*glossary*" \) -print
docs/audits/docs-ai-prompts-scan.md:316:docs/audits/docs-ai-pack-scan.md:155:docs/audits/docs-audit-report.md:269:rg -i "status|progress|workplan|roadmap|spec|design|decision|handoff|mru" -n docs
docs/audits/docs-ai-prompts-scan.md:317:docs/audits/docs-ai-pack-scan.md:156:docs/audits/docs-rules-unify-decision.md:5:- Chosen approach: **Option A** (`docs/rules/` as rules SSOT namespace).
docs/audits/docs-ai-prompts-scan.md:318:docs/audits/docs-ai-pack-scan.md:157:docs/audits/docs-rules-unify-decision.md:6:- Reason: existing rule content is concentrated in `docs/specs/parallel-task-safety.md`; new required operational topics (Multi-AI / Credit-out / HANDOFF / status / trace / decisions / LOCK policy) should be explicit and discoverable under a dedicated operations-rule path.
docs/audits/docs-ai-prompts-scan.md:319:docs/audits/docs-ai-pack-scan.md:158:docs/audits/docs-rules-unify-decision.md:14:| Multi-AI / Credit-out / Handoff | `docs/specs/parallel-task-safety.md` (partial task-generation section only) | `docs/rules/task-generation-policy.md` | merge + extend | Add stop protocol + mandatory HANDOFF update semantics. |
docs/audits/docs-ai-prompts-scan.md:320:docs/audits/docs-ai-pack-scan.md:159:docs/audits/docs-rules-unify-decision.md:16:| Decision log policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Standardize decision fixation via `docs/status/decisions.md`. |
docs/audits/docs-ai-prompts-scan.md:321:docs/audits/docs-ai-pack-scan.md:160:docs/audits/docs-rules-unify-decision.md:17:| Traceability policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Set `docs/status/trace-index.md` as trace SSOT, avoid scattered links. |
docs/audits/docs-ai-prompts-scan.md:322:docs/audits/docs-ai-pack-scan.md:161:docs/audits/docs-rules-unify-decision.md:22:- `docs/SSOT/README_AI.md` is created/updated as must-read entrypoint for AI operators.
docs/audits/docs-ai-prompts-scan.md:323:docs/audits/docs-ai-pack-scan.md:162:docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/docs-ai-prompts-scan.md:324:docs/audits/docs-ai-pack-scan.md:163:docs/audits/docs-os-consolidation-decision.md:5:| SSOT entry for Docs Development OS | `docs/README.md` (general docs index), `docs/specs/parallel-task-safety.md` (parallel safety SSOT note), `docs/audits/docs-content-overlap.md` (canonicalization guidance) | `docs/SSOT/README_AI.md` | **Merge** | Existing docs already describe partial SSOT/canonical concepts, but no single AI-first entrypoint exists. Create one canonical entrance and link all SSOT roles there. |
docs/audits/docs-ai-prompts-scan.md:325:docs/audits/docs-ai-pack-scan.md:164:docs/audits/docs-os-consolidation-decision.md:6:| Machine-readable current status | No direct candidate found in `docs/**` (`status.json` not present) | `docs/status/status.json` | **Keep (new canonical)** | Add a dedicated structured status file for active epic/task, locks, next actions, and PR state. |
docs/audits/docs-ai-prompts-scan.md:326:docs/audits/docs-ai-pack-scan.md:165:docs/audits/docs-os-consolidation-decision.md:7:| Handoff protocol | No direct candidate found in `docs/**` (`HANDOFF.json` not present) | `docs/handoff/HANDOFF.json` | **Keep (new canonical)** | Add explicit stop/credit-out handoff file with required fields for multi-AI continuity. |
docs/audits/docs-ai-prompts-scan.md:327:docs/audits/docs-ai-pack-scan.md:167:docs/audits/docs-os-consolidation-decision.md:9:| Traceability index | No direct candidate found in `docs/**` (`trace-index.json` absent) | `docs/status/trace-index.json` | **Keep (new canonical)** | Introduce one machine-readable trace SSOT for req↔progress↔PR/commit/spec/runbook/verification links. |
docs/audits/docs-ai-prompts-scan.md:328:docs/audits/docs-ai-pack-scan.md:168:docs/audits/docs-os-consolidation-decision.md:12:- No existing same-role canonical files were found for `status.json`, `HANDOFF.json`, `decisions.md`, or `trace-index.json`, so no stub replacement was required.
docs/audits/docs-ai-prompts-scan.md:329:docs/audits/docs-ai-pack-scan.md:169:docs/audits/docs-os-consolidation-decision.md:13:- `docs/README.md` remains a general docs hub; `docs/SSOT/README_AI.md` becomes the canonical AI operating entrypoint and references role-specific SSOT paths.
docs/audits/docs-ai-prompts-scan.md:337:docs/audits/docs-ai-pack-scan.md:177:docs/audits/docs-rules-unify-scan.md:52:## 3) `rg -n "parallel-task-safety|タスク生成|task generation|Codexアジャイル|致命的事故|guardrail|LOCK:|handoff|HANDOFF|status\.json|trace-index|decisions|SSOT|README_AI" docs -S`
docs/audits/docs-ai-prompts-scan.md:338:docs/audits/docs-ai-pack-scan.md:178:docs/audits/docs-rules-unify-scan.md:55:docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/audits/docs-ai-prompts-scan.md:342:docs/audits/docs-ai-pack-scan.md:183:docs/audits/docs-rules-unify-scan.md:62:docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/docs-ai-prompts-scan.md:352:docs/audits/docs-ai-pack-scan.md:193:docs/audits/docs-rules-unify-scan.md:72:docs/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/audits/docs-ai-prompts-scan.md:353:docs/audits/docs-ai-pack-scan.md:194:docs/audits/docs-rules-unify-scan.md:73:docs/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/audits/docs-ai-prompts-scan.md:357:docs/audits/docs-ai-pack-scan.md:198:docs/audits/docs-rules-unify-scan.md:87:  -iname "*handoff*" -o -iname "*decision*" -o -iname "*status*.json" -o -iname "*trace*" \
docs/audits/docs-ai-prompts-scan.md:359:docs/audits/docs-ai-pack-scan.md:204:docs/status/status.json
docs/audits/docs-ai-prompts-scan.md:360:docs/audits/docs-rules-unify-scan.md:52:## 3) `rg -n "parallel-task-safety|タスク生成|task generation|Codexアジャイル|致命的事故|guardrail|LOCK:|handoff|HANDOFF|status\.json|trace-index|decisions|SSOT|README_AI" docs -S`
docs/audits/docs-ai-prompts-scan.md:361:docs/audits/docs-rules-unify-scan.md:55:docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/audits/docs-ai-prompts-scan.md:365:docs/audits/docs-rules-unify-scan.md:62:docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/docs-ai-prompts-scan.md:375:docs/audits/docs-rules-unify-scan.md:72:docs/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/audits/docs-ai-prompts-scan.md:376:docs/audits/docs-rules-unify-scan.md:73:docs/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/audits/docs-ai-prompts-scan.md:380:docs/audits/docs-rules-unify-scan.md:87:  -iname "*handoff*" -o -iname "*decision*" -o -iname "*status*.json" -o -iname "*trace*" \
```

## Command: find docs -maxdepth 5 -type f \( -iname "*checklist*" -o -iname "*preflight*" -o -iname "*merge*" -o -iname "*review*" -o -iname "*rollback*" -o -iname "*push*" \) -print
```
docs/audits/docs-ai-checklist-scan.md
```
