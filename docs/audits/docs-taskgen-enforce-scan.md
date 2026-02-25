# DOC-TASKGEN-ENFORCE Scan

## Command: git status --short
```
?? docs/audits/docs-taskgen-enforce-scan.md
?? services/marketdata-rs/Cargo.lock
```

## Command: git ls-tree -r --name-only HEAD docs
```
docs/README.md
docs/context/AI_PROMPTS.md
docs/context/README_AI.md
docs/context/TECH_CONTEXT.md
docs/assumptions.md
docs/decisions/records/docs-ai-checklist-decision.md
docs/audits/docs-ai-checklist-scan.md
docs/decisions/records/docs-ai-pack-decision.md
docs/audits/docs-ai-pack-scan.md
docs/decisions/records/docs-ai-prompts-decision.md
docs/audits/docs-ai-prompts-scan.md
docs/audits/docs-audit-report.md
docs/audits/docs-content-overlap.md
docs/decisions/records/docs-os-consolidation-decision.md
docs/audits/docs-os-existing-file-scan.md
docs/decisions/records/docs-rules-unify-decision.md
docs/audits/docs-rules-unify-scan.md
docs/audits/repo-progress-audit-2026-02-14.md
docs/audits/ui-current-vs-spec.md
docs/changelog.md
docs/decisions/decisions.md
docs/handoff/HANDOFF.json
docs/plans/roadmap.md
docs/specs/crosscut/safety_interlock_spec.md
docs/rules/task-generation-policy.md
docs/runbooks/e2e-smoke-runbook.md
docs/runbooks/marketdata-local.md
docs/runbooks/marketdata-replay.md
docs/runbooks/paper_e2e.md
docs/runbooks/pr-preflight.md
docs/runbooks/reconcile-mismatch-repair.md
docs/runbooks/supply-chain-security.md
docs/specs/controlplane-bots.md
docs/specs/crosscut/dangerous_ops_confirmation.md
docs/specs/crosscut/dangerous_ops_taxonomy.md
docs/context/notes/execution-gmo.md
docs/context/notes/execution.md
docs/specs/crosscut/parallel_task_safety_spec.md
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

## Command: rg -n "task generation|タスク生成|parallel-task-safety|Docs Development OS|README_AI|status\.json|HANDOFF|decisions|trace-index|LOCK:" docs -S
```
docs/decisions/decisions.md:7:- 2026-02-17: docs canonical入口 is `docs/context/README_AI.md` / Single entrypoint prevents SSOT ambiguity / AI and human operators start from one canonical path.
docs/decisions/decisions.md:8:- 2026-02-17: stopping requires `docs/handoff/HANDOFF.json` update / Multi-agent and interrupted work need explicit continuity state / No stop/handoff without handoff state refresh.
docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/README.md:6:- AI SSOT entry (canonical docs development OS): [`SSOT/README_AI.md`](SSOT/README_AI.md)
docs/README.md:13:- Rules SSOT (task generation + parallel safety): [`rules/task-generation-policy.md`](rules/task-generation-policy.md), [`rules/parallel-development-safety.md`](rules/parallel-development-safety.md)
docs/README.md:14:- AI operator onboarding SSOT: [`SSOT/README_AI.md`](SSOT/README_AI.md)
docs/specs/crosscut/parallel_task_safety_spec.md:8:- `docs/rules/task-generation-policy.md` (task generation / Multi-AI / Credit-out / HANDOFF / status / decisions / trace)
docs/specs/crosscut/parallel_task_safety_spec.md:12:- `docs/context/README_AI.md`
docs/status/trace-index.json:9:        "docs/specs/crosscut/parallel_task_safety_spec.md"
docs/status/trace-index.json:16:        "docs/context/README_AI.md",
docs/status/trace-index.json:17:        "docs/status/status.json",
docs/status/trace-index.json:18:        "docs/handoff/HANDOFF.json",
docs/status/trace-index.json:19:        "docs/decisions/decisions.md"
docs/status/CURRENT_STATUS.md:3:_Last updated: 1970-01-01T00:00:00Z (from `docs/status/status.json`)_
docs/status/CURRENT_STATUS.md:6:> **SSOT is `docs/status/status.json`**.
docs/status/CURRENT_STATUS.md:11:- Active epic: _not specified in `status.json`_
docs/status/CURRENT_STATUS.md:17:- _No open PRs recorded in `status.json`._
docs/status/CURRENT_STATUS.md:21:- _No locks recorded in `status.json`._
docs/status/CURRENT_STATUS.md:25:- _None explicitly recorded in `status.json`._
docs/status/CURRENT_STATUS.md:29:- Update `docs/status/status.json` when a task owner is assigned.
docs/status/CURRENT_STATUS.md:31:- Add progress evidence under `docs/status/progress-updates/` and trace links in `docs/status/trace-index.md`/`.json` as work proceeds.
docs/status/CURRENT_STATUS.md:36:- `docs/status/status.json` (machine-readable SSOT)
docs/context/AI_PROMPTS.md:4:It is a usage library (non-SSOT) aligned to Docs Development OS.
docs/context/AI_PROMPTS.md:6:- Canonical entrypoint: `docs/context/README_AI.md`
docs/context/AI_PROMPTS.md:7:- Status SSOT: `docs/status/status.json`
docs/context/AI_PROMPTS.md:8:- Trace SSOT: `docs/status/trace-index.json`
docs/context/AI_PROMPTS.md:16:- Read in strict order: `docs/context/README_AI.md` → `docs/status/status.json` → `docs/handoff/HANDOFF.json` (or `docs/status/HANDOFF.json`) → `docs/status/decisions.md`.
docs/context/AI_PROMPTS.md:17:- Before planning or editing, inspect `docs/status/status.json` keys: `active_task`, `open_prs`, `locks_held`, `owner`, `state`, `last_updated`.
docs/context/AI_PROMPTS.md:21:- If pausing/stopping/credit-out before completion, update `docs/handoff/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next` (and sync `docs/status/HANDOFF.json` if that path is being used by current workflow).
docs/context/AI_PROMPTS.md:22:- Any "progress made" claim requires evidence updates in `docs/status/status.json` and/or `docs/status/progress-updates/*`.
docs/context/AI_PROMPTS.md:23:- Record evidence links in `docs/status/trace-index.json` (SSOT) and mirror to markdown index if required.
docs/context/AI_PROMPTS.md:24:- If uncertain, add/update assumptions or decisions via the existing mechanism (`docs/assumptions.md`, `docs/status/decisions.md`) before irreversible changes.
docs/context/AI_PROMPTS.md:32:- Read order: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
docs/context/AI_PROMPTS.md:37:- If assumptions are needed, record them in assumptions/decisions flow before execution.
docs/context/AI_PROMPTS.md:38:- Do not claim progress unless `status.json`/progress-updates are updated.
docs/context/AI_PROMPTS.md:42:- Read order: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
docs/context/AI_PROMPTS.md:47:- On stop/credit-out, update HANDOFF required fields before exit.
docs/context/AI_PROMPTS.md:48:- Any progress statement must be backed by `status.json` or progress update entries.
docs/context/AI_PROMPTS.md:52:- Read order: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
docs/context/AI_PROMPTS.md:55:- If evidence is missing from `status.json`/progress updates/trace index, fail verification.
docs/context/AI_PROMPTS.md:56:- If uncertain behavior exists, request assumptions/decisions update before pass.
docs/context/AI_PROMPTS.md:61:- Read order: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
docs/context/AI_PROMPTS.md:65:- Verify progress claims have concrete evidence updates (`status.json`, progress updates, trace index).
docs/context/AI_PROMPTS.md:74:Read `docs/context/README_AI.md`, then `docs/status/status.json`, `docs/handoff/HANDOFF.json` (or `docs/status/HANDOFF.json`), then `docs/status/decisions.md`; treat `status.json` as SSOT and check `active_task/open_prs/locks_held` before action, declare Allowed/Forbidden paths and Required LOCKS in your task card, stop on LOCK conflict, record trace links in `docs/status/trace-index.json`, and if stopping early update HANDOFF with `what_done/what_next/errors/commands_next`.
docs/context/AI_PROMPTS.md:78:Follow OS read order `README_AI → status.json → HANDOFF → decisions`; use `status.json` as authoritative runtime state (`active_task`, `open_prs`, `locks_held`), operate only in Allowed paths, stop and return on LOCK conflict, require evidence updates for any progress claim (`status.json` or progress updates), write trace evidence to `docs/status/trace-index.json`, and update HANDOFF fields before any pause/credit-out.
docs/context/AI_PROMPTS.md:82:Start by reading `docs/context/README_AI.md`, `docs/status/status.json`, `docs/handoff/HANDOFF.json`/`docs/status/HANDOFF.json`, and `docs/status/decisions.md`; then create a task card with scope + Allowed/Forbidden + LOCKS, execute only safe paths, halt on lock collisions, ensure progress claims are backed by status/progress updates, keep trace links in `docs/status/trace-index.json`, and always write HANDOFF before stopping mid-task.
docs/context/AI_PROMPTS.md:90:- Read: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
docs/context/AI_PROMPTS.md:96:- Update evidence: `status.json` or `progress-updates` + `trace-index.json`.
docs/context/AI_PROMPTS.md:97:- If pausing, update HANDOFF required keys.
docs/context/AI_PROMPTS.md:102:- Validate active ownership and lock status in `status.json`.
docs/context/AI_PROMPTS.md:107:- Record trace links in `trace-index.json`.
docs/context/AI_PROMPTS.md:108:- On credit-out, update HANDOFF with actionable next commands.
docs/context/AI_PROMPTS.md:119:- On interruption, complete HANDOFF fields before ending response.
docs/context/README_AI.md:1:# README_AI (SSOT entrypoint for operators)
docs/context/README_AI.md:7:1. This file: `docs/context/README_AI.md` (AI constitution / entrypoint)
docs/context/README_AI.md:9:3. Runtime status SSOT: `docs/status/status.json`
docs/context/README_AI.md:12:6. Handoff state: `docs/status/HANDOFF.json` and `docs/handoff/HANDOFF.json`
docs/context/README_AI.md:13:7. Decision baseline: `docs/status/decisions.md`
docs/context/README_AI.md:14:8. Trace SSOT: `docs/status/trace-index.md` and `docs/status/trace-index.json`
docs/context/README_AI.md:18:   - `docs/specs/crosscut/parallel_task_safety_spec.md`
docs/context/README_AI.md:23:- **Canonical AI entrypoint:** `docs/context/README_AI.md`
docs/context/README_AI.md:24:- **Status SSOT:** `docs/status/status.json`
docs/context/README_AI.md:34:> Read in order: `docs/context/README_AI.md` → `docs/status/status.json` → `docs/handoff/HANDOFF.json` (or `docs/status/HANDOFF.json`) → `docs/status/decisions.md`.
docs/context/README_AI.md:35:> Treat `docs/status/status.json` as SSOT and check `active_task`, `open_prs`, `locks_held` before planning.
docs/context/README_AI.md:36:> If stopping/pause/credit-out before completion, you MUST update `docs/handoff/HANDOFF.json` (and keep `docs/status/HANDOFF.json` aligned if used).
docs/context/README_AI.md:37:> Use `docs/status/trace-index.json` as trace link SSOT.
docs/context/README_AI.md:42:- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/status/HANDOFF.json` and `docs/handoff/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
docs/context/README_AI.md:43:- **Single active task:** only one active task is tracked in `docs/status/status.json`.
docs/context/README_AI.md:44:- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
docs/context/README_AI.md:46:- **Trace SSOT:** `docs/status/trace-index.md` / `docs/status/trace-index.json` are the canonical place for PR/commit/evidence links.
docs/context/README_AI.md:47:- **Decisions are binding:** use `docs/status/decisions.md` to fix assumptions and supersede only via explicit new decision entries.
docs/context/TECH_CONTEXT.md:10:- AI entrypoint: `docs/context/README_AI.md`
docs/context/TECH_CONTEXT.md:11:- Parallel safety rules: `docs/specs/crosscut/parallel_task_safety_spec.md` and `docs/specs/crosscut/safety_interlock_spec.md`
docs/context/TECH_CONTEXT.md:12:- Runtime status SSOT: `docs/status/status.json`
docs/context/TECH_CONTEXT.md:13:- Trace index: `docs/status/trace-index.md` and `docs/status/trace-index.json`
docs/context/TECH_CONTEXT.md:14:- Decisions baseline: `docs/status/decisions.md` (and `docs/decisions/decisions.md` if referenced by task)
docs/context/TECH_CONTEXT.md:15:- Handoff state: `docs/status/HANDOFF.json` (legacy/alt path may exist at `docs/handoff/HANDOFF.json`)
docs/context/TECH_CONTEXT.md:42:- When in doubt, follow linked SSOT/authoritative docs and record decisions in `docs/status/decisions.md`.
docs/rules/task-generation-policy.md:9:1. `docs/context/README_AI.md`
docs/rules/task-generation-policy.md:10:2. `docs/status/status.json`
docs/rules/task-generation-policy.md:11:3. `docs/status/HANDOFF.json`
docs/rules/task-generation-policy.md:12:4. `docs/status/decisions.md`
docs/rules/task-generation-policy.md:19:- Active task ownership and lifecycle are tracked in `docs/status/status.json`.
docs/rules/task-generation-policy.md:45:- Canonical runtime status must be updated through `docs/status/status.json` only.
docs/rules/task-generation-policy.md:52:### 4.3 Mandatory HANDOFF update on stop/switch
docs/rules/task-generation-policy.md:54:Before stopping or switching owner, update `docs/status/HANDOFF.json` with all of:
docs/rules/task-generation-policy.md:66:  - an update to `docs/status/status.json`, **or**
docs/rules/task-generation-policy.md:72:- Use `docs/status/decisions.md` to lock assumptions and decisions that affect ongoing work.
docs/rules/task-generation-policy.md:78:- `docs/status/trace-index.md` is the SSOT for trace links (PRs, commits, issues, run artifacts).
docs/rules/task-generation-policy.md:80:- Other docs may include convenience links, but must treat trace-index as canonical.
docs/handoff/HANDOFF.json:8:    "On next docs task, update status.json and trace-index.json first.",
docs/handoff/HANDOFF.json:16:    "cat docs/status/status.json",
docs/handoff/HANDOFF.json:17:    "cat docs/status/trace-index.json"
docs/audits/docs-os-existing-file-scan.md:31:docs/specs/crosscut/parallel_task_safety_spec.md
docs/audits/docs-os-existing-file-scan.md:51:## Command: rg -n "README_AI|SSOT|handoff|HANDOFF|decision|decisions|status\.json|trace-index|LOCKS|glossary|canonical|North Star" docs -S
docs/audits/docs-os-existing-file-scan.md:60:docs/specs/crosscut/parallel_task_safety_spec.md:1:# Parallel Task Safety Spec (SSOT)
docs/audits/docs-os-existing-file-scan.md:96:docs/README.md:12:- Parallel task safety (SSOT for parallel development safety): [`specs/parallel-task-safety.md`](specs/parallel-task-safety.md)
docs/audits/docs-os-existing-file-scan.md:100:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/audits/docs-audit-report.md:38:- `docs/specs/crosscut/parallel_task_safety_spec.md` → `rules/spec (parallel safety SSOT)`
docs/audits/docs-audit-report.md:89:  - `parallel-task-safety` に `1PR=1scope` が明文化されている。
docs/audits/docs-audit-report.md:155:  - `docs/specs/crosscut/parallel_task_safety_spec.md`
docs/audits/docs-audit-report.md:161:  - **運用原則SSOT:** `docs/specs/crosscut/parallel_task_safety_spec.md`
docs/audits/docs-audit-report.md:166:  - タスク生成時に参照されるため、互換性のため見出し名は維持推奨。
docs/audits/docs-audit-report.md:245:   - 目的: `1PR=1scope` 等の運用規範を `parallel-task-safety` に集約。
docs/decisions/records/docs-ai-checklist-decision.md:6:| Merge decision template | No dedicated merge-decision template found; partial reminders in `docs/context/README_AI.md` | `docs/runbooks/pr-preflight.md` | merge | Keep one practical checklist/template doc to avoid duplication. |
docs/decisions/records/docs-ai-checklist-decision.md:7:| Rollback prevention rules | Existing safety/parallel policy docs (`docs/specs/crosscut/safety_interlock_spec.md`, `docs/specs/crosscut/parallel_task_safety_spec.md`) | `docs/runbooks/pr-preflight.md` (with references) | reference | Keep concise runbook guidance and link policy sources rather than duplicating policy text. |
docs/decisions/records/docs-ai-checklist-decision.md:8:| Multi-AI handoff requirements | `docs/context/README_AI.md`, `docs/status/HANDOFF.json`, `docs/handoff/HANDOFF.json` | `docs/context/README_AI.md` + `docs/runbooks/pr-preflight.md` link/reference | reference | README_AI stays canonical entrypoint; runbook references handoff obligations for PR/merge flow. |
docs/decisions/records/docs-ai-checklist-decision.md:13:- `docs/context/README_AI.md` remains AI entrypoint and links this checklist as pre-PR must-read.
docs/decisions/records/docs-rules-unify-decision.md:6:- Reason: existing rule content is concentrated in `docs/specs/crosscut/parallel_task_safety_spec.md`; new required operational topics (Multi-AI / Credit-out / HANDOFF / status / trace / decisions / LOCK policy) should be explicit and discoverable under a dedicated operations-rule path.
docs/decisions/records/docs-rules-unify-decision.md:7:- Anti-duplication action: `docs/specs/crosscut/parallel_task_safety_spec.md` is retained as a **stub** that redirects to canonical docs.
docs/decisions/records/docs-rules-unify-decision.md:13:| Parallel safety / 1PR=1scope / Allowed/Forbidden | `docs/specs/crosscut/parallel_task_safety_spec.md` | `docs/specs/crosscut/safety_interlock_spec.md` | merge + stub old | Preserve prior safety intent and checklist; move to rules namespace. |
docs/decisions/records/docs-rules-unify-decision.md:14:| Multi-AI / Credit-out / Handoff | `docs/specs/crosscut/parallel_task_safety_spec.md` (partial task-generation section only) | `docs/rules/task-generation-policy.md` | merge + extend | Add stop protocol + mandatory HANDOFF update semantics. |
docs/decisions/records/docs-rules-unify-decision.md:15:| LOCK policy | `docs/specs/crosscut/parallel_task_safety_spec.md` (LOCK/semi-LOCK partial) | `docs/specs/crosscut/safety_interlock_spec.md` | merge + extend | Add explicit lock areas incl. shared-docs and Required Locks declaration. |
docs/decisions/records/docs-rules-unify-decision.md:16:| Decision log policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Standardize decision fixation via `docs/status/decisions.md`. |
docs/decisions/records/docs-rules-unify-decision.md:17:| Traceability policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Set `docs/status/trace-index.md` as trace SSOT, avoid scattered links. |
docs/decisions/records/docs-rules-unify-decision.md:18:| Task card required fields | `docs/specs/crosscut/parallel_task_safety_spec.md` section 4.1 | `docs/rules/task-generation-policy.md` | merge + extend | Keep minimum fields and add multi-AI runtime governance fields. |
docs/decisions/records/docs-rules-unify-decision.md:22:- `docs/context/README_AI.md` is created/updated as must-read entrypoint for AI operators.
docs/decisions/records/docs-ai-prompts-decision.md:5:| Magic prompt (must-read OS) | `docs/context/README_AI.md` mandatory entrypoint; no dedicated magic-prompt block found | `docs/context/README_AI.md` | merge | Keep single canonical onboarding page and add concise universal magic prompt there. |
docs/decisions/records/docs-ai-prompts-decision.md:6:| Role prompts (Planner/Builder/Verifier/Reviewer) | No dedicated role-prompt file found | `docs/context/AI_PROMPTS.md` | choose one | Create focused prompts file to avoid over-growing README_AI; README_AI links to it. |
docs/decisions/records/docs-ai-prompts-decision.md:7:| Stop/Credit-out/Handoff text | Existing stop protocol in `docs/context/README_AI.md` | `docs/context/README_AI.md` + mirrored in `docs/context/AI_PROMPTS.md` | merge | Keep wording consistent with OS: stop/pause/handoff requires HANDOFF update before exit. |
docs/decisions/records/docs-ai-prompts-decision.md:12:  - `README_AI.md` for universal short magic prompt and governance boundary.
docs/decisions/records/docs-ai-pack-decision.md:5:| AI entry (constitution) | `docs/context/README_AI.md` | `docs/context/README_AI.md` | keep/merge | Must remain canonical AI entrypoint. Add links to new non-SSOT context files. |
docs/decisions/records/docs-ai-pack-decision.md:6:| Human-readable current status | `docs/status/status.json` (machine-readable SSOT), no existing markdown summary found | `docs/status/CURRENT_STATUS.md` | create/merge | New file is a thin, human-readable summary only. Explicitly states SSOT is `docs/status/status.json`. |
docs/decisions/records/docs-ai-pack-decision.md:12:- Keep `docs/context/README_AI.md` as the single AI constitution entrypoint.
docs/decisions/records/docs-ai-pack-decision.md:13:- Treat `docs/status/CURRENT_STATUS.md` as non-SSOT summary derived from `docs/status/status.json`.
docs/runbooks/pr-preflight.md:6:- AI entrypoint: `docs/context/README_AI.md`
docs/runbooks/pr-preflight.md:7:- Runtime status SSOT: `docs/status/status.json`
docs/runbooks/pr-preflight.md:8:- Handoff: `docs/handoff/HANDOFF.json` and `docs/status/HANDOFF.json`
docs/runbooks/pr-preflight.md:9:- Decisions: `docs/status/decisions.md`
docs/runbooks/pr-preflight.md:10:- Trace index SSOT: `docs/status/trace-index.json`
docs/runbooks/pr-preflight.md:11:- Safety policy: `docs/specs/crosscut/safety_interlock_spec.md`, `docs/specs/crosscut/parallel_task_safety_spec.md`
docs/runbooks/pr-preflight.md:24:  - [ ] either `docs/status/status.json` updated, or
docs/runbooks/pr-preflight.md:27:  - [ ] update `docs/handoff/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
docs/runbooks/pr-preflight.md:72:- [ ] PR link/evidence is registered in `docs/status/trace-index.json` if required by team flow.
docs/runbooks/pr-preflight.md:73:- [ ] `docs/status/decisions.md` updated if assumptions/spec baseline changed.
docs/runbooks/pr-preflight.md:81:- If spec/assumptions changed mid-flight, record explicit entry in `docs/status/decisions.md` before merge.
docs/decisions/records/docs-os-consolidation-decision.md:5:| SSOT entry for Docs Development OS | `docs/README.md` (general docs index), `docs/specs/crosscut/parallel_task_safety_spec.md` (parallel safety SSOT note), `docs/audits/docs-content-overlap.md` (canonicalization guidance) | `docs/context/README_AI.md` | **Merge** | Existing docs already describe partial SSOT/canonical concepts, but no single AI-first entrypoint exists. Create one canonical entrance and link all SSOT roles there. |
docs/decisions/records/docs-os-consolidation-decision.md:6:| Machine-readable current status | No direct candidate found in `docs/**` (`status.json` not present) | `docs/status/status.json` | **Keep (new canonical)** | Add a dedicated structured status file for active epic/task, locks, next actions, and PR state. |
docs/decisions/records/docs-os-consolidation-decision.md:7:| Handoff protocol | No direct candidate found in `docs/**` (`HANDOFF.json` not present) | `docs/handoff/HANDOFF.json` | **Keep (new canonical)** | Add explicit stop/credit-out handoff file with required fields for multi-AI continuity. |
docs/decisions/records/docs-os-consolidation-decision.md:8:| Decision log | No direct candidate found in `docs/**` (`docs/decisions/*.md` absent). Related mentions in `docs/changelog.md` and workplan notes. | `docs/decisions/decisions.md` | **Merge** | Keep changelog/workplan for their original purpose, and establish one decision log canonical path for short decision entries. |
docs/decisions/records/docs-os-consolidation-decision.md:9:| Traceability index | No direct candidate found in `docs/**` (`trace-index.json` absent) | `docs/status/trace-index.json` | **Keep (new canonical)** | Introduce one machine-readable trace SSOT for req↔progress↔PR/commit/spec/runbook/verification links. |
docs/decisions/records/docs-os-consolidation-decision.md:12:- No existing same-role canonical files were found for `status.json`, `HANDOFF.json`, `decisions.md`, or `trace-index.json`, so no stub replacement was required.
docs/decisions/records/docs-os-consolidation-decision.md:13:- `docs/README.md` remains a general docs hub; `docs/context/README_AI.md` becomes the canonical AI operating entrypoint and references role-specific SSOT paths.
docs/audits/docs-ai-pack-scan.md:12:docs/context/README_AI.md
docs/audits/docs-ai-pack-scan.md:23:docs/decisions/decisions.md
docs/audits/docs-ai-pack-scan.md:24:docs/handoff/HANDOFF.json
docs/audits/docs-ai-pack-scan.md:39:docs/specs/crosscut/parallel_task_safety_spec.md
docs/audits/docs-ai-pack-scan.md:43:docs/status/HANDOFF.json
docs/audits/docs-ai-pack-scan.md:44:docs/status/decisions.md
docs/audits/docs-ai-pack-scan.md:54:docs/status/status.json
docs/audits/docs-ai-pack-scan.md:55:docs/status/trace-index.json
docs/audits/docs-ai-pack-scan.md:56:docs/status/trace-index.md
docs/audits/docs-ai-pack-scan.md:64:## Command: rg -n "CURRENT_STATUS|AI_READ_ME|TECH_CONTEXT|README_AI|SSOT|status\.json|trace-index|handoff|HANDOFF|decisions|LOCKS|glossary|context pack" docs -S
docs/audits/docs-ai-pack-scan.md:66:docs/decisions/decisions.md:6:- 2026-02-17: contracts additive-only (see contracts policy SSOT where applicable) / Prevents breaking downstream consumers / Contract evolution must preserve compatibility.
docs/audits/docs-ai-pack-scan.md:67:docs/decisions/decisions.md:7:- 2026-02-17: docs canonical入口 is `docs/context/README_AI.md` / Single entrypoint prevents SSOT ambiguity / AI and human operators start from one canonical path.
docs/audits/docs-ai-pack-scan.md:68:docs/decisions/decisions.md:8:- 2026-02-17: stopping requires `docs/handoff/HANDOFF.json` update / Multi-agent and interrupted work need explicit continuity state / No stop/handoff without handoff state refresh.
docs/audits/docs-ai-pack-scan.md:70:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/audits/docs-ai-pack-scan.md:73:docs/README.md:6:- AI SSOT entry (canonical docs development OS): [`SSOT/README_AI.md`](SSOT/README_AI.md)
docs/audits/docs-ai-pack-scan.md:74:docs/README.md:13:- Rules SSOT (task generation + parallel safety): [`rules/task-generation-policy.md`](rules/task-generation-policy.md), [`rules/parallel-development-safety.md`](rules/parallel-development-safety.md)
docs/audits/docs-ai-pack-scan.md:75:docs/README.md:14:- AI operator onboarding SSOT: [`SSOT/README_AI.md`](SSOT/README_AI.md)
docs/audits/docs-ai-pack-scan.md:78:docs/specs/crosscut/parallel_task_safety_spec.md:8:- `docs/rules/task-generation-policy.md` (task generation / Multi-AI / Credit-out / HANDOFF / status / decisions / trace)
docs/audits/docs-ai-pack-scan.md:79:docs/specs/crosscut/parallel_task_safety_spec.md:12:- `docs/context/README_AI.md`
docs/audits/docs-ai-pack-scan.md:80:docs/status/trace-index.md:1:# Trace Index (SSOT)
docs/audits/docs-ai-pack-scan.md:81:docs/status/trace-index.json:16:        "docs/context/README_AI.md",
docs/audits/docs-ai-pack-scan.md:82:docs/status/trace-index.json:17:        "docs/status/status.json",
docs/audits/docs-ai-pack-scan.md:83:docs/status/trace-index.json:18:        "docs/handoff/HANDOFF.json",
docs/audits/docs-ai-pack-scan.md:84:docs/status/trace-index.json:19:        "docs/decisions/decisions.md"
docs/audits/docs-ai-pack-scan.md:87:docs/status/decisions.md:1:# Decisions Log (SSOT)
docs/audits/docs-ai-pack-scan.md:88:docs/context/README_AI.md:1:# README_AI (SSOT entrypoint for operators)
docs/audits/docs-ai-pack-scan.md:89:docs/context/README_AI.md:7:1. This file: `docs/context/README_AI.md`
docs/audits/docs-ai-pack-scan.md:90:docs/context/README_AI.md:8:2. Runtime status: `docs/status/status.json`
docs/audits/docs-ai-pack-scan.md:91:docs/context/README_AI.md:9:3. Handoff state: `docs/status/HANDOFF.json`
docs/audits/docs-ai-pack-scan.md:92:docs/context/README_AI.md:10:4. Decision baseline: `docs/status/decisions.md`
docs/audits/docs-ai-pack-scan.md:93:docs/context/README_AI.md:11:5. Rules SSOT:
docs/audits/docs-ai-pack-scan.md:94:docs/context/README_AI.md:17:- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/status/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
docs/audits/docs-ai-pack-scan.md:95:docs/context/README_AI.md:18:- **Single active task:** only one active task is tracked in `docs/status/status.json`.
docs/audits/docs-ai-pack-scan.md:96:docs/context/README_AI.md:19:- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
docs/audits/docs-ai-pack-scan.md:97:docs/context/README_AI.md:20:- **LOCK areas:** lock-sensitive areas include contracts/ci/infra/lockfiles and shared-docs (`docs/context/**`, `docs/rules/**`, docs hubs). Declare `Required Locks` in task cards.
docs/audits/docs-ai-pack-scan.md:98:docs/context/README_AI.md:21:- **Trace SSOT:** `docs/status/trace-index.md` is the canonical place for PR/commit/evidence links.
docs/audits/docs-ai-pack-scan.md:99:docs/context/README_AI.md:22:- **Decisions are binding:** use `docs/status/decisions.md` to fix assumptions and supersede only via explicit new decision entries.
docs/audits/docs-ai-pack-scan.md:101:docs/rules/task-generation-policy.md:9:1. `docs/context/README_AI.md`
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
docs/audits/docs-ai-pack-scan.md:117:docs/handoff/HANDOFF.json:8:    "On next docs task, update status.json and trace-index.json first.",
docs/audits/docs-ai-pack-scan.md:118:docs/handoff/HANDOFF.json:9:    "If stopping mid-task, update this handoff file before exit."
docs/audits/docs-ai-pack-scan.md:119:docs/handoff/HANDOFF.json:16:    "cat docs/status/status.json",
docs/audits/docs-ai-pack-scan.md:120:docs/handoff/HANDOFF.json:17:    "cat docs/status/trace-index.json"
docs/audits/docs-ai-pack-scan.md:123:docs/audits/docs-os-existing-file-scan.md:51:## Command: rg -n "README_AI|SSOT|handoff|HANDOFF|decision|decisions|status\.json|trace-index|LOCKS|glossary|canonical|North Star" docs -S
docs/audits/docs-ai-pack-scan.md:126:docs/audits/docs-os-existing-file-scan.md:60:docs/specs/crosscut/parallel_task_safety_spec.md:1:# Parallel Task Safety Spec (SSOT)
docs/audits/docs-ai-pack-scan.md:140:docs/audits/docs-os-existing-file-scan.md:96:docs/README.md:12:- Parallel task safety (SSOT for parallel development safety): [`specs/parallel-task-safety.md`](specs/parallel-task-safety.md)
docs/audits/docs-ai-pack-scan.md:142:docs/audits/docs-os-existing-file-scan.md:100:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/audits/docs-ai-pack-scan.md:145:docs/audits/docs-audit-report.md:38:- `docs/specs/crosscut/parallel_task_safety_spec.md` → `rules/spec (parallel safety SSOT)`
docs/audits/docs-ai-pack-scan.md:153:docs/audits/docs-audit-report.md:161:  - **運用原則SSOT:** `docs/specs/crosscut/parallel_task_safety_spec.md`
docs/audits/docs-ai-pack-scan.md:157:docs/decisions/records/docs-rules-unify-decision.md:6:- Reason: existing rule content is concentrated in `docs/specs/crosscut/parallel_task_safety_spec.md`; new required operational topics (Multi-AI / Credit-out / HANDOFF / status / trace / decisions / LOCK policy) should be explicit and discoverable under a dedicated operations-rule path.
docs/audits/docs-ai-pack-scan.md:158:docs/decisions/records/docs-rules-unify-decision.md:14:| Multi-AI / Credit-out / Handoff | `docs/specs/crosscut/parallel_task_safety_spec.md` (partial task-generation section only) | `docs/rules/task-generation-policy.md` | merge + extend | Add stop protocol + mandatory HANDOFF update semantics. |
docs/audits/docs-ai-pack-scan.md:159:docs/decisions/records/docs-rules-unify-decision.md:16:| Decision log policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Standardize decision fixation via `docs/status/decisions.md`. |
docs/audits/docs-ai-pack-scan.md:160:docs/decisions/records/docs-rules-unify-decision.md:17:| Traceability policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Set `docs/status/trace-index.md` as trace SSOT, avoid scattered links. |
docs/audits/docs-ai-pack-scan.md:161:docs/decisions/records/docs-rules-unify-decision.md:22:- `docs/context/README_AI.md` is created/updated as must-read entrypoint for AI operators.
docs/audits/docs-ai-pack-scan.md:163:docs/decisions/records/docs-os-consolidation-decision.md:5:| SSOT entry for Docs Development OS | `docs/README.md` (general docs index), `docs/specs/crosscut/parallel_task_safety_spec.md` (parallel safety SSOT note), `docs/audits/docs-content-overlap.md` (canonicalization guidance) | `docs/context/README_AI.md` | **Merge** | Existing docs already describe partial SSOT/canonical concepts, but no single AI-first entrypoint exists. Create one canonical entrance and link all SSOT roles there. |
docs/audits/docs-ai-pack-scan.md:164:docs/decisions/records/docs-os-consolidation-decision.md:6:| Machine-readable current status | No direct candidate found in `docs/**` (`status.json` not present) | `docs/status/status.json` | **Keep (new canonical)** | Add a dedicated structured status file for active epic/task, locks, next actions, and PR state. |
docs/audits/docs-ai-pack-scan.md:165:docs/decisions/records/docs-os-consolidation-decision.md:7:| Handoff protocol | No direct candidate found in `docs/**` (`HANDOFF.json` not present) | `docs/handoff/HANDOFF.json` | **Keep (new canonical)** | Add explicit stop/credit-out handoff file with required fields for multi-AI continuity. |
docs/audits/docs-ai-pack-scan.md:166:docs/decisions/records/docs-os-consolidation-decision.md:8:| Decision log | No direct candidate found in `docs/**` (`docs/decisions/*.md` absent). Related mentions in `docs/changelog.md` and workplan notes. | `docs/decisions/decisions.md` | **Merge** | Keep changelog/workplan for their original purpose, and establish one decision log canonical path for short decision entries. |
docs/audits/docs-ai-pack-scan.md:167:docs/decisions/records/docs-os-consolidation-decision.md:9:| Traceability index | No direct candidate found in `docs/**` (`trace-index.json` absent) | `docs/status/trace-index.json` | **Keep (new canonical)** | Introduce one machine-readable trace SSOT for req↔progress↔PR/commit/spec/runbook/verification links. |
docs/audits/docs-ai-pack-scan.md:168:docs/decisions/records/docs-os-consolidation-decision.md:12:- No existing same-role canonical files were found for `status.json`, `HANDOFF.json`, `decisions.md`, or `trace-index.json`, so no stub replacement was required.
docs/audits/docs-ai-pack-scan.md:169:docs/decisions/records/docs-os-consolidation-decision.md:13:- `docs/README.md` remains a general docs hub; `docs/context/README_AI.md` becomes the canonical AI operating entrypoint and references role-specific SSOT paths.
docs/audits/docs-ai-pack-scan.md:177:docs/audits/docs-rules-unify-scan.md:52:## 3) `rg -n "parallel-task-safety|タスク生成|task generation|Codexアジャイル|致命的事故|guardrail|LOCK:|handoff|HANDOFF|status\.json|trace-index|decisions|SSOT|README_AI" docs -S`
docs/audits/docs-ai-pack-scan.md:181:docs/audits/docs-rules-unify-scan.md:58:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/audits/docs-ai-pack-scan.md:196:docs/audits/docs-rules-unify-scan.md:75:docs/README.md:12:- Parallel task safety (SSOT for parallel development safety): [`specs/parallel-task-safety.md`](specs/parallel-task-safety.md)
docs/audits/docs-ai-pack-scan.md:197:docs/audits/docs-rules-unify-scan.md:76:docs/specs/crosscut/parallel_task_safety_spec.md:1:# Parallel Task Safety Spec (SSOT)
docs/audits/docs-ai-pack-scan.md:203:docs/context/README_AI.md
docs/audits/docs-ai-pack-scan.md:204:docs/status/status.json
docs/audits/docs-rules-unify-scan.md:32:docs/specs/crosscut/parallel_task_safety_spec.md
docs/audits/docs-rules-unify-scan.md:52:## 3) `rg -n "parallel-task-safety|タスク生成|task generation|Codexアジャイル|致命的事故|guardrail|LOCK:|handoff|HANDOFF|status\.json|trace-index|decisions|SSOT|README_AI" docs -S`
docs/audits/docs-rules-unify-scan.md:58:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/audits/docs-rules-unify-scan.md:75:docs/README.md:12:- Parallel task safety (SSOT for parallel development safety): [`specs/parallel-task-safety.md`](specs/parallel-task-safety.md)
docs/audits/docs-rules-unify-scan.md:76:docs/specs/crosscut/parallel_task_safety_spec.md:1:# Parallel Task Safety Spec (SSOT)
docs/audits/docs-rules-unify-scan.md:77:docs/specs/crosscut/parallel_task_safety_spec.md:45:## 4. Profinaut開発：Codexアジャイル「タスク生成」方針
docs/audits/docs-rules-unify-scan.md:94:docs/specs/crosscut/parallel_task_safety_spec.md
docs/audits/docs-ai-prompts-scan.md:12:docs/context/README_AI.md
docs/audits/docs-ai-prompts-scan.md:26:docs/decisions/decisions.md
docs/audits/docs-ai-prompts-scan.md:27:docs/handoff/HANDOFF.json
docs/audits/docs-ai-prompts-scan.md:42:docs/specs/crosscut/parallel_task_safety_spec.md
docs/audits/docs-ai-prompts-scan.md:47:docs/status/HANDOFF.json
docs/audits/docs-ai-prompts-scan.md:48:docs/status/decisions.md
docs/audits/docs-ai-prompts-scan.md:58:docs/status/status.json
docs/audits/docs-ai-prompts-scan.md:59:docs/status/trace-index.json
docs/audits/docs-ai-prompts-scan.md:60:docs/status/trace-index.md
docs/audits/docs-ai-prompts-scan.md:68:## Command: rg -n "magic prompt|プロンプト|prompt template|Planner|Builder|Verifier|Reviewer|Copilot|Claude|Gemini|ChatGPT|README_AI|SSOT|handoff|HANDOFF|status\.json|trace-index|LOCK" docs -S
docs/audits/docs-ai-prompts-scan.md:70:docs/decisions/decisions.md:6:- 2026-02-17: contracts additive-only (see contracts policy SSOT where applicable) / Prevents breaking downstream consumers / Contract evolution must preserve compatibility.
docs/audits/docs-ai-prompts-scan.md:71:docs/decisions/decisions.md:7:- 2026-02-17: docs canonical入口 is `docs/context/README_AI.md` / Single entrypoint prevents SSOT ambiguity / AI and human operators start from one canonical path.
docs/audits/docs-ai-prompts-scan.md:72:docs/decisions/decisions.md:8:- 2026-02-17: stopping requires `docs/handoff/HANDOFF.json` update / Multi-agent and interrupted work need explicit continuity state / No stop/handoff without handoff state refresh.
docs/audits/docs-ai-prompts-scan.md:79:docs/README.md:6:- AI SSOT entry (canonical docs development OS): [`SSOT/README_AI.md`](SSOT/README_AI.md)
docs/audits/docs-ai-prompts-scan.md:80:docs/README.md:13:- Rules SSOT (task generation + parallel safety): [`rules/task-generation-policy.md`](rules/task-generation-policy.md), [`rules/parallel-development-safety.md`](rules/parallel-development-safety.md)
docs/audits/docs-ai-prompts-scan.md:81:docs/README.md:14:- AI operator onboarding SSOT: [`SSOT/README_AI.md`](SSOT/README_AI.md)
docs/audits/docs-ai-prompts-scan.md:84:docs/specs/crosscut/parallel_task_safety_spec.md:8:- `docs/rules/task-generation-policy.md` (task generation / Multi-AI / Credit-out / HANDOFF / status / decisions / trace)
docs/audits/docs-ai-prompts-scan.md:85:docs/specs/crosscut/parallel_task_safety_spec.md:12:- `docs/context/README_AI.md`
docs/audits/docs-ai-prompts-scan.md:86:docs/status/trace-index.md:1:# Trace Index (SSOT)
docs/audits/docs-ai-prompts-scan.md:87:docs/status/trace-index.json:16:        "docs/context/README_AI.md",
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
docs/audits/docs-ai-prompts-scan.md:111:docs/rules/task-generation-policy.md:9:1. `docs/context/README_AI.md`
docs/audits/docs-ai-prompts-scan.md:112:docs/rules/task-generation-policy.md:10:2. `docs/status/status.json`
docs/audits/docs-ai-prompts-scan.md:113:docs/rules/task-generation-policy.md:11:3. `docs/status/HANDOFF.json`
docs/audits/docs-ai-prompts-scan.md:114:docs/rules/task-generation-policy.md:19:- Active task ownership and lifecycle are tracked in `docs/status/status.json`.
docs/audits/docs-ai-prompts-scan.md:116:docs/rules/task-generation-policy.md:45:- Canonical runtime status must be updated through `docs/status/status.json` only.
docs/audits/docs-ai-prompts-scan.md:118:docs/rules/task-generation-policy.md:52:### 4.3 Mandatory HANDOFF update on stop/switch
docs/audits/docs-ai-prompts-scan.md:119:docs/rules/task-generation-policy.md:54:Before stopping or switching owner, update `docs/status/HANDOFF.json` with all of:
docs/audits/docs-ai-prompts-scan.md:120:docs/rules/task-generation-policy.md:66:  - an update to `docs/status/status.json`, **or**
docs/audits/docs-ai-prompts-scan.md:121:docs/rules/task-generation-policy.md:78:- `docs/status/trace-index.md` is the SSOT for trace links (PRs, commits, issues, run artifacts).
docs/audits/docs-ai-prompts-scan.md:122:docs/rules/task-generation-policy.md:80:- Other docs may include convenience links, but must treat trace-index as canonical.
docs/audits/docs-ai-prompts-scan.md:131:docs/context/README_AI.md:1:# README_AI (SSOT entrypoint for operators)
docs/audits/docs-ai-prompts-scan.md:132:docs/context/README_AI.md:7:1. This file: `docs/context/README_AI.md` (AI constitution / entrypoint)
docs/audits/docs-ai-prompts-scan.md:133:docs/context/README_AI.md:8:2. Human-readable status snapshot (non-SSOT): `docs/status/CURRENT_STATUS.md`
docs/audits/docs-ai-prompts-scan.md:134:docs/context/README_AI.md:9:3. Runtime status SSOT: `docs/status/status.json`
docs/audits/docs-ai-prompts-scan.md:135:docs/context/README_AI.md:10:4. Tech context hub (non-SSOT links-only): `docs/context/TECH_CONTEXT.md`
docs/audits/docs-ai-prompts-scan.md:136:docs/context/README_AI.md:11:5. Handoff state: `docs/status/HANDOFF.json`
docs/audits/docs-ai-prompts-scan.md:137:docs/context/README_AI.md:13:7. Trace SSOT: `docs/status/trace-index.md`
docs/audits/docs-ai-prompts-scan.md:138:docs/context/README_AI.md:14:8. Rules SSOT:
docs/audits/docs-ai-prompts-scan.md:139:docs/context/README_AI.md:19:## SSOT boundaries (important)
docs/audits/docs-ai-prompts-scan.md:140:docs/context/README_AI.md:21:- **Canonical AI entrypoint:** `docs/context/README_AI.md`
docs/audits/docs-ai-prompts-scan.md:141:docs/context/README_AI.md:22:- **Status SSOT:** `docs/status/status.json`
docs/audits/docs-ai-prompts-scan.md:142:docs/context/README_AI.md:23:- **`docs/status/CURRENT_STATUS.md` is a summary only (non-SSOT).**
docs/audits/docs-ai-prompts-scan.md:143:docs/context/README_AI.md:24:- **`docs/context/TECH_CONTEXT.md` is a links hub only (non-SSOT).**
docs/audits/docs-ai-prompts-scan.md:144:docs/context/README_AI.md:29:- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/status/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
docs/audits/docs-ai-prompts-scan.md:145:docs/context/README_AI.md:30:- **Single active task:** only one active task is tracked in `docs/status/status.json`.
docs/audits/docs-ai-prompts-scan.md:146:docs/context/README_AI.md:31:- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
docs/audits/docs-ai-prompts-scan.md:147:docs/context/README_AI.md:32:- **LOCK areas:** lock-sensitive areas include contracts/ci/infra/lockfiles and shared-docs (`docs/context/**`, `docs/rules/**`, docs hubs). Declare `Required Locks` in task cards.
docs/audits/docs-ai-prompts-scan.md:148:docs/context/README_AI.md:33:- **Trace SSOT:** `docs/status/trace-index.md` is the canonical place for PR/commit/evidence links.
docs/audits/docs-ai-prompts-scan.md:150:docs/context/TECH_CONTEXT.md:10:- AI entrypoint: `docs/context/README_AI.md`
docs/audits/docs-ai-prompts-scan.md:151:docs/context/TECH_CONTEXT.md:12:- Runtime status SSOT: `docs/status/status.json`
docs/audits/docs-ai-prompts-scan.md:152:docs/context/TECH_CONTEXT.md:13:- Trace index: `docs/status/trace-index.md` and `docs/status/trace-index.json`
docs/audits/docs-ai-prompts-scan.md:153:docs/context/TECH_CONTEXT.md:15:- Handoff state: `docs/status/HANDOFF.json` (legacy/alt path may exist at `docs/handoff/HANDOFF.json`)
docs/audits/docs-ai-prompts-scan.md:154:docs/context/TECH_CONTEXT.md:42:- When in doubt, follow linked SSOT/authoritative docs and record decisions in `docs/status/decisions.md`.
docs/audits/docs-ai-prompts-scan.md:155:docs/handoff/HANDOFF.json:8:    "On next docs task, update status.json and trace-index.json first.",
docs/audits/docs-ai-prompts-scan.md:156:docs/handoff/HANDOFF.json:9:    "If stopping mid-task, update this handoff file before exit."
docs/audits/docs-ai-prompts-scan.md:157:docs/handoff/HANDOFF.json:16:    "cat docs/status/status.json",
docs/audits/docs-ai-prompts-scan.md:158:docs/handoff/HANDOFF.json:17:    "cat docs/status/trace-index.json"
docs/audits/docs-ai-prompts-scan.md:161:docs/audits/docs-os-existing-file-scan.md:51:## Command: rg -n "README_AI|SSOT|handoff|HANDOFF|decision|decisions|status\.json|trace-index|LOCKS|glossary|canonical|North Star" docs -S
docs/audits/docs-ai-prompts-scan.md:164:docs/audits/docs-os-existing-file-scan.md:60:docs/specs/crosscut/parallel_task_safety_spec.md:1:# Parallel Task Safety Spec (SSOT)
docs/audits/docs-ai-prompts-scan.md:180:docs/audits/docs-os-existing-file-scan.md:96:docs/README.md:12:- Parallel task safety (SSOT for parallel development safety): [`specs/parallel-task-safety.md`](specs/parallel-task-safety.md)
docs/audits/docs-ai-prompts-scan.md:184:docs/audits/docs-audit-report.md:38:- `docs/specs/crosscut/parallel_task_safety_spec.md` → `rules/spec (parallel safety SSOT)`
docs/audits/docs-ai-prompts-scan.md:192:docs/audits/docs-audit-report.md:161:  - **運用原則SSOT:** `docs/specs/crosscut/parallel_task_safety_spec.md`
docs/audits/docs-ai-prompts-scan.md:196:docs/decisions/records/docs-rules-unify-decision.md:6:- Reason: existing rule content is concentrated in `docs/specs/crosscut/parallel_task_safety_spec.md`; new required operational topics (Multi-AI / Credit-out / HANDOFF / status / trace / decisions / LOCK policy) should be explicit and discoverable under a dedicated operations-rule path.
docs/audits/docs-ai-prompts-scan.md:197:docs/decisions/records/docs-rules-unify-decision.md:14:| Multi-AI / Credit-out / Handoff | `docs/specs/crosscut/parallel_task_safety_spec.md` (partial task-generation section only) | `docs/rules/task-generation-policy.md` | merge + extend | Add stop protocol + mandatory HANDOFF update semantics. |
docs/audits/docs-ai-prompts-scan.md:198:docs/decisions/records/docs-rules-unify-decision.md:15:| LOCK policy | `docs/specs/crosscut/parallel_task_safety_spec.md` (LOCK/semi-LOCK partial) | `docs/specs/crosscut/safety_interlock_spec.md` | merge + extend | Add explicit lock areas incl. shared-docs and Required Locks declaration. |
docs/audits/docs-ai-prompts-scan.md:199:docs/decisions/records/docs-rules-unify-decision.md:16:| Decision log policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Standardize decision fixation via `docs/status/decisions.md`. |
docs/audits/docs-ai-prompts-scan.md:200:docs/decisions/records/docs-rules-unify-decision.md:17:| Traceability policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Set `docs/status/trace-index.md` as trace SSOT, avoid scattered links. |
docs/audits/docs-ai-prompts-scan.md:201:docs/decisions/records/docs-rules-unify-decision.md:22:- `docs/context/README_AI.md` is created/updated as must-read entrypoint for AI operators.
docs/audits/docs-ai-prompts-scan.md:202:docs/decisions/records/docs-ai-pack-decision.md:5:| AI entry (constitution) | `docs/context/README_AI.md` | `docs/context/README_AI.md` | keep/merge | Must remain canonical AI entrypoint. Add links to new non-SSOT context files. |
docs/audits/docs-ai-prompts-scan.md:203:docs/decisions/records/docs-ai-pack-decision.md:6:| Human-readable current status | `docs/status/status.json` (machine-readable SSOT), no existing markdown summary found | `docs/status/CURRENT_STATUS.md` | create/merge | New file is a thin, human-readable summary only. Explicitly states SSOT is `docs/status/status.json`. |
docs/audits/docs-ai-prompts-scan.md:206:docs/decisions/records/docs-ai-pack-decision.md:12:- Keep `docs/context/README_AI.md` as the single AI constitution entrypoint.
docs/audits/docs-ai-prompts-scan.md:207:docs/decisions/records/docs-ai-pack-decision.md:13:- Treat `docs/status/CURRENT_STATUS.md` as non-SSOT summary derived from `docs/status/status.json`.
docs/audits/docs-ai-prompts-scan.md:211:docs/decisions/records/docs-os-consolidation-decision.md:5:| SSOT entry for Docs Development OS | `docs/README.md` (general docs index), `docs/specs/crosscut/parallel_task_safety_spec.md` (parallel safety SSOT note), `docs/audits/docs-content-overlap.md` (canonicalization guidance) | `docs/context/README_AI.md` | **Merge** | Existing docs already describe partial SSOT/canonical concepts, but no single AI-first entrypoint exists. Create one canonical entrance and link all SSOT roles there. |
docs/audits/docs-ai-prompts-scan.md:212:docs/decisions/records/docs-os-consolidation-decision.md:6:| Machine-readable current status | No direct candidate found in `docs/**` (`status.json` not present) | `docs/status/status.json` | **Keep (new canonical)** | Add a dedicated structured status file for active epic/task, locks, next actions, and PR state. |
docs/audits/docs-ai-prompts-scan.md:213:docs/decisions/records/docs-os-consolidation-decision.md:7:| Handoff protocol | No direct candidate found in `docs/**` (`HANDOFF.json` not present) | `docs/handoff/HANDOFF.json` | **Keep (new canonical)** | Add explicit stop/credit-out handoff file with required fields for multi-AI continuity. |
docs/audits/docs-ai-prompts-scan.md:214:docs/decisions/records/docs-os-consolidation-decision.md:9:| Traceability index | No direct candidate found in `docs/**` (`trace-index.json` absent) | `docs/status/trace-index.json` | **Keep (new canonical)** | Introduce one machine-readable trace SSOT for req↔progress↔PR/commit/spec/runbook/verification links. |
docs/audits/docs-ai-prompts-scan.md:215:docs/decisions/records/docs-os-consolidation-decision.md:12:- No existing same-role canonical files were found for `status.json`, `HANDOFF.json`, `decisions.md`, or `trace-index.json`, so no stub replacement was required.
docs/audits/docs-ai-prompts-scan.md:216:docs/decisions/records/docs-os-consolidation-decision.md:13:- `docs/README.md` remains a general docs hub; `docs/context/README_AI.md` becomes the canonical AI operating entrypoint and references role-specific SSOT paths.
docs/audits/docs-ai-prompts-scan.md:224:docs/audits/docs-ai-pack-scan.md:12:docs/context/README_AI.md
docs/audits/docs-ai-prompts-scan.md:225:docs/audits/docs-ai-pack-scan.md:24:docs/handoff/HANDOFF.json
docs/audits/docs-ai-prompts-scan.md:226:docs/audits/docs-ai-pack-scan.md:43:docs/status/HANDOFF.json
docs/audits/docs-ai-prompts-scan.md:227:docs/audits/docs-ai-pack-scan.md:54:docs/status/status.json
docs/audits/docs-ai-prompts-scan.md:228:docs/audits/docs-ai-pack-scan.md:55:docs/status/trace-index.json
docs/audits/docs-ai-prompts-scan.md:229:docs/audits/docs-ai-pack-scan.md:56:docs/status/trace-index.md
docs/audits/docs-ai-prompts-scan.md:230:docs/audits/docs-ai-pack-scan.md:64:## Command: rg -n "CURRENT_STATUS|AI_READ_ME|TECH_CONTEXT|README_AI|SSOT|status\.json|trace-index|handoff|HANDOFF|decisions|LOCKS|glossary|context pack" docs -S
docs/audits/docs-ai-prompts-scan.md:231:docs/audits/docs-ai-pack-scan.md:66:docs/decisions/decisions.md:6:- 2026-02-17: contracts additive-only (see contracts policy SSOT where applicable) / Prevents breaking downstream consumers / Contract evolution must preserve compatibility.
docs/audits/docs-ai-prompts-scan.md:232:docs/audits/docs-ai-pack-scan.md:67:docs/decisions/decisions.md:7:- 2026-02-17: docs canonical入口 is `docs/context/README_AI.md` / Single entrypoint prevents SSOT ambiguity / AI and human operators start from one canonical path.
docs/audits/docs-ai-prompts-scan.md:233:docs/audits/docs-ai-pack-scan.md:68:docs/decisions/decisions.md:8:- 2026-02-17: stopping requires `docs/handoff/HANDOFF.json` update / Multi-agent and interrupted work need explicit continuity state / No stop/handoff without handoff state refresh.
docs/audits/docs-ai-prompts-scan.md:237:docs/audits/docs-ai-pack-scan.md:73:docs/README.md:6:- AI SSOT entry (canonical docs development OS): [`SSOT/README_AI.md`](SSOT/README_AI.md)
docs/audits/docs-ai-prompts-scan.md:238:docs/audits/docs-ai-pack-scan.md:74:docs/README.md:13:- Rules SSOT (task generation + parallel safety): [`rules/task-generation-policy.md`](rules/task-generation-policy.md), [`rules/parallel-development-safety.md`](rules/parallel-development-safety.md)
docs/audits/docs-ai-prompts-scan.md:239:docs/audits/docs-ai-pack-scan.md:75:docs/README.md:14:- AI operator onboarding SSOT: [`SSOT/README_AI.md`](SSOT/README_AI.md)
docs/audits/docs-ai-prompts-scan.md:242:docs/audits/docs-ai-pack-scan.md:78:docs/specs/crosscut/parallel_task_safety_spec.md:8:- `docs/rules/task-generation-policy.md` (task generation / Multi-AI / Credit-out / HANDOFF / status / decisions / trace)
docs/audits/docs-ai-prompts-scan.md:243:docs/audits/docs-ai-pack-scan.md:79:docs/specs/crosscut/parallel_task_safety_spec.md:12:- `docs/context/README_AI.md`
docs/audits/docs-ai-prompts-scan.md:244:docs/audits/docs-ai-pack-scan.md:80:docs/status/trace-index.md:1:# Trace Index (SSOT)
docs/audits/docs-ai-prompts-scan.md:245:docs/audits/docs-ai-pack-scan.md:81:docs/status/trace-index.json:16:        "docs/context/README_AI.md",
docs/audits/docs-ai-prompts-scan.md:246:docs/audits/docs-ai-pack-scan.md:82:docs/status/trace-index.json:17:        "docs/status/status.json",
docs/audits/docs-ai-prompts-scan.md:247:docs/audits/docs-ai-pack-scan.md:83:docs/status/trace-index.json:18:        "docs/handoff/HANDOFF.json",
docs/audits/docs-ai-prompts-scan.md:248:docs/audits/docs-ai-pack-scan.md:84:docs/status/trace-index.json:19:        "docs/decisions/decisions.md"
docs/audits/docs-ai-prompts-scan.md:251:docs/audits/docs-ai-pack-scan.md:87:docs/status/decisions.md:1:# Decisions Log (SSOT)
docs/audits/docs-ai-prompts-scan.md:252:docs/audits/docs-ai-pack-scan.md:88:docs/context/README_AI.md:1:# README_AI (SSOT entrypoint for operators)
docs/audits/docs-ai-prompts-scan.md:253:docs/audits/docs-ai-pack-scan.md:89:docs/context/README_AI.md:7:1. This file: `docs/context/README_AI.md`
docs/audits/docs-ai-prompts-scan.md:254:docs/audits/docs-ai-pack-scan.md:90:docs/context/README_AI.md:8:2. Runtime status: `docs/status/status.json`
docs/audits/docs-ai-prompts-scan.md:255:docs/audits/docs-ai-pack-scan.md:91:docs/context/README_AI.md:9:3. Handoff state: `docs/status/HANDOFF.json`
docs/audits/docs-ai-prompts-scan.md:256:docs/audits/docs-ai-pack-scan.md:92:docs/context/README_AI.md:10:4. Decision baseline: `docs/status/decisions.md`
docs/audits/docs-ai-prompts-scan.md:257:docs/audits/docs-ai-pack-scan.md:93:docs/context/README_AI.md:11:5. Rules SSOT:
docs/audits/docs-ai-prompts-scan.md:258:docs/audits/docs-ai-pack-scan.md:94:docs/context/README_AI.md:17:- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/status/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
docs/audits/docs-ai-prompts-scan.md:259:docs/audits/docs-ai-pack-scan.md:95:docs/context/README_AI.md:18:- **Single active task:** only one active task is tracked in `docs/status/status.json`.
docs/audits/docs-ai-prompts-scan.md:260:docs/audits/docs-ai-pack-scan.md:96:docs/context/README_AI.md:19:- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
docs/audits/docs-ai-prompts-scan.md:261:docs/audits/docs-ai-pack-scan.md:97:docs/context/README_AI.md:20:- **LOCK areas:** lock-sensitive areas include contracts/ci/infra/lockfiles and shared-docs (`docs/context/**`, `docs/rules/**`, docs hubs). Declare `Required Locks` in task cards.
docs/audits/docs-ai-prompts-scan.md:262:docs/audits/docs-ai-pack-scan.md:98:docs/context/README_AI.md:21:- **Trace SSOT:** `docs/status/trace-index.md` is the canonical place for PR/commit/evidence links.
docs/audits/docs-ai-prompts-scan.md:263:docs/audits/docs-ai-pack-scan.md:99:docs/context/README_AI.md:22:- **Decisions are binding:** use `docs/status/decisions.md` to fix assumptions and supersede only via explicit new decision entries.
docs/audits/docs-ai-prompts-scan.md:265:docs/audits/docs-ai-pack-scan.md:101:docs/rules/task-generation-policy.md:9:1. `docs/context/README_AI.md`
docs/audits/docs-ai-prompts-scan.md:266:docs/audits/docs-ai-pack-scan.md:102:docs/rules/task-generation-policy.md:10:2. `docs/status/status.json`
docs/audits/docs-ai-prompts-scan.md:267:docs/audits/docs-ai-pack-scan.md:103:docs/rules/task-generation-policy.md:11:3. `docs/status/HANDOFF.json`
docs/audits/docs-ai-prompts-scan.md:268:docs/audits/docs-ai-pack-scan.md:105:docs/rules/task-generation-policy.md:19:- Active task ownership and lifecycle are tracked in `docs/status/status.json`.
docs/audits/docs-ai-prompts-scan.md:269:docs/audits/docs-ai-pack-scan.md:106:docs/rules/task-generation-policy.md:45:- Canonical runtime status must be updated through `docs/status/status.json` only.
docs/audits/docs-ai-prompts-scan.md:271:docs/audits/docs-ai-pack-scan.md:108:docs/rules/task-generation-policy.md:52:### 4.3 Mandatory HANDOFF update on stop/switch
docs/audits/docs-ai-prompts-scan.md:272:docs/audits/docs-ai-pack-scan.md:109:docs/rules/task-generation-policy.md:54:Before stopping or switching owner, update `docs/status/HANDOFF.json` with all of:
docs/audits/docs-ai-prompts-scan.md:273:docs/audits/docs-ai-pack-scan.md:110:docs/rules/task-generation-policy.md:66:  - an update to `docs/status/status.json`, **or**
docs/audits/docs-ai-prompts-scan.md:274:docs/audits/docs-ai-pack-scan.md:112:docs/rules/task-generation-policy.md:78:- `docs/status/trace-index.md` is the SSOT for trace links (PRs, commits, issues, run artifacts).
docs/audits/docs-ai-prompts-scan.md:275:docs/audits/docs-ai-pack-scan.md:113:docs/rules/task-generation-policy.md:80:- Other docs may include convenience links, but must treat trace-index as canonical.
docs/audits/docs-ai-prompts-scan.md:279:docs/audits/docs-ai-pack-scan.md:117:docs/handoff/HANDOFF.json:8:    "On next docs task, update status.json and trace-index.json first.",
docs/audits/docs-ai-prompts-scan.md:280:docs/audits/docs-ai-pack-scan.md:118:docs/handoff/HANDOFF.json:9:    "If stopping mid-task, update this handoff file before exit."
docs/audits/docs-ai-prompts-scan.md:281:docs/audits/docs-ai-pack-scan.md:119:docs/handoff/HANDOFF.json:16:    "cat docs/status/status.json",
docs/audits/docs-ai-prompts-scan.md:282:docs/audits/docs-ai-pack-scan.md:120:docs/handoff/HANDOFF.json:17:    "cat docs/status/trace-index.json"
docs/audits/docs-ai-prompts-scan.md:285:docs/audits/docs-ai-pack-scan.md:123:docs/audits/docs-os-existing-file-scan.md:51:## Command: rg -n "README_AI|SSOT|handoff|HANDOFF|decision|decisions|status\.json|trace-index|LOCKS|glossary|canonical|North Star" docs -S
docs/audits/docs-ai-prompts-scan.md:288:docs/audits/docs-ai-pack-scan.md:126:docs/audits/docs-os-existing-file-scan.md:60:docs/specs/crosscut/parallel_task_safety_spec.md:1:# Parallel Task Safety Spec (SSOT)
docs/audits/docs-ai-prompts-scan.md:302:docs/audits/docs-ai-pack-scan.md:140:docs/audits/docs-os-existing-file-scan.md:96:docs/README.md:12:- Parallel task safety (SSOT for parallel development safety): [`specs/parallel-task-safety.md`](specs/parallel-task-safety.md)
docs/audits/docs-ai-prompts-scan.md:306:docs/audits/docs-ai-pack-scan.md:145:docs/audits/docs-audit-report.md:38:- `docs/specs/crosscut/parallel_task_safety_spec.md` → `rules/spec (parallel safety SSOT)`
docs/audits/docs-ai-prompts-scan.md:314:docs/audits/docs-ai-pack-scan.md:153:docs/audits/docs-audit-report.md:161:  - **運用原則SSOT:** `docs/specs/crosscut/parallel_task_safety_spec.md`
docs/audits/docs-ai-prompts-scan.md:318:docs/audits/docs-ai-pack-scan.md:157:docs/decisions/records/docs-rules-unify-decision.md:6:- Reason: existing rule content is concentrated in `docs/specs/crosscut/parallel_task_safety_spec.md`; new required operational topics (Multi-AI / Credit-out / HANDOFF / status / trace / decisions / LOCK policy) should be explicit and discoverable under a dedicated operations-rule path.
docs/audits/docs-ai-prompts-scan.md:319:docs/audits/docs-ai-pack-scan.md:158:docs/decisions/records/docs-rules-unify-decision.md:14:| Multi-AI / Credit-out / Handoff | `docs/specs/crosscut/parallel_task_safety_spec.md` (partial task-generation section only) | `docs/rules/task-generation-policy.md` | merge + extend | Add stop protocol + mandatory HANDOFF update semantics. |
docs/audits/docs-ai-prompts-scan.md:320:docs/audits/docs-ai-pack-scan.md:159:docs/decisions/records/docs-rules-unify-decision.md:16:| Decision log policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Standardize decision fixation via `docs/status/decisions.md`. |
docs/audits/docs-ai-prompts-scan.md:321:docs/audits/docs-ai-pack-scan.md:160:docs/decisions/records/docs-rules-unify-decision.md:17:| Traceability policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Set `docs/status/trace-index.md` as trace SSOT, avoid scattered links. |
docs/audits/docs-ai-prompts-scan.md:322:docs/audits/docs-ai-pack-scan.md:161:docs/decisions/records/docs-rules-unify-decision.md:22:- `docs/context/README_AI.md` is created/updated as must-read entrypoint for AI operators.
docs/audits/docs-ai-prompts-scan.md:324:docs/audits/docs-ai-pack-scan.md:163:docs/decisions/records/docs-os-consolidation-decision.md:5:| SSOT entry for Docs Development OS | `docs/README.md` (general docs index), `docs/specs/crosscut/parallel_task_safety_spec.md` (parallel safety SSOT note), `docs/audits/docs-content-overlap.md` (canonicalization guidance) | `docs/context/README_AI.md` | **Merge** | Existing docs already describe partial SSOT/canonical concepts, but no single AI-first entrypoint exists. Create one canonical entrance and link all SSOT roles there. |
docs/audits/docs-ai-prompts-scan.md:325:docs/audits/docs-ai-pack-scan.md:164:docs/decisions/records/docs-os-consolidation-decision.md:6:| Machine-readable current status | No direct candidate found in `docs/**` (`status.json` not present) | `docs/status/status.json` | **Keep (new canonical)** | Add a dedicated structured status file for active epic/task, locks, next actions, and PR state. |
docs/audits/docs-ai-prompts-scan.md:326:docs/audits/docs-ai-pack-scan.md:165:docs/decisions/records/docs-os-consolidation-decision.md:7:| Handoff protocol | No direct candidate found in `docs/**` (`HANDOFF.json` not present) | `docs/handoff/HANDOFF.json` | **Keep (new canonical)** | Add explicit stop/credit-out handoff file with required fields for multi-AI continuity. |
docs/audits/docs-ai-prompts-scan.md:327:docs/audits/docs-ai-pack-scan.md:167:docs/decisions/records/docs-os-consolidation-decision.md:9:| Traceability index | No direct candidate found in `docs/**` (`trace-index.json` absent) | `docs/status/trace-index.json` | **Keep (new canonical)** | Introduce one machine-readable trace SSOT for req↔progress↔PR/commit/spec/runbook/verification links. |
docs/audits/docs-ai-prompts-scan.md:328:docs/audits/docs-ai-pack-scan.md:168:docs/decisions/records/docs-os-consolidation-decision.md:12:- No existing same-role canonical files were found for `status.json`, `HANDOFF.json`, `decisions.md`, or `trace-index.json`, so no stub replacement was required.
docs/audits/docs-ai-prompts-scan.md:329:docs/audits/docs-ai-pack-scan.md:169:docs/decisions/records/docs-os-consolidation-decision.md:13:- `docs/README.md` remains a general docs hub; `docs/context/README_AI.md` becomes the canonical AI operating entrypoint and references role-specific SSOT paths.
docs/audits/docs-ai-prompts-scan.md:337:docs/audits/docs-ai-pack-scan.md:177:docs/audits/docs-rules-unify-scan.md:52:## 3) `rg -n "parallel-task-safety|タスク生成|task generation|Codexアジャイル|致命的事故|guardrail|LOCK:|handoff|HANDOFF|status\.json|trace-index|decisions|SSOT|README_AI" docs -S`
docs/audits/docs-ai-prompts-scan.md:355:docs/audits/docs-ai-pack-scan.md:196:docs/audits/docs-rules-unify-scan.md:75:docs/README.md:12:- Parallel task safety (SSOT for parallel development safety): [`specs/parallel-task-safety.md`](specs/parallel-task-safety.md)
docs/audits/docs-ai-prompts-scan.md:356:docs/audits/docs-ai-pack-scan.md:197:docs/audits/docs-rules-unify-scan.md:76:docs/specs/crosscut/parallel_task_safety_spec.md:1:# Parallel Task Safety Spec (SSOT)
docs/audits/docs-ai-prompts-scan.md:358:docs/audits/docs-ai-pack-scan.md:203:docs/context/README_AI.md
docs/audits/docs-ai-prompts-scan.md:359:docs/audits/docs-ai-pack-scan.md:204:docs/status/status.json
docs/audits/docs-ai-prompts-scan.md:360:docs/audits/docs-rules-unify-scan.md:52:## 3) `rg -n "parallel-task-safety|タスク生成|task generation|Codexアジャイル|致命的事故|guardrail|LOCK:|handoff|HANDOFF|status\.json|trace-index|decisions|SSOT|README_AI" docs -S`
docs/audits/docs-ai-prompts-scan.md:378:docs/audits/docs-rules-unify-scan.md:75:docs/README.md:12:- Parallel task safety (SSOT for parallel development safety): [`specs/parallel-task-safety.md`](specs/parallel-task-safety.md)
docs/audits/docs-ai-prompts-scan.md:379:docs/audits/docs-rules-unify-scan.md:76:docs/specs/crosscut/parallel_task_safety_spec.md:1:# Parallel Task Safety Spec (SSOT)
docs/audits/docs-ai-checklist-scan.md:13:docs/context/README_AI.md
docs/audits/docs-ai-checklist-scan.md:29:docs/decisions/decisions.md
docs/audits/docs-ai-checklist-scan.md:30:docs/handoff/HANDOFF.json
docs/audits/docs-ai-checklist-scan.md:45:docs/specs/crosscut/parallel_task_safety_spec.md
docs/audits/docs-ai-checklist-scan.md:50:docs/status/HANDOFF.json
docs/audits/docs-ai-checklist-scan.md:51:docs/status/decisions.md
docs/audits/docs-ai-checklist-scan.md:61:docs/status/status.json
docs/audits/docs-ai-checklist-scan.md:62:docs/status/trace-index.json
docs/audits/docs-ai-checklist-scan.md:63:docs/status/trace-index.md
docs/audits/docs-ai-checklist-scan.md:71:## Command: rg -n "checklist|preflight|merge|push|rollback|rebase|conflict|LOCK:|Required Locks|Allowed paths|Forbidden paths|CI|up-to-date|Handoff|HANDOFF|status\.json|trace-index|decision" docs -S
docs/audits/docs-ai-checklist-scan.md:73:docs/decisions/decisions.md:8:- 2026-02-17: stopping requires `docs/handoff/HANDOFF.json` update / Multi-agent and interrupted work need explicit continuity state / No stop/handoff without handoff state refresh.
docs/audits/docs-ai-checklist-scan.md:81:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/audits/docs-ai-checklist-scan.md:99:docs/specs/crosscut/parallel_task_safety_spec.md:8:- `docs/rules/task-generation-policy.md` (task generation / Multi-AI / Credit-out / HANDOFF / status / decisions / trace)
docs/audits/docs-ai-checklist-scan.md:101:docs/status/trace-index.json:15:        "docs/decisions/records/docs-os-consolidation-decision.md",
docs/audits/docs-ai-checklist-scan.md:102:docs/status/trace-index.json:17:        "docs/status/status.json",
docs/audits/docs-ai-checklist-scan.md:103:docs/status/trace-index.json:18:        "docs/handoff/HANDOFF.json",
docs/audits/docs-ai-checklist-scan.md:104:docs/status/trace-index.json:19:        "docs/decisions/decisions.md"
docs/audits/docs-ai-checklist-scan.md:110:docs/status/decisions.md:3:Record decision entries here in append-only format.
docs/audits/docs-ai-checklist-scan.md:111:docs/status/CURRENT_STATUS.md:3:_Last updated: 1970-01-01T00:00:00Z (from `docs/status/status.json`)_
docs/audits/docs-ai-checklist-scan.md:112:docs/status/CURRENT_STATUS.md:6:> **SSOT is `docs/status/status.json`**.
docs/audits/docs-ai-checklist-scan.md:113:docs/status/CURRENT_STATUS.md:11:- Active epic: _not specified in `status.json`_
docs/audits/docs-ai-checklist-scan.md:114:docs/status/CURRENT_STATUS.md:17:- _No open PRs recorded in `status.json`._
docs/audits/docs-ai-checklist-scan.md:115:docs/status/CURRENT_STATUS.md:21:- _No locks recorded in `status.json`._
docs/audits/docs-ai-checklist-scan.md:116:docs/status/CURRENT_STATUS.md:25:- _None explicitly recorded in `status.json`._
docs/audits/docs-ai-checklist-scan.md:117:docs/status/CURRENT_STATUS.md:29:- Update `docs/status/status.json` when a task owner is assigned.
docs/audits/docs-ai-checklist-scan.md:118:docs/status/CURRENT_STATUS.md:31:- Add progress evidence under `docs/status/progress-updates/` and trace links in `docs/status/trace-index.md`/`.json` as work proceeds.
docs/audits/docs-ai-checklist-scan.md:119:docs/status/CURRENT_STATUS.md:36:- `docs/status/status.json` (machine-readable SSOT)
docs/audits/docs-ai-checklist-scan.md:129:docs/context/AI_PROMPTS.md:7:- Status SSOT: `docs/status/status.json`
docs/audits/docs-ai-checklist-scan.md:130:docs/context/AI_PROMPTS.md:8:- Trace SSOT: `docs/status/trace-index.json`
docs/audits/docs-ai-checklist-scan.md:131:docs/context/AI_PROMPTS.md:16:- Read in strict order: `docs/context/README_AI.md` → `docs/status/status.json` → `docs/handoff/HANDOFF.json` (or `docs/status/HANDOFF.json`) → `docs/status/decisions.md`.
docs/audits/docs-ai-checklist-scan.md:132:docs/context/AI_PROMPTS.md:17:- Before planning or editing, inspect `docs/status/status.json` keys: `active_task`, `open_prs`, `locks_held`, `owner`, `state`, `last_updated`.
docs/audits/docs-ai-checklist-scan.md:136:docs/context/AI_PROMPTS.md:21:- If pausing/stopping/credit-out before completion, update `docs/handoff/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next` (and sync `docs/status/HANDOFF.json` if that path is being used by current workflow).
docs/audits/docs-ai-checklist-scan.md:137:docs/context/AI_PROMPTS.md:22:- Any "progress made" claim requires evidence updates in `docs/status/status.json` and/or `docs/status/progress-updates/*`.
docs/audits/docs-ai-checklist-scan.md:138:docs/context/AI_PROMPTS.md:23:- Record evidence links in `docs/status/trace-index.json` (SSOT) and mirror to markdown index if required.
docs/audits/docs-ai-checklist-scan.md:139:docs/context/AI_PROMPTS.md:24:- If uncertain, add/update assumptions or decisions via the existing mechanism (`docs/assumptions.md`, `docs/status/decisions.md`) before irreversible changes.
docs/audits/docs-ai-checklist-scan.md:140:docs/context/AI_PROMPTS.md:32:- Read order: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
docs/audits/docs-ai-checklist-scan.md:143:docs/context/AI_PROMPTS.md:37:- If assumptions are needed, record them in assumptions/decisions flow before execution.
docs/audits/docs-ai-checklist-scan.md:144:docs/context/AI_PROMPTS.md:38:- Do not claim progress unless `status.json`/progress-updates are updated.
docs/audits/docs-ai-checklist-scan.md:145:docs/context/AI_PROMPTS.md:42:- Read order: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
docs/audits/docs-ai-checklist-scan.md:148:docs/context/AI_PROMPTS.md:47:- On stop/credit-out, update HANDOFF required fields before exit.
docs/audits/docs-ai-checklist-scan.md:149:docs/context/AI_PROMPTS.md:48:- Any progress statement must be backed by `status.json` or progress update entries.
docs/audits/docs-ai-checklist-scan.md:150:docs/context/AI_PROMPTS.md:52:- Read order: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
docs/audits/docs-ai-checklist-scan.md:151:docs/context/AI_PROMPTS.md:55:- If evidence is missing from `status.json`/progress updates/trace index, fail verification.
docs/audits/docs-ai-checklist-scan.md:152:docs/context/AI_PROMPTS.md:56:- If uncertain behavior exists, request assumptions/decisions update before pass.
docs/audits/docs-ai-checklist-scan.md:153:docs/context/AI_PROMPTS.md:61:- Read order: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
docs/audits/docs-ai-checklist-scan.md:156:docs/context/AI_PROMPTS.md:65:- Verify progress claims have concrete evidence updates (`status.json`, progress updates, trace index).
docs/audits/docs-ai-checklist-scan.md:158:docs/context/AI_PROMPTS.md:74:Read `docs/context/README_AI.md`, then `docs/status/status.json`, `docs/handoff/HANDOFF.json` (or `docs/status/HANDOFF.json`), then `docs/status/decisions.md`; treat `status.json` as SSOT and check `active_task/open_prs/locks_held` before action, declare Allowed/Forbidden paths and Required LOCKS in your task card, stop on LOCK conflict, record trace links in `docs/status/trace-index.json`, and if stopping early update HANDOFF with `what_done/what_next/errors/commands_next`.
docs/audits/docs-ai-checklist-scan.md:159:docs/context/AI_PROMPTS.md:78:Follow OS read order `README_AI → status.json → HANDOFF → decisions`; use `status.json` as authoritative runtime state (`active_task`, `open_prs`, `locks_held`), operate only in Allowed paths, stop and return on LOCK conflict, require evidence updates for any progress claim (`status.json` or progress updates), write trace evidence to `docs/status/trace-index.json`, and update HANDOFF fields before any pause/credit-out.
docs/audits/docs-ai-checklist-scan.md:160:docs/context/AI_PROMPTS.md:82:Start by reading `docs/context/README_AI.md`, `docs/status/status.json`, `docs/handoff/HANDOFF.json`/`docs/status/HANDOFF.json`, and `docs/status/decisions.md`; then create a task card with scope + Allowed/Forbidden + LOCKS, execute only safe paths, halt on lock collisions, ensure progress claims are backed by status/progress updates, keep trace links in `docs/status/trace-index.json`, and always write HANDOFF before stopping mid-task.
docs/audits/docs-ai-checklist-scan.md:161:docs/context/AI_PROMPTS.md:90:- Read: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
docs/audits/docs-ai-checklist-scan.md:164:docs/context/AI_PROMPTS.md:96:- Update evidence: `status.json` or `progress-updates` + `trace-index.json`.
docs/audits/docs-ai-checklist-scan.md:165:docs/context/AI_PROMPTS.md:97:- If pausing, update HANDOFF required keys.
docs/audits/docs-ai-checklist-scan.md:166:docs/context/AI_PROMPTS.md:102:- Validate active ownership and lock status in `status.json`.
docs/audits/docs-ai-checklist-scan.md:168:docs/context/AI_PROMPTS.md:107:- Record trace links in `trace-index.json`.
docs/audits/docs-ai-checklist-scan.md:169:docs/context/AI_PROMPTS.md:108:- On credit-out, update HANDOFF with actionable next commands.
docs/audits/docs-ai-checklist-scan.md:172:docs/context/AI_PROMPTS.md:119:- On interruption, complete HANDOFF fields before ending response.
docs/audits/docs-ai-checklist-scan.md:173:docs/rules/task-generation-policy.md:10:2. `docs/status/status.json`
docs/audits/docs-ai-checklist-scan.md:174:docs/rules/task-generation-policy.md:11:3. `docs/status/HANDOFF.json`
docs/audits/docs-ai-checklist-scan.md:175:docs/rules/task-generation-policy.md:12:4. `docs/status/decisions.md`
docs/audits/docs-ai-checklist-scan.md:176:docs/rules/task-generation-policy.md:19:- Active task ownership and lifecycle are tracked in `docs/status/status.json`.
docs/audits/docs-ai-checklist-scan.md:181:docs/rules/task-generation-policy.md:45:- Canonical runtime status must be updated through `docs/status/status.json` only.
docs/audits/docs-ai-checklist-scan.md:183:docs/rules/task-generation-policy.md:52:### 4.3 Mandatory HANDOFF update on stop/switch
docs/audits/docs-ai-checklist-scan.md:184:docs/rules/task-generation-policy.md:54:Before stopping or switching owner, update `docs/status/HANDOFF.json` with all of:
docs/audits/docs-ai-checklist-scan.md:185:docs/rules/task-generation-policy.md:66:  - an update to `docs/status/status.json`, **or**
docs/audits/docs-ai-checklist-scan.md:186:docs/rules/task-generation-policy.md:72:- Use `docs/status/decisions.md` to lock assumptions and decisions that affect ongoing work.
docs/audits/docs-ai-checklist-scan.md:189:docs/rules/task-generation-policy.md:78:- `docs/status/trace-index.md` is the SSOT for trace links (PRs, commits, issues, run artifacts).
docs/audits/docs-ai-checklist-scan.md:190:docs/rules/task-generation-policy.md:80:- Other docs may include convenience links, but must treat trace-index as canonical.
docs/audits/docs-ai-checklist-scan.md:191:docs/context/README_AI.md:9:3. Runtime status SSOT: `docs/status/status.json`
docs/audits/docs-ai-checklist-scan.md:192:docs/context/README_AI.md:12:6. Handoff state: `docs/status/HANDOFF.json` and `docs/handoff/HANDOFF.json`
docs/audits/docs-ai-checklist-scan.md:193:docs/context/README_AI.md:13:7. Decision baseline: `docs/status/decisions.md`
docs/audits/docs-ai-checklist-scan.md:194:docs/context/README_AI.md:14:8. Trace SSOT: `docs/status/trace-index.md` and `docs/status/trace-index.json`
docs/audits/docs-ai-checklist-scan.md:195:docs/context/README_AI.md:23:- **Status SSOT:** `docs/status/status.json`
docs/audits/docs-ai-checklist-scan.md:196:docs/context/README_AI.md:33:> Read in order: `docs/context/README_AI.md` → `docs/status/status.json` → `docs/handoff/HANDOFF.json` (or `docs/status/HANDOFF.json`) → `docs/status/decisions.md`.
docs/audits/docs-ai-checklist-scan.md:197:docs/context/README_AI.md:34:> Treat `docs/status/status.json` as SSOT and check `active_task`, `open_prs`, `locks_held` before planning.
docs/audits/docs-ai-checklist-scan.md:198:docs/context/README_AI.md:35:> If stopping/pause/credit-out before completion, you MUST update `docs/handoff/HANDOFF.json` (and keep `docs/status/HANDOFF.json` aligned if used).
docs/audits/docs-ai-checklist-scan.md:199:docs/context/README_AI.md:36:> Use `docs/status/trace-index.json` as trace link SSOT.
docs/audits/docs-ai-checklist-scan.md:200:docs/context/README_AI.md:37:> In task cards always declare Allowed/Forbidden paths and Required LOCKS; if LOCK conflicts exist, stop and return.
docs/audits/docs-ai-checklist-scan.md:201:docs/context/README_AI.md:41:- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/status/HANDOFF.json` and `docs/handoff/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
docs/audits/docs-ai-checklist-scan.md:202:docs/context/README_AI.md:42:- **Single active task:** only one active task is tracked in `docs/status/status.json`.
docs/audits/docs-ai-checklist-scan.md:203:docs/context/README_AI.md:43:- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
docs/audits/docs-ai-checklist-scan.md:204:docs/context/README_AI.md:44:- **LOCK areas:** lock-sensitive areas include contracts/ci/infra/lockfiles and shared-docs (`docs/context/**`, `docs/rules/**`, docs hubs). Declare `Required Locks` in task cards.
docs/audits/docs-ai-checklist-scan.md:205:docs/context/README_AI.md:45:- **Trace SSOT:** `docs/status/trace-index.md` / `docs/status/trace-index.json` are the canonical place for PR/commit/evidence links.
docs/audits/docs-ai-checklist-scan.md:206:docs/context/README_AI.md:46:- **Decisions are binding:** use `docs/status/decisions.md` to fix assumptions and supersede only via explicit new decision entries.
docs/audits/docs-ai-checklist-scan.md:207:docs/context/TECH_CONTEXT.md:12:- Runtime status SSOT: `docs/status/status.json`
docs/audits/docs-ai-checklist-scan.md:208:docs/context/TECH_CONTEXT.md:13:- Trace index: `docs/status/trace-index.md` and `docs/status/trace-index.json`
docs/audits/docs-ai-checklist-scan.md:209:docs/context/TECH_CONTEXT.md:14:- Decisions baseline: `docs/status/decisions.md` (and `docs/decisions/decisions.md` if referenced by task)
docs/audits/docs-ai-checklist-scan.md:210:docs/context/TECH_CONTEXT.md:15:- Handoff state: `docs/status/HANDOFF.json` (legacy/alt path may exist at `docs/handoff/HANDOFF.json`)
docs/audits/docs-ai-checklist-scan.md:211:docs/context/TECH_CONTEXT.md:42:- When in doubt, follow linked SSOT/authoritative docs and record decisions in `docs/status/decisions.md`.
docs/audits/docs-ai-checklist-scan.md:231:docs/handoff/HANDOFF.json:8:    "On next docs task, update status.json and trace-index.json first.",
docs/audits/docs-ai-checklist-scan.md:232:docs/handoff/HANDOFF.json:16:    "cat docs/status/status.json",
docs/audits/docs-ai-checklist-scan.md:233:docs/handoff/HANDOFF.json:17:    "cat docs/status/trace-index.json"
docs/audits/docs-ai-checklist-scan.md:265:docs/audits/docs-os-existing-file-scan.md:51:## Command: rg -n "README_AI|SSOT|handoff|HANDOFF|decision|decisions|status\.json|trace-index|LOCKS|glossary|canonical|North Star" docs -S
docs/audits/docs-ai-checklist-scan.md:281:docs/audits/docs-os-existing-file-scan.md:100:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/audits/docs-ai-checklist-scan.md:283:docs/decisions/records/docs-os-consolidation-decision.md:6:| Machine-readable current status | No direct candidate found in `docs/**` (`status.json` not present) | `docs/status/status.json` | **Keep (new canonical)** | Add a dedicated structured status file for active epic/task, locks, next actions, and PR state. |
docs/audits/docs-ai-checklist-scan.md:284:docs/decisions/records/docs-os-consolidation-decision.md:7:| Handoff protocol | No direct candidate found in `docs/**` (`HANDOFF.json` not present) | `docs/handoff/HANDOFF.json` | **Keep (new canonical)** | Add explicit stop/credit-out handoff file with required fields for multi-AI continuity. |
docs/audits/docs-ai-checklist-scan.md:285:docs/decisions/records/docs-os-consolidation-decision.md:8:| Decision log | No direct candidate found in `docs/**` (`docs/decisions/*.md` absent). Related mentions in `docs/changelog.md` and workplan notes. | `docs/decisions/decisions.md` | **Merge** | Keep changelog/workplan for their original purpose, and establish one decision log canonical path for short decision entries. |
docs/audits/docs-ai-checklist-scan.md:286:docs/decisions/records/docs-os-consolidation-decision.md:9:| Traceability index | No direct candidate found in `docs/**` (`trace-index.json` absent) | `docs/status/trace-index.json` | **Keep (new canonical)** | Introduce one machine-readable trace SSOT for req↔progress↔PR/commit/spec/runbook/verification links. |
docs/audits/docs-ai-checklist-scan.md:287:docs/decisions/records/docs-os-consolidation-decision.md:12:- No existing same-role canonical files were found for `status.json`, `HANDOFF.json`, `decisions.md`, or `trace-index.json`, so no stub replacement was required.
docs/audits/docs-ai-checklist-scan.md:293:docs/decisions/records/docs-rules-unify-decision.md:6:- Reason: existing rule content is concentrated in `docs/specs/crosscut/parallel_task_safety_spec.md`; new required operational topics (Multi-AI / Credit-out / HANDOFF / status / trace / decisions / LOCK policy) should be explicit and discoverable under a dedicated operations-rule path.
docs/audits/docs-ai-checklist-scan.md:295:docs/decisions/records/docs-rules-unify-decision.md:13:| Parallel safety / 1PR=1scope / Allowed/Forbidden | `docs/specs/crosscut/parallel_task_safety_spec.md` | `docs/specs/crosscut/safety_interlock_spec.md` | merge + stub old | Preserve prior safety intent and checklist; move to rules namespace. |
docs/audits/docs-ai-checklist-scan.md:296:docs/decisions/records/docs-rules-unify-decision.md:14:| Multi-AI / Credit-out / Handoff | `docs/specs/crosscut/parallel_task_safety_spec.md` (partial task-generation section only) | `docs/rules/task-generation-policy.md` | merge + extend | Add stop protocol + mandatory HANDOFF update semantics. |
docs/audits/docs-ai-checklist-scan.md:297:docs/decisions/records/docs-rules-unify-decision.md:15:| LOCK policy | `docs/specs/crosscut/parallel_task_safety_spec.md` (LOCK/semi-LOCK partial) | `docs/specs/crosscut/safety_interlock_spec.md` | merge + extend | Add explicit lock areas incl. shared-docs and Required Locks declaration. |
docs/audits/docs-ai-checklist-scan.md:298:docs/decisions/records/docs-rules-unify-decision.md:16:| Decision log policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Standardize decision fixation via `docs/status/decisions.md`. |
docs/audits/docs-ai-checklist-scan.md:299:docs/decisions/records/docs-rules-unify-decision.md:17:| Traceability policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Set `docs/status/trace-index.md` as trace SSOT, avoid scattered links. |
docs/audits/docs-ai-checklist-scan.md:300:docs/decisions/records/docs-rules-unify-decision.md:18:| Task card required fields | `docs/specs/crosscut/parallel_task_safety_spec.md` section 4.1 | `docs/rules/task-generation-policy.md` | merge + extend | Keep minimum fields and add multi-AI runtime governance fields. |
docs/audits/docs-ai-checklist-scan.md:302:docs/decisions/records/docs-ai-prompts-decision.md:5:| Magic prompt (must-read OS) | `docs/context/README_AI.md` mandatory entrypoint; no dedicated magic-prompt block found | `docs/context/README_AI.md` | merge | Keep single canonical onboarding page and add concise universal magic prompt there. |
docs/audits/docs-ai-checklist-scan.md:303:docs/decisions/records/docs-ai-prompts-decision.md:7:| Stop/Credit-out/Handoff text | Existing stop protocol in `docs/context/README_AI.md` | `docs/context/README_AI.md` + mirrored in `docs/context/AI_PROMPTS.md` | merge | Keep wording consistent with OS: stop/pause/handoff requires HANDOFF update before exit. |
docs/audits/docs-ai-checklist-scan.md:306:docs/decisions/records/docs-ai-pack-decision.md:5:| AI entry (constitution) | `docs/context/README_AI.md` | `docs/context/README_AI.md` | keep/merge | Must remain canonical AI entrypoint. Add links to new non-SSOT context files. |
docs/audits/docs-ai-checklist-scan.md:307:docs/decisions/records/docs-ai-pack-decision.md:6:| Human-readable current status | `docs/status/status.json` (machine-readable SSOT), no existing markdown summary found | `docs/status/CURRENT_STATUS.md` | create/merge | New file is a thin, human-readable summary only. Explicitly states SSOT is `docs/status/status.json`. |
docs/audits/docs-ai-checklist-scan.md:309:docs/decisions/records/docs-ai-pack-decision.md:13:- Treat `docs/status/CURRENT_STATUS.md` as non-SSOT summary derived from `docs/status/status.json`.
docs/audits/docs-ai-checklist-scan.md:312:docs/audits/docs-ai-pack-scan.md:23:docs/decisions/decisions.md
docs/audits/docs-ai-checklist-scan.md:313:docs/audits/docs-ai-pack-scan.md:24:docs/handoff/HANDOFF.json
docs/audits/docs-ai-checklist-scan.md:314:docs/audits/docs-ai-pack-scan.md:43:docs/status/HANDOFF.json
docs/audits/docs-ai-checklist-scan.md:315:docs/audits/docs-ai-pack-scan.md:44:docs/status/decisions.md
docs/audits/docs-ai-checklist-scan.md:316:docs/audits/docs-ai-pack-scan.md:54:docs/status/status.json
docs/audits/docs-ai-checklist-scan.md:317:docs/audits/docs-ai-pack-scan.md:55:docs/status/trace-index.json
docs/audits/docs-ai-checklist-scan.md:318:docs/audits/docs-ai-pack-scan.md:56:docs/status/trace-index.md
docs/audits/docs-ai-checklist-scan.md:319:docs/audits/docs-ai-pack-scan.md:64:## Command: rg -n "CURRENT_STATUS|AI_READ_ME|TECH_CONTEXT|README_AI|SSOT|status\.json|trace-index|handoff|HANDOFF|decisions|LOCKS|glossary|context pack" docs -S
docs/audits/docs-ai-checklist-scan.md:320:docs/audits/docs-ai-pack-scan.md:66:docs/decisions/decisions.md:6:- 2026-02-17: contracts additive-only (see contracts policy SSOT where applicable) / Prevents breaking downstream consumers / Contract evolution must preserve compatibility.
docs/audits/docs-ai-checklist-scan.md:321:docs/audits/docs-ai-pack-scan.md:67:docs/decisions/decisions.md:7:- 2026-02-17: docs canonical入口 is `docs/context/README_AI.md` / Single entrypoint prevents SSOT ambiguity / AI and human operators start from one canonical path.
docs/audits/docs-ai-checklist-scan.md:322:docs/audits/docs-ai-pack-scan.md:68:docs/decisions/decisions.md:8:- 2026-02-17: stopping requires `docs/handoff/HANDOFF.json` update / Multi-agent and interrupted work need explicit continuity state / No stop/handoff without handoff state refresh.
docs/audits/docs-ai-checklist-scan.md:323:docs/audits/docs-ai-pack-scan.md:70:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/audits/docs-ai-checklist-scan.md:326:docs/audits/docs-ai-pack-scan.md:78:docs/specs/crosscut/parallel_task_safety_spec.md:8:- `docs/rules/task-generation-policy.md` (task generation / Multi-AI / Credit-out / HANDOFF / status / decisions / trace)
docs/audits/docs-ai-checklist-scan.md:327:docs/audits/docs-ai-pack-scan.md:80:docs/status/trace-index.md:1:# Trace Index (SSOT)
docs/audits/docs-ai-checklist-scan.md:328:docs/audits/docs-ai-pack-scan.md:81:docs/status/trace-index.json:16:        "docs/context/README_AI.md",
docs/audits/docs-ai-checklist-scan.md:329:docs/audits/docs-ai-pack-scan.md:82:docs/status/trace-index.json:17:        "docs/status/status.json",
docs/audits/docs-ai-checklist-scan.md:330:docs/audits/docs-ai-pack-scan.md:83:docs/status/trace-index.json:18:        "docs/handoff/HANDOFF.json",
docs/audits/docs-ai-checklist-scan.md:331:docs/audits/docs-ai-pack-scan.md:84:docs/status/trace-index.json:19:        "docs/decisions/decisions.md"
docs/audits/docs-ai-checklist-scan.md:332:docs/audits/docs-ai-pack-scan.md:87:docs/status/decisions.md:1:# Decisions Log (SSOT)
docs/audits/docs-ai-checklist-scan.md:333:docs/audits/docs-ai-pack-scan.md:90:docs/context/README_AI.md:8:2. Runtime status: `docs/status/status.json`
docs/audits/docs-ai-checklist-scan.md:334:docs/audits/docs-ai-pack-scan.md:91:docs/context/README_AI.md:9:3. Handoff state: `docs/status/HANDOFF.json`
docs/audits/docs-ai-checklist-scan.md:335:docs/audits/docs-ai-pack-scan.md:92:docs/context/README_AI.md:10:4. Decision baseline: `docs/status/decisions.md`
docs/audits/docs-ai-checklist-scan.md:336:docs/audits/docs-ai-pack-scan.md:94:docs/context/README_AI.md:17:- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/status/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
docs/audits/docs-ai-checklist-scan.md:337:docs/audits/docs-ai-pack-scan.md:95:docs/context/README_AI.md:18:- **Single active task:** only one active task is tracked in `docs/status/status.json`.
docs/audits/docs-ai-checklist-scan.md:338:docs/audits/docs-ai-pack-scan.md:96:docs/context/README_AI.md:19:- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
docs/audits/docs-ai-checklist-scan.md:339:docs/audits/docs-ai-pack-scan.md:97:docs/context/README_AI.md:20:- **LOCK areas:** lock-sensitive areas include contracts/ci/infra/lockfiles and shared-docs (`docs/context/**`, `docs/rules/**`, docs hubs). Declare `Required Locks` in task cards.
docs/audits/docs-ai-checklist-scan.md:340:docs/audits/docs-ai-pack-scan.md:98:docs/context/README_AI.md:21:- **Trace SSOT:** `docs/status/trace-index.md` is the canonical place for PR/commit/evidence links.
docs/audits/docs-ai-checklist-scan.md:341:docs/audits/docs-ai-pack-scan.md:99:docs/context/README_AI.md:22:- **Decisions are binding:** use `docs/status/decisions.md` to fix assumptions and supersede only via explicit new decision entries.
docs/audits/docs-ai-checklist-scan.md:342:docs/audits/docs-ai-pack-scan.md:102:docs/rules/task-generation-policy.md:10:2. `docs/status/status.json`
docs/audits/docs-ai-checklist-scan.md:343:docs/audits/docs-ai-pack-scan.md:103:docs/rules/task-generation-policy.md:11:3. `docs/status/HANDOFF.json`
docs/audits/docs-ai-checklist-scan.md:344:docs/audits/docs-ai-pack-scan.md:104:docs/rules/task-generation-policy.md:12:4. `docs/status/decisions.md`
docs/audits/docs-ai-checklist-scan.md:345:docs/audits/docs-ai-pack-scan.md:105:docs/rules/task-generation-policy.md:19:- Active task ownership and lifecycle are tracked in `docs/status/status.json`.
docs/audits/docs-ai-checklist-scan.md:346:docs/audits/docs-ai-pack-scan.md:106:docs/rules/task-generation-policy.md:45:- Canonical runtime status must be updated through `docs/status/status.json` only.
docs/audits/docs-ai-checklist-scan.md:347:docs/audits/docs-ai-pack-scan.md:108:docs/rules/task-generation-policy.md:52:### 4.3 Mandatory HANDOFF update on stop/switch
docs/audits/docs-ai-checklist-scan.md:348:docs/audits/docs-ai-pack-scan.md:109:docs/rules/task-generation-policy.md:54:Before stopping or switching owner, update `docs/status/HANDOFF.json` with all of:
docs/audits/docs-ai-checklist-scan.md:349:docs/audits/docs-ai-pack-scan.md:110:docs/rules/task-generation-policy.md:66:  - an update to `docs/status/status.json`, **or**
docs/audits/docs-ai-checklist-scan.md:350:docs/audits/docs-ai-pack-scan.md:111:docs/rules/task-generation-policy.md:72:- Use `docs/status/decisions.md` to lock assumptions and decisions that affect ongoing work.
docs/audits/docs-ai-checklist-scan.md:351:docs/audits/docs-ai-pack-scan.md:112:docs/rules/task-generation-policy.md:78:- `docs/status/trace-index.md` is the SSOT for trace links (PRs, commits, issues, run artifacts).
docs/audits/docs-ai-checklist-scan.md:352:docs/audits/docs-ai-pack-scan.md:113:docs/rules/task-generation-policy.md:80:- Other docs may include convenience links, but must treat trace-index as canonical.
docs/audits/docs-ai-checklist-scan.md:354:docs/audits/docs-ai-pack-scan.md:117:docs/handoff/HANDOFF.json:8:    "On next docs task, update status.json and trace-index.json first.",
docs/audits/docs-ai-checklist-scan.md:355:docs/audits/docs-ai-pack-scan.md:118:docs/handoff/HANDOFF.json:9:    "If stopping mid-task, update this handoff file before exit."
docs/audits/docs-ai-checklist-scan.md:356:docs/audits/docs-ai-pack-scan.md:119:docs/handoff/HANDOFF.json:16:    "cat docs/status/status.json",
docs/audits/docs-ai-checklist-scan.md:357:docs/audits/docs-ai-pack-scan.md:120:docs/handoff/HANDOFF.json:17:    "cat docs/status/trace-index.json"
docs/audits/docs-ai-checklist-scan.md:359:docs/audits/docs-ai-pack-scan.md:123:docs/audits/docs-os-existing-file-scan.md:51:## Command: rg -n "README_AI|SSOT|handoff|HANDOFF|decision|decisions|status\.json|trace-index|LOCKS|glossary|canonical|North Star" docs -S
docs/audits/docs-ai-checklist-scan.md:364:docs/audits/docs-ai-pack-scan.md:142:docs/audits/docs-os-existing-file-scan.md:100:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/audits/docs-ai-checklist-scan.md:368:docs/audits/docs-ai-pack-scan.md:157:docs/decisions/records/docs-rules-unify-decision.md:6:- Reason: existing rule content is concentrated in `docs/specs/crosscut/parallel_task_safety_spec.md`; new required operational topics (Multi-AI / Credit-out / HANDOFF / status / trace / decisions / LOCK policy) should be explicit and discoverable under a dedicated operations-rule path.
docs/audits/docs-ai-checklist-scan.md:369:docs/audits/docs-ai-pack-scan.md:158:docs/decisions/records/docs-rules-unify-decision.md:14:| Multi-AI / Credit-out / Handoff | `docs/specs/crosscut/parallel_task_safety_spec.md` (partial task-generation section only) | `docs/rules/task-generation-policy.md` | merge + extend | Add stop protocol + mandatory HANDOFF update semantics. |
docs/audits/docs-ai-checklist-scan.md:370:docs/audits/docs-ai-pack-scan.md:159:docs/decisions/records/docs-rules-unify-decision.md:16:| Decision log policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Standardize decision fixation via `docs/status/decisions.md`. |
docs/audits/docs-ai-checklist-scan.md:371:docs/audits/docs-ai-pack-scan.md:160:docs/decisions/records/docs-rules-unify-decision.md:17:| Traceability policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Set `docs/status/trace-index.md` as trace SSOT, avoid scattered links. |
docs/audits/docs-ai-checklist-scan.md:372:docs/audits/docs-ai-pack-scan.md:161:docs/decisions/records/docs-rules-unify-decision.md:22:- `docs/context/README_AI.md` is created/updated as must-read entrypoint for AI operators.
docs/audits/docs-ai-checklist-scan.md:374:docs/audits/docs-ai-pack-scan.md:163:docs/decisions/records/docs-os-consolidation-decision.md:5:| SSOT entry for Docs Development OS | `docs/README.md` (general docs index), `docs/specs/crosscut/parallel_task_safety_spec.md` (parallel safety SSOT note), `docs/audits/docs-content-overlap.md` (canonicalization guidance) | `docs/context/README_AI.md` | **Merge** | Existing docs already describe partial SSOT/canonical concepts, but no single AI-first entrypoint exists. Create one canonical entrance and link all SSOT roles there. |
docs/audits/docs-ai-checklist-scan.md:375:docs/audits/docs-ai-pack-scan.md:164:docs/decisions/records/docs-os-consolidation-decision.md:6:| Machine-readable current status | No direct candidate found in `docs/**` (`status.json` not present) | `docs/status/status.json` | **Keep (new canonical)** | Add a dedicated structured status file for active epic/task, locks, next actions, and PR state. |
docs/audits/docs-ai-checklist-scan.md:376:docs/audits/docs-ai-pack-scan.md:165:docs/decisions/records/docs-os-consolidation-decision.md:7:| Handoff protocol | No direct candidate found in `docs/**` (`HANDOFF.json` not present) | `docs/handoff/HANDOFF.json` | **Keep (new canonical)** | Add explicit stop/credit-out handoff file with required fields for multi-AI continuity. |
docs/audits/docs-ai-checklist-scan.md:377:docs/audits/docs-ai-pack-scan.md:166:docs/decisions/records/docs-os-consolidation-decision.md:8:| Decision log | No direct candidate found in `docs/**` (`docs/decisions/*.md` absent). Related mentions in `docs/changelog.md` and workplan notes. | `docs/decisions/decisions.md` | **Merge** | Keep changelog/workplan for their original purpose, and establish one decision log canonical path for short decision entries. |
docs/audits/docs-ai-checklist-scan.md:378:docs/audits/docs-ai-pack-scan.md:167:docs/decisions/records/docs-os-consolidation-decision.md:9:| Traceability index | No direct candidate found in `docs/**` (`trace-index.json` absent) | `docs/status/trace-index.json` | **Keep (new canonical)** | Introduce one machine-readable trace SSOT for req↔progress↔PR/commit/spec/runbook/verification links. |
docs/audits/docs-ai-checklist-scan.md:379:docs/audits/docs-ai-pack-scan.md:168:docs/decisions/records/docs-os-consolidation-decision.md:12:- No existing same-role canonical files were found for `status.json`, `HANDOFF.json`, `decisions.md`, or `trace-index.json`, so no stub replacement was required.
docs/audits/docs-ai-checklist-scan.md:380:docs/audits/docs-ai-pack-scan.md:169:docs/decisions/records/docs-os-consolidation-decision.md:13:- `docs/README.md` remains a general docs hub; `docs/context/README_AI.md` becomes the canonical AI operating entrypoint and references role-specific SSOT paths.
docs/audits/docs-ai-checklist-scan.md:381:docs/audits/docs-ai-pack-scan.md:177:docs/audits/docs-rules-unify-scan.md:52:## 3) `rg -n "parallel-task-safety|タスク生成|task generation|Codexアジャイル|致命的事故|guardrail|LOCK:|handoff|HANDOFF|status\.json|trace-index|decisions|SSOT|README_AI" docs -S`
docs/audits/docs-ai-checklist-scan.md:383:docs/audits/docs-ai-pack-scan.md:181:docs/audits/docs-rules-unify-scan.md:58:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/audits/docs-ai-checklist-scan.md:388:docs/audits/docs-ai-pack-scan.md:204:docs/status/status.json
docs/audits/docs-ai-checklist-scan.md:389:docs/audits/docs-rules-unify-scan.md:52:## 3) `rg -n "parallel-task-safety|タスク生成|task generation|Codexアジャイル|致命的事故|guardrail|LOCK:|handoff|HANDOFF|status\.json|trace-index|decisions|SSOT|README_AI" docs -S`
docs/audits/docs-ai-checklist-scan.md:391:docs/audits/docs-rules-unify-scan.md:58:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/audits/docs-ai-checklist-scan.md:399:docs/audits/docs-ai-prompts-scan.md:26:docs/decisions/decisions.md
docs/audits/docs-ai-checklist-scan.md:400:docs/audits/docs-ai-prompts-scan.md:27:docs/handoff/HANDOFF.json
docs/audits/docs-ai-checklist-scan.md:401:docs/audits/docs-ai-prompts-scan.md:47:docs/status/HANDOFF.json
docs/audits/docs-ai-checklist-scan.md:402:docs/audits/docs-ai-prompts-scan.md:48:docs/status/decisions.md
docs/audits/docs-ai-checklist-scan.md:403:docs/audits/docs-ai-prompts-scan.md:58:docs/status/status.json
docs/audits/docs-ai-checklist-scan.md:404:docs/audits/docs-ai-prompts-scan.md:59:docs/status/trace-index.json
docs/audits/docs-ai-checklist-scan.md:405:docs/audits/docs-ai-prompts-scan.md:60:docs/status/trace-index.md
docs/audits/docs-ai-checklist-scan.md:406:docs/audits/docs-ai-prompts-scan.md:68:## Command: rg -n "magic prompt|プロンプト|prompt template|Planner|Builder|Verifier|Reviewer|Copilot|Claude|Gemini|ChatGPT|README_AI|SSOT|handoff|HANDOFF|status\.json|trace-index|LOCK" docs -S
docs/audits/docs-ai-checklist-scan.md:407:docs/audits/docs-ai-prompts-scan.md:70:docs/decisions/decisions.md:6:- 2026-02-17: contracts additive-only (see contracts policy SSOT where applicable) / Prevents breaking downstream consumers / Contract evolution must preserve compatibility.
docs/audits/docs-ai-checklist-scan.md:408:docs/audits/docs-ai-prompts-scan.md:71:docs/decisions/decisions.md:7:- 2026-02-17: docs canonical入口 is `docs/context/README_AI.md` / Single entrypoint prevents SSOT ambiguity / AI and human operators start from one canonical path.
docs/audits/docs-ai-checklist-scan.md:409:docs/audits/docs-ai-prompts-scan.md:72:docs/decisions/decisions.md:8:- 2026-02-17: stopping requires `docs/handoff/HANDOFF.json` update / Multi-agent and interrupted work need explicit continuity state / No stop/handoff without handoff state refresh.
docs/audits/docs-ai-checklist-scan.md:412:docs/audits/docs-ai-prompts-scan.md:84:docs/specs/crosscut/parallel_task_safety_spec.md:8:- `docs/rules/task-generation-policy.md` (task generation / Multi-AI / Credit-out / HANDOFF / status / decisions / trace)
docs/audits/docs-ai-checklist-scan.md:413:docs/audits/docs-ai-prompts-scan.md:86:docs/status/trace-index.md:1:# Trace Index (SSOT)
docs/audits/docs-ai-checklist-scan.md:414:docs/audits/docs-ai-prompts-scan.md:87:docs/status/trace-index.json:16:        "docs/context/README_AI.md",
docs/audits/docs-ai-checklist-scan.md:415:docs/audits/docs-ai-prompts-scan.md:88:docs/status/trace-index.json:17:        "docs/status/status.json",
docs/audits/docs-ai-checklist-scan.md:416:docs/audits/docs-ai-prompts-scan.md:89:docs/status/trace-index.json:18:        "docs/handoff/HANDOFF.json",
docs/audits/docs-ai-checklist-scan.md:417:docs/audits/docs-ai-prompts-scan.md:95:docs/status/decisions.md:1:# Decisions Log (SSOT)
docs/audits/docs-ai-checklist-scan.md:418:docs/audits/docs-ai-prompts-scan.md:96:docs/status/CURRENT_STATUS.md:3:_Last updated: 1970-01-01T00:00:00Z (from `docs/status/status.json`)_
docs/audits/docs-ai-checklist-scan.md:419:docs/audits/docs-ai-prompts-scan.md:97:docs/status/CURRENT_STATUS.md:6:> **SSOT is `docs/status/status.json`**.
docs/audits/docs-ai-checklist-scan.md:420:docs/audits/docs-ai-prompts-scan.md:98:docs/status/CURRENT_STATUS.md:11:- Active epic: _not specified in `status.json`_
docs/audits/docs-ai-checklist-scan.md:421:docs/audits/docs-ai-prompts-scan.md:99:docs/status/CURRENT_STATUS.md:17:- _No open PRs recorded in `status.json`._
docs/audits/docs-ai-checklist-scan.md:422:docs/audits/docs-ai-prompts-scan.md:100:docs/status/CURRENT_STATUS.md:21:- _No locks recorded in `status.json`._
docs/audits/docs-ai-checklist-scan.md:423:docs/audits/docs-ai-prompts-scan.md:101:docs/status/CURRENT_STATUS.md:25:- _None explicitly recorded in `status.json`._
docs/audits/docs-ai-checklist-scan.md:424:docs/audits/docs-ai-prompts-scan.md:102:docs/status/CURRENT_STATUS.md:29:- Update `docs/status/status.json` when a task owner is assigned.
docs/audits/docs-ai-checklist-scan.md:425:docs/audits/docs-ai-prompts-scan.md:103:docs/status/CURRENT_STATUS.md:31:- Add progress evidence under `docs/status/progress-updates/` and trace links in `docs/status/trace-index.md`/`.json` as work proceeds.
docs/audits/docs-ai-checklist-scan.md:426:docs/audits/docs-ai-prompts-scan.md:104:docs/status/CURRENT_STATUS.md:36:- `docs/status/status.json` (machine-readable SSOT)
docs/audits/docs-ai-checklist-scan.md:429:docs/audits/docs-ai-prompts-scan.md:112:docs/rules/task-generation-policy.md:10:2. `docs/status/status.json`
docs/audits/docs-ai-checklist-scan.md:430:docs/audits/docs-ai-prompts-scan.md:113:docs/rules/task-generation-policy.md:11:3. `docs/status/HANDOFF.json`
docs/audits/docs-ai-checklist-scan.md:431:docs/audits/docs-ai-prompts-scan.md:114:docs/rules/task-generation-policy.md:19:- Active task ownership and lifecycle are tracked in `docs/status/status.json`.
docs/audits/docs-ai-checklist-scan.md:432:docs/audits/docs-ai-prompts-scan.md:116:docs/rules/task-generation-policy.md:45:- Canonical runtime status must be updated through `docs/status/status.json` only.
docs/audits/docs-ai-checklist-scan.md:433:docs/audits/docs-ai-prompts-scan.md:118:docs/rules/task-generation-policy.md:52:### 4.3 Mandatory HANDOFF update on stop/switch
docs/audits/docs-ai-checklist-scan.md:434:docs/audits/docs-ai-prompts-scan.md:119:docs/rules/task-generation-policy.md:54:Before stopping or switching owner, update `docs/status/HANDOFF.json` with all of:
docs/audits/docs-ai-checklist-scan.md:435:docs/audits/docs-ai-prompts-scan.md:120:docs/rules/task-generation-policy.md:66:  - an update to `docs/status/status.json`, **or**
docs/audits/docs-ai-checklist-scan.md:436:docs/audits/docs-ai-prompts-scan.md:121:docs/rules/task-generation-policy.md:78:- `docs/status/trace-index.md` is the SSOT for trace links (PRs, commits, issues, run artifacts).
docs/audits/docs-ai-checklist-scan.md:437:docs/audits/docs-ai-prompts-scan.md:122:docs/rules/task-generation-policy.md:80:- Other docs may include convenience links, but must treat trace-index as canonical.
docs/audits/docs-ai-checklist-scan.md:439:docs/audits/docs-ai-prompts-scan.md:134:docs/context/README_AI.md:9:3. Runtime status SSOT: `docs/status/status.json`
docs/audits/docs-ai-checklist-scan.md:440:docs/audits/docs-ai-prompts-scan.md:136:docs/context/README_AI.md:11:5. Handoff state: `docs/status/HANDOFF.json`
docs/audits/docs-ai-checklist-scan.md:441:docs/audits/docs-ai-prompts-scan.md:137:docs/context/README_AI.md:13:7. Trace SSOT: `docs/status/trace-index.md`
docs/audits/docs-ai-checklist-scan.md:442:docs/audits/docs-ai-prompts-scan.md:141:docs/context/README_AI.md:22:- **Status SSOT:** `docs/status/status.json`
docs/audits/docs-ai-checklist-scan.md:443:docs/audits/docs-ai-prompts-scan.md:144:docs/context/README_AI.md:29:- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/status/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
docs/audits/docs-ai-checklist-scan.md:444:docs/audits/docs-ai-prompts-scan.md:145:docs/context/README_AI.md:30:- **Single active task:** only one active task is tracked in `docs/status/status.json`.
docs/audits/docs-ai-checklist-scan.md:445:docs/audits/docs-ai-prompts-scan.md:146:docs/context/README_AI.md:31:- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
docs/audits/docs-ai-checklist-scan.md:446:docs/audits/docs-ai-prompts-scan.md:147:docs/context/README_AI.md:32:- **LOCK areas:** lock-sensitive areas include contracts/ci/infra/lockfiles and shared-docs (`docs/context/**`, `docs/rules/**`, docs hubs). Declare `Required Locks` in task cards.
docs/audits/docs-ai-checklist-scan.md:447:docs/audits/docs-ai-prompts-scan.md:148:docs/context/README_AI.md:33:- **Trace SSOT:** `docs/status/trace-index.md` is the canonical place for PR/commit/evidence links.
docs/audits/docs-ai-checklist-scan.md:448:docs/audits/docs-ai-prompts-scan.md:151:docs/context/TECH_CONTEXT.md:12:- Runtime status SSOT: `docs/status/status.json`
docs/audits/docs-ai-checklist-scan.md:449:docs/audits/docs-ai-prompts-scan.md:152:docs/context/TECH_CONTEXT.md:13:- Trace index: `docs/status/trace-index.md` and `docs/status/trace-index.json`
docs/audits/docs-ai-checklist-scan.md:450:docs/audits/docs-ai-prompts-scan.md:153:docs/context/TECH_CONTEXT.md:15:- Handoff state: `docs/status/HANDOFF.json` (legacy/alt path may exist at `docs/handoff/HANDOFF.json`)
docs/audits/docs-ai-checklist-scan.md:451:docs/audits/docs-ai-prompts-scan.md:154:docs/context/TECH_CONTEXT.md:42:- When in doubt, follow linked SSOT/authoritative docs and record decisions in `docs/status/decisions.md`.
docs/audits/docs-ai-checklist-scan.md:452:docs/audits/docs-ai-prompts-scan.md:155:docs/handoff/HANDOFF.json:8:    "On next docs task, update status.json and trace-index.json first.",
docs/audits/docs-ai-checklist-scan.md:453:docs/audits/docs-ai-prompts-scan.md:156:docs/handoff/HANDOFF.json:9:    "If stopping mid-task, update this handoff file before exit."
docs/audits/docs-ai-checklist-scan.md:454:docs/audits/docs-ai-prompts-scan.md:157:docs/handoff/HANDOFF.json:16:    "cat docs/status/status.json",
docs/audits/docs-ai-checklist-scan.md:455:docs/audits/docs-ai-prompts-scan.md:158:docs/handoff/HANDOFF.json:17:    "cat docs/status/trace-index.json"
docs/audits/docs-ai-checklist-scan.md:457:docs/audits/docs-ai-prompts-scan.md:161:docs/audits/docs-os-existing-file-scan.md:51:## Command: rg -n "README_AI|SSOT|handoff|HANDOFF|decision|decisions|status\.json|trace-index|LOCKS|glossary|canonical|North Star" docs -S
docs/audits/docs-ai-checklist-scan.md:467:docs/audits/docs-ai-prompts-scan.md:196:docs/decisions/records/docs-rules-unify-decision.md:6:- Reason: existing rule content is concentrated in `docs/specs/crosscut/parallel_task_safety_spec.md`; new required operational topics (Multi-AI / Credit-out / HANDOFF / status / trace / decisions / LOCK policy) should be explicit and discoverable under a dedicated operations-rule path.
docs/audits/docs-ai-checklist-scan.md:468:docs/audits/docs-ai-prompts-scan.md:197:docs/decisions/records/docs-rules-unify-decision.md:14:| Multi-AI / Credit-out / Handoff | `docs/specs/crosscut/parallel_task_safety_spec.md` (partial task-generation section only) | `docs/rules/task-generation-policy.md` | merge + extend | Add stop protocol + mandatory HANDOFF update semantics. |
docs/audits/docs-ai-checklist-scan.md:469:docs/audits/docs-ai-prompts-scan.md:198:docs/decisions/records/docs-rules-unify-decision.md:15:| LOCK policy | `docs/specs/crosscut/parallel_task_safety_spec.md` (LOCK/semi-LOCK partial) | `docs/specs/crosscut/safety_interlock_spec.md` | merge + extend | Add explicit lock areas incl. shared-docs and Required Locks declaration. |
docs/audits/docs-ai-checklist-scan.md:470:docs/audits/docs-ai-prompts-scan.md:199:docs/decisions/records/docs-rules-unify-decision.md:16:| Decision log policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Standardize decision fixation via `docs/status/decisions.md`. |
docs/audits/docs-ai-checklist-scan.md:471:docs/audits/docs-ai-prompts-scan.md:200:docs/decisions/records/docs-rules-unify-decision.md:17:| Traceability policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Set `docs/status/trace-index.md` as trace SSOT, avoid scattered links. |
docs/audits/docs-ai-checklist-scan.md:472:docs/audits/docs-ai-prompts-scan.md:201:docs/decisions/records/docs-rules-unify-decision.md:22:- `docs/context/README_AI.md` is created/updated as must-read entrypoint for AI operators.
docs/audits/docs-ai-checklist-scan.md:473:docs/audits/docs-ai-prompts-scan.md:202:docs/decisions/records/docs-ai-pack-decision.md:5:| AI entry (constitution) | `docs/context/README_AI.md` | `docs/context/README_AI.md` | keep/merge | Must remain canonical AI entrypoint. Add links to new non-SSOT context files. |
docs/audits/docs-ai-checklist-scan.md:474:docs/audits/docs-ai-prompts-scan.md:203:docs/decisions/records/docs-ai-pack-decision.md:6:| Human-readable current status | `docs/status/status.json` (machine-readable SSOT), no existing markdown summary found | `docs/status/CURRENT_STATUS.md` | create/merge | New file is a thin, human-readable summary only. Explicitly states SSOT is `docs/status/status.json`. |
docs/audits/docs-ai-checklist-scan.md:477:docs/audits/docs-ai-prompts-scan.md:206:docs/decisions/records/docs-ai-pack-decision.md:12:- Keep `docs/context/README_AI.md` as the single AI constitution entrypoint.
docs/audits/docs-ai-checklist-scan.md:478:docs/audits/docs-ai-prompts-scan.md:207:docs/decisions/records/docs-ai-pack-decision.md:13:- Treat `docs/status/CURRENT_STATUS.md` as non-SSOT summary derived from `docs/status/status.json`.
docs/audits/docs-ai-checklist-scan.md:481:docs/audits/docs-ai-prompts-scan.md:211:docs/decisions/records/docs-os-consolidation-decision.md:5:| SSOT entry for Docs Development OS | `docs/README.md` (general docs index), `docs/specs/crosscut/parallel_task_safety_spec.md` (parallel safety SSOT note), `docs/audits/docs-content-overlap.md` (canonicalization guidance) | `docs/context/README_AI.md` | **Merge** | Existing docs already describe partial SSOT/canonical concepts, but no single AI-first entrypoint exists. Create one canonical entrance and link all SSOT roles there. |
docs/audits/docs-ai-checklist-scan.md:482:docs/audits/docs-ai-prompts-scan.md:212:docs/decisions/records/docs-os-consolidation-decision.md:6:| Machine-readable current status | No direct candidate found in `docs/**` (`status.json` not present) | `docs/status/status.json` | **Keep (new canonical)** | Add a dedicated structured status file for active epic/task, locks, next actions, and PR state. |
docs/audits/docs-ai-checklist-scan.md:483:docs/audits/docs-ai-prompts-scan.md:213:docs/decisions/records/docs-os-consolidation-decision.md:7:| Handoff protocol | No direct candidate found in `docs/**` (`HANDOFF.json` not present) | `docs/handoff/HANDOFF.json` | **Keep (new canonical)** | Add explicit stop/credit-out handoff file with required fields for multi-AI continuity. |
docs/audits/docs-ai-checklist-scan.md:484:docs/audits/docs-ai-prompts-scan.md:214:docs/decisions/records/docs-os-consolidation-decision.md:9:| Traceability index | No direct candidate found in `docs/**` (`trace-index.json` absent) | `docs/status/trace-index.json` | **Keep (new canonical)** | Introduce one machine-readable trace SSOT for req↔progress↔PR/commit/spec/runbook/verification links. |
docs/audits/docs-ai-checklist-scan.md:485:docs/audits/docs-ai-prompts-scan.md:215:docs/decisions/records/docs-os-consolidation-decision.md:12:- No existing same-role canonical files were found for `status.json`, `HANDOFF.json`, `decisions.md`, or `trace-index.json`, so no stub replacement was required.
docs/audits/docs-ai-checklist-scan.md:486:docs/audits/docs-ai-prompts-scan.md:216:docs/decisions/records/docs-os-consolidation-decision.md:13:- `docs/README.md` remains a general docs hub; `docs/context/README_AI.md` becomes the canonical AI operating entrypoint and references role-specific SSOT paths.
docs/audits/docs-ai-checklist-scan.md:487:docs/audits/docs-ai-prompts-scan.md:225:docs/audits/docs-ai-pack-scan.md:24:docs/handoff/HANDOFF.json
docs/audits/docs-ai-checklist-scan.md:488:docs/audits/docs-ai-prompts-scan.md:226:docs/audits/docs-ai-pack-scan.md:43:docs/status/HANDOFF.json
docs/audits/docs-ai-checklist-scan.md:489:docs/audits/docs-ai-prompts-scan.md:227:docs/audits/docs-ai-pack-scan.md:54:docs/status/status.json
docs/audits/docs-ai-checklist-scan.md:490:docs/audits/docs-ai-prompts-scan.md:228:docs/audits/docs-ai-pack-scan.md:55:docs/status/trace-index.json
docs/audits/docs-ai-checklist-scan.md:491:docs/audits/docs-ai-prompts-scan.md:229:docs/audits/docs-ai-pack-scan.md:56:docs/status/trace-index.md
docs/audits/docs-ai-checklist-scan.md:492:docs/audits/docs-ai-prompts-scan.md:230:docs/audits/docs-ai-pack-scan.md:64:## Command: rg -n "CURRENT_STATUS|AI_READ_ME|TECH_CONTEXT|README_AI|SSOT|status\.json|trace-index|handoff|HANDOFF|decisions|LOCKS|glossary|context pack" docs -S
docs/audits/docs-ai-checklist-scan.md:493:docs/audits/docs-ai-prompts-scan.md:231:docs/audits/docs-ai-pack-scan.md:66:docs/decisions/decisions.md:6:- 2026-02-17: contracts additive-only (see contracts policy SSOT where applicable) / Prevents breaking downstream consumers / Contract evolution must preserve compatibility.
docs/audits/docs-ai-checklist-scan.md:494:docs/audits/docs-ai-prompts-scan.md:232:docs/audits/docs-ai-pack-scan.md:67:docs/decisions/decisions.md:7:- 2026-02-17: docs canonical入口 is `docs/context/README_AI.md` / Single entrypoint prevents SSOT ambiguity / AI and human operators start from one canonical path.
docs/audits/docs-ai-checklist-scan.md:495:docs/audits/docs-ai-prompts-scan.md:233:docs/audits/docs-ai-pack-scan.md:68:docs/decisions/decisions.md:8:- 2026-02-17: stopping requires `docs/handoff/HANDOFF.json` update / Multi-agent and interrupted work need explicit continuity state / No stop/handoff without handoff state refresh.
docs/audits/docs-ai-checklist-scan.md:498:docs/audits/docs-ai-prompts-scan.md:242:docs/audits/docs-ai-pack-scan.md:78:docs/specs/crosscut/parallel_task_safety_spec.md:8:- `docs/rules/task-generation-policy.md` (task generation / Multi-AI / Credit-out / HANDOFF / status / decisions / trace)
docs/audits/docs-ai-checklist-scan.md:499:docs/audits/docs-ai-prompts-scan.md:244:docs/audits/docs-ai-pack-scan.md:80:docs/status/trace-index.md:1:# Trace Index (SSOT)
docs/audits/docs-ai-checklist-scan.md:500:docs/audits/docs-ai-prompts-scan.md:245:docs/audits/docs-ai-pack-scan.md:81:docs/status/trace-index.json:16:        "docs/context/README_AI.md",
docs/audits/docs-ai-checklist-scan.md:501:docs/audits/docs-ai-prompts-scan.md:246:docs/audits/docs-ai-pack-scan.md:82:docs/status/trace-index.json:17:        "docs/status/status.json",
docs/audits/docs-ai-checklist-scan.md:502:docs/audits/docs-ai-prompts-scan.md:247:docs/audits/docs-ai-pack-scan.md:83:docs/status/trace-index.json:18:        "docs/handoff/HANDOFF.json",
docs/audits/docs-ai-checklist-scan.md:503:docs/audits/docs-ai-prompts-scan.md:248:docs/audits/docs-ai-pack-scan.md:84:docs/status/trace-index.json:19:        "docs/decisions/decisions.md"
docs/audits/docs-ai-checklist-scan.md:504:docs/audits/docs-ai-prompts-scan.md:251:docs/audits/docs-ai-pack-scan.md:87:docs/status/decisions.md:1:# Decisions Log (SSOT)
docs/audits/docs-ai-checklist-scan.md:505:docs/audits/docs-ai-prompts-scan.md:254:docs/audits/docs-ai-pack-scan.md:90:docs/context/README_AI.md:8:2. Runtime status: `docs/status/status.json`
docs/audits/docs-ai-checklist-scan.md:506:docs/audits/docs-ai-prompts-scan.md:255:docs/audits/docs-ai-pack-scan.md:91:docs/context/README_AI.md:9:3. Handoff state: `docs/status/HANDOFF.json`
docs/audits/docs-ai-checklist-scan.md:507:docs/audits/docs-ai-prompts-scan.md:256:docs/audits/docs-ai-pack-scan.md:92:docs/context/README_AI.md:10:4. Decision baseline: `docs/status/decisions.md`
docs/audits/docs-ai-checklist-scan.md:508:docs/audits/docs-ai-prompts-scan.md:258:docs/audits/docs-ai-pack-scan.md:94:docs/context/README_AI.md:17:- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/status/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
docs/audits/docs-ai-checklist-scan.md:509:docs/audits/docs-ai-prompts-scan.md:259:docs/audits/docs-ai-pack-scan.md:95:docs/context/README_AI.md:18:- **Single active task:** only one active task is tracked in `docs/status/status.json`.
docs/audits/docs-ai-checklist-scan.md:510:docs/audits/docs-ai-prompts-scan.md:260:docs/audits/docs-ai-pack-scan.md:96:docs/context/README_AI.md:19:- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
docs/audits/docs-ai-checklist-scan.md:511:docs/audits/docs-ai-prompts-scan.md:261:docs/audits/docs-ai-pack-scan.md:97:docs/context/README_AI.md:20:- **LOCK areas:** lock-sensitive areas include contracts/ci/infra/lockfiles and shared-docs (`docs/context/**`, `docs/rules/**`, docs hubs). Declare `Required Locks` in task cards.
docs/audits/docs-ai-checklist-scan.md:512:docs/audits/docs-ai-prompts-scan.md:262:docs/audits/docs-ai-pack-scan.md:98:docs/context/README_AI.md:21:- **Trace SSOT:** `docs/status/trace-index.md` is the canonical place for PR/commit/evidence links.
docs/audits/docs-ai-checklist-scan.md:513:docs/audits/docs-ai-prompts-scan.md:263:docs/audits/docs-ai-pack-scan.md:99:docs/context/README_AI.md:22:- **Decisions are binding:** use `docs/status/decisions.md` to fix assumptions and supersede only via explicit new decision entries.
docs/audits/docs-ai-checklist-scan.md:514:docs/audits/docs-ai-prompts-scan.md:266:docs/audits/docs-ai-pack-scan.md:102:docs/rules/task-generation-policy.md:10:2. `docs/status/status.json`
docs/audits/docs-ai-checklist-scan.md:515:docs/audits/docs-ai-prompts-scan.md:267:docs/audits/docs-ai-pack-scan.md:103:docs/rules/task-generation-policy.md:11:3. `docs/status/HANDOFF.json`
docs/audits/docs-ai-checklist-scan.md:516:docs/audits/docs-ai-prompts-scan.md:268:docs/audits/docs-ai-pack-scan.md:105:docs/rules/task-generation-policy.md:19:- Active task ownership and lifecycle are tracked in `docs/status/status.json`.
docs/audits/docs-ai-checklist-scan.md:517:docs/audits/docs-ai-prompts-scan.md:269:docs/audits/docs-ai-pack-scan.md:106:docs/rules/task-generation-policy.md:45:- Canonical runtime status must be updated through `docs/status/status.json` only.
docs/audits/docs-ai-checklist-scan.md:518:docs/audits/docs-ai-prompts-scan.md:271:docs/audits/docs-ai-pack-scan.md:108:docs/rules/task-generation-policy.md:52:### 4.3 Mandatory HANDOFF update on stop/switch
docs/audits/docs-ai-checklist-scan.md:519:docs/audits/docs-ai-prompts-scan.md:272:docs/audits/docs-ai-pack-scan.md:109:docs/rules/task-generation-policy.md:54:Before stopping or switching owner, update `docs/status/HANDOFF.json` with all of:
docs/audits/docs-ai-checklist-scan.md:520:docs/audits/docs-ai-prompts-scan.md:273:docs/audits/docs-ai-pack-scan.md:110:docs/rules/task-generation-policy.md:66:  - an update to `docs/status/status.json`, **or**
docs/audits/docs-ai-checklist-scan.md:521:docs/audits/docs-ai-prompts-scan.md:274:docs/audits/docs-ai-pack-scan.md:112:docs/rules/task-generation-policy.md:78:- `docs/status/trace-index.md` is the SSOT for trace links (PRs, commits, issues, run artifacts).
docs/audits/docs-ai-checklist-scan.md:522:docs/audits/docs-ai-prompts-scan.md:275:docs/audits/docs-ai-pack-scan.md:113:docs/rules/task-generation-policy.md:80:- Other docs may include convenience links, but must treat trace-index as canonical.
docs/audits/docs-ai-checklist-scan.md:524:docs/audits/docs-ai-prompts-scan.md:279:docs/audits/docs-ai-pack-scan.md:117:docs/handoff/HANDOFF.json:8:    "On next docs task, update status.json and trace-index.json first.",
docs/audits/docs-ai-checklist-scan.md:525:docs/audits/docs-ai-prompts-scan.md:280:docs/audits/docs-ai-pack-scan.md:118:docs/handoff/HANDOFF.json:9:    "If stopping mid-task, update this handoff file before exit."
docs/audits/docs-ai-checklist-scan.md:526:docs/audits/docs-ai-prompts-scan.md:281:docs/audits/docs-ai-pack-scan.md:119:docs/handoff/HANDOFF.json:16:    "cat docs/status/status.json",
docs/audits/docs-ai-checklist-scan.md:527:docs/audits/docs-ai-prompts-scan.md:282:docs/audits/docs-ai-pack-scan.md:120:docs/handoff/HANDOFF.json:17:    "cat docs/status/trace-index.json"
docs/audits/docs-ai-checklist-scan.md:529:docs/audits/docs-ai-prompts-scan.md:285:docs/audits/docs-ai-pack-scan.md:123:docs/audits/docs-os-existing-file-scan.md:51:## Command: rg -n "README_AI|SSOT|handoff|HANDOFF|decision|decisions|status\.json|trace-index|LOCKS|glossary|canonical|North Star" docs -S
docs/audits/docs-ai-checklist-scan.md:537:docs/audits/docs-ai-prompts-scan.md:318:docs/audits/docs-ai-pack-scan.md:157:docs/decisions/records/docs-rules-unify-decision.md:6:- Reason: existing rule content is concentrated in `docs/specs/crosscut/parallel_task_safety_spec.md`; new required operational topics (Multi-AI / Credit-out / HANDOFF / status / trace / decisions / LOCK policy) should be explicit and discoverable under a dedicated operations-rule path.
docs/audits/docs-ai-checklist-scan.md:538:docs/audits/docs-ai-prompts-scan.md:319:docs/audits/docs-ai-pack-scan.md:158:docs/decisions/records/docs-rules-unify-decision.md:14:| Multi-AI / Credit-out / Handoff | `docs/specs/crosscut/parallel_task_safety_spec.md` (partial task-generation section only) | `docs/rules/task-generation-policy.md` | merge + extend | Add stop protocol + mandatory HANDOFF update semantics. |
docs/audits/docs-ai-checklist-scan.md:539:docs/audits/docs-ai-prompts-scan.md:320:docs/audits/docs-ai-pack-scan.md:159:docs/decisions/records/docs-rules-unify-decision.md:16:| Decision log policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Standardize decision fixation via `docs/status/decisions.md`. |
docs/audits/docs-ai-checklist-scan.md:540:docs/audits/docs-ai-prompts-scan.md:321:docs/audits/docs-ai-pack-scan.md:160:docs/decisions/records/docs-rules-unify-decision.md:17:| Traceability policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Set `docs/status/trace-index.md` as trace SSOT, avoid scattered links. |
docs/audits/docs-ai-checklist-scan.md:541:docs/audits/docs-ai-prompts-scan.md:322:docs/audits/docs-ai-pack-scan.md:161:docs/decisions/records/docs-rules-unify-decision.md:22:- `docs/context/README_AI.md` is created/updated as must-read entrypoint for AI operators.
docs/audits/docs-ai-checklist-scan.md:543:docs/audits/docs-ai-prompts-scan.md:324:docs/audits/docs-ai-pack-scan.md:163:docs/decisions/records/docs-os-consolidation-decision.md:5:| SSOT entry for Docs Development OS | `docs/README.md` (general docs index), `docs/specs/crosscut/parallel_task_safety_spec.md` (parallel safety SSOT note), `docs/audits/docs-content-overlap.md` (canonicalization guidance) | `docs/context/README_AI.md` | **Merge** | Existing docs already describe partial SSOT/canonical concepts, but no single AI-first entrypoint exists. Create one canonical entrance and link all SSOT roles there. |
docs/audits/docs-ai-checklist-scan.md:544:docs/audits/docs-ai-prompts-scan.md:325:docs/audits/docs-ai-pack-scan.md:164:docs/decisions/records/docs-os-consolidation-decision.md:6:| Machine-readable current status | No direct candidate found in `docs/**` (`status.json` not present) | `docs/status/status.json` | **Keep (new canonical)** | Add a dedicated structured status file for active epic/task, locks, next actions, and PR state. |
docs/audits/docs-ai-checklist-scan.md:545:docs/audits/docs-ai-prompts-scan.md:326:docs/audits/docs-ai-pack-scan.md:165:docs/decisions/records/docs-os-consolidation-decision.md:7:| Handoff protocol | No direct candidate found in `docs/**` (`HANDOFF.json` not present) | `docs/handoff/HANDOFF.json` | **Keep (new canonical)** | Add explicit stop/credit-out handoff file with required fields for multi-AI continuity. |
docs/audits/docs-ai-checklist-scan.md:546:docs/audits/docs-ai-prompts-scan.md:327:docs/audits/docs-ai-pack-scan.md:167:docs/decisions/records/docs-os-consolidation-decision.md:9:| Traceability index | No direct candidate found in `docs/**` (`trace-index.json` absent) | `docs/status/trace-index.json` | **Keep (new canonical)** | Introduce one machine-readable trace SSOT for req↔progress↔PR/commit/spec/runbook/verification links. |
docs/audits/docs-ai-checklist-scan.md:547:docs/audits/docs-ai-prompts-scan.md:328:docs/audits/docs-ai-pack-scan.md:168:docs/decisions/records/docs-os-consolidation-decision.md:12:- No existing same-role canonical files were found for `status.json`, `HANDOFF.json`, `decisions.md`, or `trace-index.json`, so no stub replacement was required.
docs/audits/docs-ai-checklist-scan.md:548:docs/audits/docs-ai-prompts-scan.md:329:docs/audits/docs-ai-pack-scan.md:169:docs/decisions/records/docs-os-consolidation-decision.md:13:- `docs/README.md` remains a general docs hub; `docs/context/README_AI.md` becomes the canonical AI operating entrypoint and references role-specific SSOT paths.
docs/audits/docs-ai-checklist-scan.md:549:docs/audits/docs-ai-prompts-scan.md:337:docs/audits/docs-ai-pack-scan.md:177:docs/audits/docs-rules-unify-scan.md:52:## 3) `rg -n "parallel-task-safety|タスク生成|task generation|Codexアジャイル|致命的事故|guardrail|LOCK:|handoff|HANDOFF|status\.json|trace-index|decisions|SSOT|README_AI" docs -S`
docs/audits/docs-ai-checklist-scan.md:555:docs/audits/docs-ai-prompts-scan.md:359:docs/audits/docs-ai-pack-scan.md:204:docs/status/status.json
docs/audits/docs-ai-checklist-scan.md:556:docs/audits/docs-ai-prompts-scan.md:360:docs/audits/docs-rules-unify-scan.md:52:## 3) `rg -n "parallel-task-safety|タスク生成|task generation|Codexアジャイル|致命的事故|guardrail|LOCK:|handoff|HANDOFF|status\.json|trace-index|decisions|SSOT|README_AI" docs -S`
```

## Command: find docs -maxdepth 5 -type f \( -iname "*task*gen*" -o -iname "*タスク生成*" -o -iname "*parallel*task*safety*" -o -iname "*rules*" -o -iname "*policy*" -o -iname "*readme*ai*" \) -print
```
docs/audits/docs-rules-unify-scan.md
docs/audits/docs-taskgen-enforce-scan.md
docs/decisions/records/docs-rules-unify-decision.md
docs/rules/task-generation-policy.md
docs/context/README_AI.md
docs/specs/crosscut/parallel_task_safety_spec.md
```
