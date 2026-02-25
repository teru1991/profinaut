# DOC-AI-PROMPT Scan

## Command: git status --short
```
?? docs/audits/docs-ai-prompts-scan.md
?? services/marketdata-rs/Cargo.lock
```

## Command: git ls-tree -r --name-only HEAD docs
```
docs/README.md
docs/context/README_AI.md
docs/context/TECH_CONTEXT.md
docs/assumptions.md
docs/decisions/records/docs-ai-pack-decision.md
docs/audits/docs-ai-pack-scan.md
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

## Command: rg -n "magic prompt|プロンプト|prompt template|Planner|Builder|Verifier|Reviewer|Copilot|Claude|Gemini|ChatGPT|README_AI|SSOT|handoff|HANDOFF|status\.json|trace-index|LOCK" docs -S
```
docs/decisions/decisions.md:6:- 2026-02-17: contracts additive-only (see contracts policy SSOT where applicable) / Prevents breaking downstream consumers / Contract evolution must preserve compatibility.
docs/decisions/decisions.md:7:- 2026-02-17: docs canonical入口 is `docs/context/README_AI.md` / Single entrypoint prevents SSOT ambiguity / AI and human operators start from one canonical path.
docs/decisions/decisions.md:8:- 2026-02-17: stopping requires `docs/handoff/HANDOFF.json` update / Multi-agent and interrupted work need explicit continuity state / No stop/handoff without handoff state refresh.
docs/workplan/ultimate-gold-implementation-feature-list.md:51:### UGF-0-023 変更管理SSOT（DecisionLog/Assumptions/ChangeLog運用）
docs/workplan/ultimate-gold-implementation-feature-list.md:63:| Notes/Links | `0-7/0-9 変更管理SSOT` |
docs/workplan/ultimate-gold-implementation-feature-list.md:111:- **UGF-0-004** 中央Policy/Risk Gate最終判定（ALLOW/BLOCK/THROTTLE/REDUCE_ONLY/CLOSE_ONLY/FLATTEN/HALT）
docs/workplan/ultimate-gold-implementation-feature-list.md:135:- **UGF-A-004** PRテンプレ必須項目（TaskID/Scope/Depends-on/Flags/LOCK/リスク）
docs/workplan/ultimate-gold-implementation-feature-list.md:195:## G. Human-in-the-loop / Copilot
docs/assumptions.md:13:1. OpenAPI uses version `1.0.0` as the first SSOT baseline for V2.5+ bootstrap.
docs/README.md:6:- AI SSOT entry (canonical docs development OS): [`SSOT/README_AI.md`](SSOT/README_AI.md)
docs/README.md:13:- Rules SSOT (task generation + parallel safety): [`rules/task-generation-policy.md`](rules/task-generation-policy.md), [`rules/parallel-development-safety.md`](rules/parallel-development-safety.md)
docs/README.md:14:- AI operator onboarding SSOT: [`SSOT/README_AI.md`](SSOT/README_AI.md)
docs/plans/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/plans/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/specs/crosscut/parallel_task_safety_spec.md:8:- `docs/rules/task-generation-policy.md` (task generation / Multi-AI / Credit-out / HANDOFF / status / decisions / trace)
docs/specs/crosscut/parallel_task_safety_spec.md:12:- `docs/context/README_AI.md`
docs/status/trace-index.md:1:# Trace Index (SSOT)
docs/status/trace-index.json:16:        "docs/context/README_AI.md",
docs/status/trace-index.json:17:        "docs/status/status.json",
docs/status/trace-index.json:18:        "docs/handoff/HANDOFF.json",
docs/status/ultimate-gold-progress-check.md:39:### Current LOCK Owners（排他管理）
docs/status/ultimate-gold-progress-check.md:54:| UG-00 | 全体NFR | P0 | In Progress | 55% | Contracts SSOT、idempotency、dead-man、監査ログ、health/capabilities基盤 | SAFE_MODE統一運用、SoT優先順位の明文化、replay決定性の全体保証 |
docs/status/ultimate-gold-progress-check.md:89:  - Contracts SSOT
docs/status/ultimate-gold-progress-check.md:115:1. **UG-00/UG-D**: SAFE_MODE遷移と許可操作（ALLOW/BLOCK/THROTTLE/REDUCE_ONLY/CLOSE_ONLY/FLATTEN/HALT）を契約と実装で統一。
docs/status/ultimate-gold-progress-check.md:130:| 2026-02-15 | Repo Snapshot/LOCK/Open PR常設欄、PR証跡URL/SHA欄を追加 | Codex |
docs/status/decisions.md:1:# Decisions Log (SSOT)
docs/status/CURRENT_STATUS.md:3:_Last updated: 1970-01-01T00:00:00Z (from `docs/status/status.json`)_
docs/status/CURRENT_STATUS.md:6:> **SSOT is `docs/status/status.json`**.
docs/status/CURRENT_STATUS.md:11:- Active epic: _not specified in `status.json`_
docs/status/CURRENT_STATUS.md:17:- _No open PRs recorded in `status.json`._
docs/status/CURRENT_STATUS.md:21:- _No locks recorded in `status.json`._
docs/status/CURRENT_STATUS.md:25:- _None explicitly recorded in `status.json`._
docs/status/CURRENT_STATUS.md:29:- Update `docs/status/status.json` when a task owner is assigned.
docs/status/CURRENT_STATUS.md:31:- Add progress evidence under `docs/status/progress-updates/` and trace links in `docs/status/trace-index.md`/`.json` as work proceeds.
docs/status/CURRENT_STATUS.md:36:- `docs/status/status.json` (machine-readable SSOT)
docs/status/progress-updates/UG-P0-101.md:18:    "decision": "BLOCK",
docs/status/progress-updates/UG-P0-101.md:19:    "reason_code": "SAFE_MODE_BLOCKED",
docs/status/progress-updates/UG-P0-102.md:7:  - `SAFE_MODE_BLOCKED`
docs/status/progress-updates/UG-P0-110.md:10:1. `SAFE_MODE` / `HALTED` hard safety flags (`SAFE_MODE_BLOCKED`)
docs/status/progress-updates/UG-P0-110.md:20:- `decision`: `ALLOW | BLOCK | THROTTLE | REDUCE_ONLY | CLOSE_ONLY | FLATTEN | HALT`
docs/rules/task-generation-policy.md:1:# Task Generation Policy (SSOT)
docs/rules/task-generation-policy.md:9:1. `docs/context/README_AI.md`
docs/rules/task-generation-policy.md:10:2. `docs/status/status.json`
docs/rules/task-generation-policy.md:11:3. `docs/status/HANDOFF.json`
docs/rules/task-generation-policy.md:19:- Active task ownership and lifecycle are tracked in `docs/status/status.json`.
docs/rules/task-generation-policy.md:31:- `LOCK`
docs/rules/task-generation-policy.md:45:- Canonical runtime status must be updated through `docs/status/status.json` only.
docs/rules/task-generation-policy.md:50:When an operator must stop early (time, token, context, incident, rate limits), that stop is treated as **Credit-out** and requires a handoff update before exiting.
docs/rules/task-generation-policy.md:52:### 4.3 Mandatory HANDOFF update on stop/switch
docs/rules/task-generation-policy.md:54:Before stopping or switching owner, update `docs/status/HANDOFF.json` with all of:
docs/rules/task-generation-policy.md:66:  - an update to `docs/status/status.json`, **or**
docs/rules/task-generation-policy.md:78:- `docs/status/trace-index.md` is the SSOT for trace links (PRs, commits, issues, run artifacts).
docs/rules/task-generation-policy.md:80:- Other docs may include convenience links, but must treat trace-index as canonical.
docs/specs/crosscut/safety_interlock_spec.md:1:# Parallel Development Safety (SSOT)
docs/specs/crosscut/safety_interlock_spec.md:27:## 3. LOCK policy
docs/specs/crosscut/safety_interlock_spec.md:29:### 3.1 LOCK / semi-LOCK behavior
docs/specs/crosscut/safety_interlock_spec.md:31:- **LOCK area**: only one active PR may touch that area.
docs/specs/crosscut/safety_interlock_spec.md:32:- **Semi-LOCK area**: parallel PRs allowed only with zero file overlap and explicit coordination.
docs/specs/crosscut/safety_interlock_spec.md:43:- shared-docs (`docs/context/**`, `docs/rules/**`, docs hubs/indexes)
docs/specs/crosscut/safety_interlock_spec.md:49:- `Required Locks: docs/context/**`
docs/specs/crosscut/safety_interlock_spec.md:72:- [ ] No overlap in LOCK/semi-LOCK areas
docs/context/README_AI.md:1:# README_AI (SSOT entrypoint for operators)
docs/context/README_AI.md:7:1. This file: `docs/context/README_AI.md` (AI constitution / entrypoint)
docs/context/README_AI.md:8:2. Human-readable status snapshot (non-SSOT): `docs/status/CURRENT_STATUS.md`
docs/context/README_AI.md:9:3. Runtime status SSOT: `docs/status/status.json`
docs/context/README_AI.md:10:4. Tech context hub (non-SSOT links-only): `docs/context/TECH_CONTEXT.md`
docs/context/README_AI.md:11:5. Handoff state: `docs/status/HANDOFF.json`
docs/context/README_AI.md:13:7. Trace SSOT: `docs/status/trace-index.md`
docs/context/README_AI.md:14:8. Rules SSOT:
docs/context/README_AI.md:19:## SSOT boundaries (important)
docs/context/README_AI.md:21:- **Canonical AI entrypoint:** `docs/context/README_AI.md`
docs/context/README_AI.md:22:- **Status SSOT:** `docs/status/status.json`
docs/context/README_AI.md:23:- **`docs/status/CURRENT_STATUS.md` is a summary only (non-SSOT).**
docs/context/README_AI.md:24:- **`docs/context/TECH_CONTEXT.md` is a links hub only (non-SSOT).**
docs/context/README_AI.md:29:- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/status/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
docs/context/README_AI.md:30:- **Single active task:** only one active task is tracked in `docs/status/status.json`.
docs/context/README_AI.md:31:- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
docs/context/README_AI.md:32:- **LOCK areas:** lock-sensitive areas include contracts/ci/infra/lockfiles and shared-docs (`docs/context/**`, `docs/rules/**`, docs hubs). Declare `Required Locks` in task cards.
docs/context/README_AI.md:33:- **Trace SSOT:** `docs/status/trace-index.md` is the canonical place for PR/commit/evidence links.
docs/context/TECH_CONTEXT.md:4:It is **not** a full technical SSOT and must not duplicate long specs.
docs/context/TECH_CONTEXT.md:10:- AI entrypoint: `docs/context/README_AI.md`
docs/context/TECH_CONTEXT.md:12:- Runtime status SSOT: `docs/status/status.json`
docs/context/TECH_CONTEXT.md:13:- Trace index: `docs/status/trace-index.md` and `docs/status/trace-index.json`
docs/context/TECH_CONTEXT.md:15:- Handoff state: `docs/status/HANDOFF.json` (legacy/alt path may exist at `docs/handoff/HANDOFF.json`)
docs/context/TECH_CONTEXT.md:42:- When in doubt, follow linked SSOT/authoritative docs and record decisions in `docs/status/decisions.md`.
docs/handoff/HANDOFF.json:8:    "On next docs task, update status.json and trace-index.json first.",
docs/handoff/HANDOFF.json:9:    "If stopping mid-task, update this handoff file before exit."
docs/handoff/HANDOFF.json:16:    "cat docs/status/status.json",
docs/handoff/HANDOFF.json:17:    "cat docs/status/trace-index.json"
docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/changelog.md:220:- Updated README with contract SSOT and validation commands.
docs/audits/docs-os-existing-file-scan.md:51:## Command: rg -n "README_AI|SSOT|handoff|HANDOFF|decision|decisions|status\.json|trace-index|LOCKS|glossary|canonical|North Star" docs -S
docs/audits/docs-os-existing-file-scan.md:53:docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/audits/docs-os-existing-file-scan.md:54:docs/changelog.md:220:- Updated README with contract SSOT and validation commands.
docs/audits/docs-os-existing-file-scan.md:60:docs/specs/crosscut/parallel_task_safety_spec.md:1:# Parallel Task Safety Spec (SSOT)
docs/audits/docs-os-existing-file-scan.md:64:docs/audits/docs-content-overlap.md:34:| Area ID | Suspected overlap area | Current files involved | Proposed canonical (SSOT) | Rationale | Follow-up touch plan (next PRs) |
docs/audits/docs-os-existing-file-scan.md:65:docs/audits/docs-content-overlap.md:37:| OVL-02 | Ultimate Gold planning/progress duplication | `docs/workplan/ultimate-gold-implementation-feature-list.md`, `docs/status/ultimate-gold-progress-check.md`, `docs/status/progress-updates/UG-P0-*.md` | `docs/workplan/ultimate-gold-implementation-feature-list.md` = requirements/catalog SSOT; `docs/status/ultimate-gold-progress-check.md` = current status dashboard SSOT; `docs/status/progress-updates/*.md` = append-only evidence logs | The workplan and progress-check both contain requirement mapping + milestone status narrative; risk of drift is high. Split roles explicitly: "what should exist" vs "what currently exists" vs "event/evidence log". | 1) Remove requirement restatement from progress-check; keep live status only. 2) In progress updates, enforce short template + links to PR/commit only. 3) Add explicit cross-links among the 3 layers. |
docs/audits/docs-os-existing-file-scan.md:66:docs/audits/docs-content-overlap.md:38:| OVL-03 | Dangerous operation rule definition split | `docs/specs/crosscut/dangerous_ops_taxonomy.md`, `docs/specs/crosscut/dangerous_ops_confirmation.md` | `docs/specs/crosscut/dangerous_ops_taxonomy.md` as policy/rule SSOT; `docs/specs/crosscut/dangerous_ops_confirmation.md` as UX/API confirmation flow spec only | Taxonomy (what is dangerous and why) and confirmation (how operator confirms) should be separated but non-overlapping. Current boundary is easy to blur. | 1) Keep taxonomy doc free of UI workflow details. 2) Keep confirmation doc free of category/policy duplication; only reference taxonomy IDs. |
docs/audits/docs-os-existing-file-scan.md:67:docs/audits/docs-content-overlap.md:39:| OVL-04 | Execution specification layering | `docs/context/notes/execution.md`, `docs/context/notes/execution-gmo.md` | `docs/context/notes/execution.md` as provider-agnostic execution contract/behavior SSOT; `docs/context/notes/execution-gmo.md` as GMO adapter profile | Provider-specific constraints can leak into generic execution spec, causing multi-exchange evolution pain. | 1) Extract any GMO-only semantics out of generic spec into GMO profile. 2) Keep generic spec with neutral terms and extension hooks. |
docs/audits/docs-os-existing-file-scan.md:68:docs/audits/docs-content-overlap.md:40:| OVL-05 | Bots API/UX status model duplication | `docs/specs/controlplane-bots.md`, `docs/specs/ui-bots.md` | `docs/specs/controlplane-bots.md` as API schema/status semantics SSOT; `docs/specs/ui-bots.md` as UI behavior/rendering SSOT | Bot status fields (e.g., degraded reasons, states) are often repeated in both docs. API semantics should live in controlplane spec, UI doc should reference them. | 1) In UI spec, replace duplicated field definitions with links/brief mapping tables. 2) Keep authoritative enums/field contracts in controlplane spec. |
docs/audits/docs-os-existing-file-scan.md:69:docs/audits/docs-content-overlap.md:41:| OVL-06 | Paper E2E runbook duplication | `docs/runbooks/e2e-smoke-runbook.md`, `docs/runbooks/paper_e2e.md` | `docs/runbooks/e2e-smoke-runbook.md` as "one-command" operational runbook SSOT; `docs/runbooks/paper_e2e.md` as deep troubleshooting/reference | Both cover paper path checks; one should be quick path and the other should be expanded diagnostics. Distinguish by depth and audience. | 1) Ensure smoke runbook contains only happy-path + short triage. 2) Keep paper_e2e as detailed manual procedures and edge-case diagnostics. |
docs/audits/docs-os-existing-file-scan.md:70:docs/audits/docs-content-overlap.md:42:| OVL-07 | Governance baseline duplication | `docs/assumptions.md`, `docs/plans/roadmap.md`, `README.md` | `docs/assumptions.md` as baseline assumptions/constraints SSOT | Guardrails and defaults appear in multiple places; assumptions should be centralized and referenced elsewhere. | 1) Keep roadmap guardrails short and link to assumptions. 2) Keep README to brief security/baseline pointers only. |
docs/audits/docs-os-existing-file-scan.md:81:docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/docs-os-existing-file-scan.md:84:docs/assumptions.md:13:1. OpenAPI uses version `1.0.0` as the first SSOT baseline for V2.5+ bootstrap.
docs/audits/docs-os-existing-file-scan.md:86:docs/status/progress-updates/UG-P0-110.md:20:- `decision`: `ALLOW | BLOCK | THROTTLE | REDUCE_ONLY | CLOSE_ONLY | FLATTEN | HALT`
docs/audits/docs-os-existing-file-scan.md:90:docs/status/progress-updates/UG-P0-101.md:18:    "decision": "BLOCK",
docs/audits/docs-os-existing-file-scan.md:92:docs/status/ultimate-gold-progress-check.md:54:| UG-00 | 全体NFR | P0 | In Progress | 55% | Contracts SSOT、idempotency、dead-man、監査ログ、health/capabilities基盤 | SAFE_MODE統一運用、SoT優先順位の明文化、replay決定性の全体保証 |
docs/audits/docs-os-existing-file-scan.md:93:docs/status/ultimate-gold-progress-check.md:89:  - Contracts SSOT
docs/audits/docs-os-existing-file-scan.md:94:docs/plans/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/audits/docs-os-existing-file-scan.md:95:docs/plans/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/audits/docs-os-existing-file-scan.md:96:docs/README.md:12:- Parallel task safety (SSOT for parallel development safety): [`specs/parallel-task-safety.md`](specs/parallel-task-safety.md)
docs/audits/docs-os-existing-file-scan.md:98:docs/workplan/ultimate-gold-implementation-feature-list.md:51:### UGF-0-023 変更管理SSOT（DecisionLog/Assumptions/ChangeLog運用）
docs/audits/docs-os-existing-file-scan.md:101:docs/workplan/ultimate-gold-implementation-feature-list.md:63:| Notes/Links | `0-7/0-9 変更管理SSOT` |
docs/audits/docs-os-existing-file-scan.md:105:## Command: find docs -maxdepth 4 -type f \( -iname "*readme*" -o -iname "*ssot*" -o -iname "*handoff*" -o -iname "*decision*" -o -iname "*status*.json" -o -iname "*trace*" -o -iname "*locks*" -o -iname "*glossary*" \) -print
docs/audits/docs-audit-report.md:38:- `docs/specs/crosscut/parallel_task_safety_spec.md` → `rules/spec (parallel safety SSOT)`
docs/audits/docs-audit-report.md:72:### 判定: **部分的に固定（入口はあるが単一SSOTとしては未固定）**
docs/audits/docs-audit-report.md:83:### B-2. Epic / MRU / Contracts-SSOT / 運用原則の接続性
docs/audits/docs-audit-report.md:90:  - `roadmap` に Contracts SSOT guardrail がある。
docs/audits/docs-audit-report.md:93:  - 「Contractsが絶対SSOT」であることは docs内に分散記述され、統治入口（governance hub）が未固定。
docs/audits/docs-audit-report.md:100:2. **Decision Log の明示的SSOT入口**
docs/audits/docs-audit-report.md:104:4. **handoff運用の明示文書**
docs/audits/docs-audit-report.md:128:  - **監査レポート:** `docs/audits/*`（非SSOT）
docs/audits/docs-audit-report.md:161:  - **運用原則SSOT:** `docs/specs/crosscut/parallel_task_safety_spec.md`
docs/audits/docs-audit-report.md:194:  - **差分監査:** `ui-current-vs-spec.md`（非SSOT）
docs/audits/docs-audit-report.md:269:rg -i "status|progress|workplan|roadmap|spec|design|decision|handoff|mru" -n docs
docs/decisions/records/docs-rules-unify-decision.md:5:- Chosen approach: **Option A** (`docs/rules/` as rules SSOT namespace).
docs/decisions/records/docs-rules-unify-decision.md:6:- Reason: existing rule content is concentrated in `docs/specs/crosscut/parallel_task_safety_spec.md`; new required operational topics (Multi-AI / Credit-out / HANDOFF / status / trace / decisions / LOCK policy) should be explicit and discoverable under a dedicated operations-rule path.
docs/decisions/records/docs-rules-unify-decision.md:14:| Multi-AI / Credit-out / Handoff | `docs/specs/crosscut/parallel_task_safety_spec.md` (partial task-generation section only) | `docs/rules/task-generation-policy.md` | merge + extend | Add stop protocol + mandatory HANDOFF update semantics. |
docs/decisions/records/docs-rules-unify-decision.md:15:| LOCK policy | `docs/specs/crosscut/parallel_task_safety_spec.md` (LOCK/semi-LOCK partial) | `docs/specs/crosscut/safety_interlock_spec.md` | merge + extend | Add explicit lock areas incl. shared-docs and Required Locks declaration. |
docs/decisions/records/docs-rules-unify-decision.md:16:| Decision log policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Standardize decision fixation via `docs/status/decisions.md`. |
docs/decisions/records/docs-rules-unify-decision.md:17:| Traceability policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Set `docs/status/trace-index.md` as trace SSOT, avoid scattered links. |
docs/decisions/records/docs-rules-unify-decision.md:22:- `docs/context/README_AI.md` is created/updated as must-read entrypoint for AI operators.
docs/decisions/records/docs-ai-pack-decision.md:5:| AI entry (constitution) | `docs/context/README_AI.md` | `docs/context/README_AI.md` | keep/merge | Must remain canonical AI entrypoint. Add links to new non-SSOT context files. |
docs/decisions/records/docs-ai-pack-decision.md:6:| Human-readable current status | `docs/status/status.json` (machine-readable SSOT), no existing markdown summary found | `docs/status/CURRENT_STATUS.md` | create/merge | New file is a thin, human-readable summary only. Explicitly states SSOT is `docs/status/status.json`. |
docs/decisions/records/docs-ai-pack-decision.md:7:| Tech context (links + constraints) | No dedicated tech-context hub found in `docs/` | `docs/context/TECH_CONTEXT.md` | create/merge | Links-only navigation hub to existing specs/runbooks/status artifacts; no full-schema duplication. |
docs/decisions/records/docs-ai-pack-decision.md:8:| Any root-level AI files | None detected | none | avoid | Do not create root-level `AI_READ_ME.md` or equivalent SSOT duplicates. |
docs/decisions/records/docs-ai-pack-decision.md:12:- Keep `docs/context/README_AI.md` as the single AI constitution entrypoint.
docs/decisions/records/docs-ai-pack-decision.md:13:- Treat `docs/status/CURRENT_STATUS.md` as non-SSOT summary derived from `docs/status/status.json`.
docs/decisions/records/docs-ai-pack-decision.md:14:- Keep `docs/context/TECH_CONTEXT.md` as a link hub (not a new canonical source of detailed specs).
docs/audits/repo-progress-audit-2026-02-14.md:1:# Repo Progress Audit (Planner + Repo Auditor)
docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/decisions/records/docs-os-consolidation-decision.md:5:| SSOT entry for Docs Development OS | `docs/README.md` (general docs index), `docs/specs/crosscut/parallel_task_safety_spec.md` (parallel safety SSOT note), `docs/audits/docs-content-overlap.md` (canonicalization guidance) | `docs/context/README_AI.md` | **Merge** | Existing docs already describe partial SSOT/canonical concepts, but no single AI-first entrypoint exists. Create one canonical entrance and link all SSOT roles there. |
docs/decisions/records/docs-os-consolidation-decision.md:6:| Machine-readable current status | No direct candidate found in `docs/**` (`status.json` not present) | `docs/status/status.json` | **Keep (new canonical)** | Add a dedicated structured status file for active epic/task, locks, next actions, and PR state. |
docs/decisions/records/docs-os-consolidation-decision.md:7:| Handoff protocol | No direct candidate found in `docs/**` (`HANDOFF.json` not present) | `docs/handoff/HANDOFF.json` | **Keep (new canonical)** | Add explicit stop/credit-out handoff file with required fields for multi-AI continuity. |
docs/decisions/records/docs-os-consolidation-decision.md:9:| Traceability index | No direct candidate found in `docs/**` (`trace-index.json` absent) | `docs/status/trace-index.json` | **Keep (new canonical)** | Introduce one machine-readable trace SSOT for req↔progress↔PR/commit/spec/runbook/verification links. |
docs/decisions/records/docs-os-consolidation-decision.md:12:- No existing same-role canonical files were found for `status.json`, `HANDOFF.json`, `decisions.md`, or `trace-index.json`, so no stub replacement was required.
docs/decisions/records/docs-os-consolidation-decision.md:13:- `docs/README.md` remains a general docs hub; `docs/context/README_AI.md` becomes the canonical AI operating entrypoint and references role-specific SSOT paths.
docs/audits/docs-content-overlap.md:34:| Area ID | Suspected overlap area | Current files involved | Proposed canonical (SSOT) | Rationale | Follow-up touch plan (next PRs) |
docs/audits/docs-content-overlap.md:37:| OVL-02 | Ultimate Gold planning/progress duplication | `docs/workplan/ultimate-gold-implementation-feature-list.md`, `docs/status/ultimate-gold-progress-check.md`, `docs/status/progress-updates/UG-P0-*.md` | `docs/workplan/ultimate-gold-implementation-feature-list.md` = requirements/catalog SSOT; `docs/status/ultimate-gold-progress-check.md` = current status dashboard SSOT; `docs/status/progress-updates/*.md` = append-only evidence logs | The workplan and progress-check both contain requirement mapping + milestone status narrative; risk of drift is high. Split roles explicitly: "what should exist" vs "what currently exists" vs "event/evidence log". | 1) Remove requirement restatement from progress-check; keep live status only. 2) In progress updates, enforce short template + links to PR/commit only. 3) Add explicit cross-links among the 3 layers. |
docs/audits/docs-content-overlap.md:38:| OVL-03 | Dangerous operation rule definition split | `docs/specs/crosscut/dangerous_ops_taxonomy.md`, `docs/specs/crosscut/dangerous_ops_confirmation.md` | `docs/specs/crosscut/dangerous_ops_taxonomy.md` as policy/rule SSOT; `docs/specs/crosscut/dangerous_ops_confirmation.md` as UX/API confirmation flow spec only | Taxonomy (what is dangerous and why) and confirmation (how operator confirms) should be separated but non-overlapping. Current boundary is easy to blur. | 1) Keep taxonomy doc free of UI workflow details. 2) Keep confirmation doc free of category/policy duplication; only reference taxonomy IDs. |
docs/audits/docs-content-overlap.md:39:| OVL-04 | Execution specification layering | `docs/context/notes/execution.md`, `docs/context/notes/execution-gmo.md` | `docs/context/notes/execution.md` as provider-agnostic execution contract/behavior SSOT; `docs/context/notes/execution-gmo.md` as GMO adapter profile | Provider-specific constraints can leak into generic execution spec, causing multi-exchange evolution pain. | 1) Extract any GMO-only semantics out of generic spec into GMO profile. 2) Keep generic spec with neutral terms and extension hooks. |
docs/audits/docs-content-overlap.md:40:| OVL-05 | Bots API/UX status model duplication | `docs/specs/controlplane-bots.md`, `docs/specs/ui-bots.md` | `docs/specs/controlplane-bots.md` as API schema/status semantics SSOT; `docs/specs/ui-bots.md` as UI behavior/rendering SSOT | Bot status fields (e.g., degraded reasons, states) are often repeated in both docs. API semantics should live in controlplane spec, UI doc should reference them. | 1) In UI spec, replace duplicated field definitions with links/brief mapping tables. 2) Keep authoritative enums/field contracts in controlplane spec. |
docs/audits/docs-content-overlap.md:41:| OVL-06 | Paper E2E runbook duplication | `docs/runbooks/e2e-smoke-runbook.md`, `docs/runbooks/paper_e2e.md` | `docs/runbooks/e2e-smoke-runbook.md` as "one-command" operational runbook SSOT; `docs/runbooks/paper_e2e.md` as deep troubleshooting/reference | Both cover paper path checks; one should be quick path and the other should be expanded diagnostics. Distinguish by depth and audience. | 1) Ensure smoke runbook contains only happy-path + short triage. 2) Keep paper_e2e as detailed manual procedures and edge-case diagnostics. |
docs/audits/docs-content-overlap.md:42:| OVL-07 | Governance baseline duplication | `docs/assumptions.md`, `docs/plans/roadmap.md`, `README.md` | `docs/assumptions.md` as baseline assumptions/constraints SSOT | Guardrails and defaults appear in multiple places; assumptions should be centralized and referenced elsewhere. | 1) Keep roadmap guardrails short and link to assumptions. 2) Keep README to brief security/baseline pointers only. |
docs/audits/docs-ai-pack-scan.md:12:docs/context/README_AI.md
docs/audits/docs-ai-pack-scan.md:24:docs/handoff/HANDOFF.json
docs/audits/docs-ai-pack-scan.md:43:docs/status/HANDOFF.json
docs/audits/docs-ai-pack-scan.md:54:docs/status/status.json
docs/audits/docs-ai-pack-scan.md:55:docs/status/trace-index.json
docs/audits/docs-ai-pack-scan.md:56:docs/status/trace-index.md
docs/audits/docs-ai-pack-scan.md:64:## Command: rg -n "CURRENT_STATUS|AI_READ_ME|TECH_CONTEXT|README_AI|SSOT|status\.json|trace-index|handoff|HANDOFF|decisions|LOCKS|glossary|context pack" docs -S
docs/audits/docs-ai-pack-scan.md:66:docs/decisions/decisions.md:6:- 2026-02-17: contracts additive-only (see contracts policy SSOT where applicable) / Prevents breaking downstream consumers / Contract evolution must preserve compatibility.
docs/audits/docs-ai-pack-scan.md:67:docs/decisions/decisions.md:7:- 2026-02-17: docs canonical入口 is `docs/context/README_AI.md` / Single entrypoint prevents SSOT ambiguity / AI and human operators start from one canonical path.
docs/audits/docs-ai-pack-scan.md:68:docs/decisions/decisions.md:8:- 2026-02-17: stopping requires `docs/handoff/HANDOFF.json` update / Multi-agent and interrupted work need explicit continuity state / No stop/handoff without handoff state refresh.
docs/audits/docs-ai-pack-scan.md:69:docs/workplan/ultimate-gold-implementation-feature-list.md:51:### UGF-0-023 変更管理SSOT（DecisionLog/Assumptions/ChangeLog運用）
docs/audits/docs-ai-pack-scan.md:71:docs/workplan/ultimate-gold-implementation-feature-list.md:63:| Notes/Links | `0-7/0-9 変更管理SSOT` |
docs/audits/docs-ai-pack-scan.md:72:docs/assumptions.md:13:1. OpenAPI uses version `1.0.0` as the first SSOT baseline for V2.5+ bootstrap.
docs/audits/docs-ai-pack-scan.md:73:docs/README.md:6:- AI SSOT entry (canonical docs development OS): [`SSOT/README_AI.md`](SSOT/README_AI.md)
docs/audits/docs-ai-pack-scan.md:74:docs/README.md:13:- Rules SSOT (task generation + parallel safety): [`rules/task-generation-policy.md`](rules/task-generation-policy.md), [`rules/parallel-development-safety.md`](rules/parallel-development-safety.md)
docs/audits/docs-ai-pack-scan.md:75:docs/README.md:14:- AI operator onboarding SSOT: [`SSOT/README_AI.md`](SSOT/README_AI.md)
docs/audits/docs-ai-pack-scan.md:76:docs/plans/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/audits/docs-ai-pack-scan.md:77:docs/plans/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/audits/docs-ai-pack-scan.md:78:docs/specs/crosscut/parallel_task_safety_spec.md:8:- `docs/rules/task-generation-policy.md` (task generation / Multi-AI / Credit-out / HANDOFF / status / decisions / trace)
docs/audits/docs-ai-pack-scan.md:79:docs/specs/crosscut/parallel_task_safety_spec.md:12:- `docs/context/README_AI.md`
docs/audits/docs-ai-pack-scan.md:80:docs/status/trace-index.md:1:# Trace Index (SSOT)
docs/audits/docs-ai-pack-scan.md:81:docs/status/trace-index.json:16:        "docs/context/README_AI.md",
docs/audits/docs-ai-pack-scan.md:82:docs/status/trace-index.json:17:        "docs/status/status.json",
docs/audits/docs-ai-pack-scan.md:83:docs/status/trace-index.json:18:        "docs/handoff/HANDOFF.json",
docs/audits/docs-ai-pack-scan.md:84:docs/status/trace-index.json:19:        "docs/decisions/decisions.md"
docs/audits/docs-ai-pack-scan.md:85:docs/status/ultimate-gold-progress-check.md:54:| UG-00 | 全体NFR | P0 | In Progress | 55% | Contracts SSOT、idempotency、dead-man、監査ログ、health/capabilities基盤 | SAFE_MODE統一運用、SoT優先順位の明文化、replay決定性の全体保証 |
docs/audits/docs-ai-pack-scan.md:86:docs/status/ultimate-gold-progress-check.md:89:  - Contracts SSOT
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
docs/audits/docs-ai-pack-scan.md:100:docs/rules/task-generation-policy.md:1:# Task Generation Policy (SSOT)
docs/audits/docs-ai-pack-scan.md:101:docs/rules/task-generation-policy.md:9:1. `docs/context/README_AI.md`
docs/audits/docs-ai-pack-scan.md:102:docs/rules/task-generation-policy.md:10:2. `docs/status/status.json`
docs/audits/docs-ai-pack-scan.md:103:docs/rules/task-generation-policy.md:11:3. `docs/status/HANDOFF.json`
docs/audits/docs-ai-pack-scan.md:105:docs/rules/task-generation-policy.md:19:- Active task ownership and lifecycle are tracked in `docs/status/status.json`.
docs/audits/docs-ai-pack-scan.md:106:docs/rules/task-generation-policy.md:45:- Canonical runtime status must be updated through `docs/status/status.json` only.
docs/audits/docs-ai-pack-scan.md:107:docs/rules/task-generation-policy.md:50:When an operator must stop early (time, token, context, incident, rate limits), that stop is treated as **Credit-out** and requires a handoff update before exiting.
docs/audits/docs-ai-pack-scan.md:108:docs/rules/task-generation-policy.md:52:### 4.3 Mandatory HANDOFF update on stop/switch
docs/audits/docs-ai-pack-scan.md:109:docs/rules/task-generation-policy.md:54:Before stopping or switching owner, update `docs/status/HANDOFF.json` with all of:
docs/audits/docs-ai-pack-scan.md:110:docs/rules/task-generation-policy.md:66:  - an update to `docs/status/status.json`, **or**
docs/audits/docs-ai-pack-scan.md:112:docs/rules/task-generation-policy.md:78:- `docs/status/trace-index.md` is the SSOT for trace links (PRs, commits, issues, run artifacts).
docs/audits/docs-ai-pack-scan.md:113:docs/rules/task-generation-policy.md:80:- Other docs may include convenience links, but must treat trace-index as canonical.
docs/audits/docs-ai-pack-scan.md:114:docs/specs/crosscut/safety_interlock_spec.md:1:# Parallel Development Safety (SSOT)
docs/audits/docs-ai-pack-scan.md:115:docs/specs/crosscut/safety_interlock_spec.md:43:- shared-docs (`docs/context/**`, `docs/rules/**`, docs hubs/indexes)
docs/audits/docs-ai-pack-scan.md:116:docs/specs/crosscut/safety_interlock_spec.md:49:- `Required Locks: docs/context/**`
docs/audits/docs-ai-pack-scan.md:117:docs/handoff/HANDOFF.json:8:    "On next docs task, update status.json and trace-index.json first.",
docs/audits/docs-ai-pack-scan.md:118:docs/handoff/HANDOFF.json:9:    "If stopping mid-task, update this handoff file before exit."
docs/audits/docs-ai-pack-scan.md:119:docs/handoff/HANDOFF.json:16:    "cat docs/status/status.json",
docs/audits/docs-ai-pack-scan.md:120:docs/handoff/HANDOFF.json:17:    "cat docs/status/trace-index.json"
docs/audits/docs-ai-pack-scan.md:121:docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/audits/docs-ai-pack-scan.md:122:docs/changelog.md:220:- Updated README with contract SSOT and validation commands.
docs/audits/docs-ai-pack-scan.md:123:docs/audits/docs-os-existing-file-scan.md:51:## Command: rg -n "README_AI|SSOT|handoff|HANDOFF|decision|decisions|status\.json|trace-index|LOCKS|glossary|canonical|North Star" docs -S
docs/audits/docs-ai-pack-scan.md:124:docs/audits/docs-os-existing-file-scan.md:53:docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/audits/docs-ai-pack-scan.md:125:docs/audits/docs-os-existing-file-scan.md:54:docs/changelog.md:220:- Updated README with contract SSOT and validation commands.
docs/audits/docs-ai-pack-scan.md:126:docs/audits/docs-os-existing-file-scan.md:60:docs/specs/crosscut/parallel_task_safety_spec.md:1:# Parallel Task Safety Spec (SSOT)
docs/audits/docs-ai-pack-scan.md:127:docs/audits/docs-os-existing-file-scan.md:64:docs/audits/docs-content-overlap.md:34:| Area ID | Suspected overlap area | Current files involved | Proposed canonical (SSOT) | Rationale | Follow-up touch plan (next PRs) |
docs/audits/docs-ai-pack-scan.md:128:docs/audits/docs-os-existing-file-scan.md:65:docs/audits/docs-content-overlap.md:37:| OVL-02 | Ultimate Gold planning/progress duplication | `docs/workplan/ultimate-gold-implementation-feature-list.md`, `docs/status/ultimate-gold-progress-check.md`, `docs/status/progress-updates/UG-P0-*.md` | `docs/workplan/ultimate-gold-implementation-feature-list.md` = requirements/catalog SSOT; `docs/status/ultimate-gold-progress-check.md` = current status dashboard SSOT; `docs/status/progress-updates/*.md` = append-only evidence logs | The workplan and progress-check both contain requirement mapping + milestone status narrative; risk of drift is high. Split roles explicitly: "what should exist" vs "what currently exists" vs "event/evidence log". | 1) Remove requirement restatement from progress-check; keep live status only. 2) In progress updates, enforce short template + links to PR/commit only. 3) Add explicit cross-links among the 3 layers. |
docs/audits/docs-ai-pack-scan.md:129:docs/audits/docs-os-existing-file-scan.md:66:docs/audits/docs-content-overlap.md:38:| OVL-03 | Dangerous operation rule definition split | `docs/specs/crosscut/dangerous_ops_taxonomy.md`, `docs/specs/crosscut/dangerous_ops_confirmation.md` | `docs/specs/crosscut/dangerous_ops_taxonomy.md` as policy/rule SSOT; `docs/specs/crosscut/dangerous_ops_confirmation.md` as UX/API confirmation flow spec only | Taxonomy (what is dangerous and why) and confirmation (how operator confirms) should be separated but non-overlapping. Current boundary is easy to blur. | 1) Keep taxonomy doc free of UI workflow details. 2) Keep confirmation doc free of category/policy duplication; only reference taxonomy IDs. |
docs/audits/docs-ai-pack-scan.md:130:docs/audits/docs-os-existing-file-scan.md:67:docs/audits/docs-content-overlap.md:39:| OVL-04 | Execution specification layering | `docs/context/notes/execution.md`, `docs/context/notes/execution-gmo.md` | `docs/context/notes/execution.md` as provider-agnostic execution contract/behavior SSOT; `docs/context/notes/execution-gmo.md` as GMO adapter profile | Provider-specific constraints can leak into generic execution spec, causing multi-exchange evolution pain. | 1) Extract any GMO-only semantics out of generic spec into GMO profile. 2) Keep generic spec with neutral terms and extension hooks. |
docs/audits/docs-ai-pack-scan.md:131:docs/audits/docs-os-existing-file-scan.md:68:docs/audits/docs-content-overlap.md:40:| OVL-05 | Bots API/UX status model duplication | `docs/specs/controlplane-bots.md`, `docs/specs/ui-bots.md` | `docs/specs/controlplane-bots.md` as API schema/status semantics SSOT; `docs/specs/ui-bots.md` as UI behavior/rendering SSOT | Bot status fields (e.g., degraded reasons, states) are often repeated in both docs. API semantics should live in controlplane spec, UI doc should reference them. | 1) In UI spec, replace duplicated field definitions with links/brief mapping tables. 2) Keep authoritative enums/field contracts in controlplane spec. |
docs/audits/docs-ai-pack-scan.md:132:docs/audits/docs-os-existing-file-scan.md:69:docs/audits/docs-content-overlap.md:41:| OVL-06 | Paper E2E runbook duplication | `docs/runbooks/e2e-smoke-runbook.md`, `docs/runbooks/paper_e2e.md` | `docs/runbooks/e2e-smoke-runbook.md` as "one-command" operational runbook SSOT; `docs/runbooks/paper_e2e.md` as deep troubleshooting/reference | Both cover paper path checks; one should be quick path and the other should be expanded diagnostics. Distinguish by depth and audience. | 1) Ensure smoke runbook contains only happy-path + short triage. 2) Keep paper_e2e as detailed manual procedures and edge-case diagnostics. |
docs/audits/docs-ai-pack-scan.md:133:docs/audits/docs-os-existing-file-scan.md:70:docs/audits/docs-content-overlap.md:42:| OVL-07 | Governance baseline duplication | `docs/assumptions.md`, `docs/plans/roadmap.md`, `README.md` | `docs/assumptions.md` as baseline assumptions/constraints SSOT | Guardrails and defaults appear in multiple places; assumptions should be centralized and referenced elsewhere. | 1) Keep roadmap guardrails short and link to assumptions. 2) Keep README to brief security/baseline pointers only. |
docs/audits/docs-ai-pack-scan.md:134:docs/audits/docs-os-existing-file-scan.md:81:docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/docs-ai-pack-scan.md:135:docs/audits/docs-os-existing-file-scan.md:84:docs/assumptions.md:13:1. OpenAPI uses version `1.0.0` as the first SSOT baseline for V2.5+ bootstrap.
docs/audits/docs-ai-pack-scan.md:136:docs/audits/docs-os-existing-file-scan.md:92:docs/status/ultimate-gold-progress-check.md:54:| UG-00 | 全体NFR | P0 | In Progress | 55% | Contracts SSOT、idempotency、dead-man、監査ログ、health/capabilities基盤 | SAFE_MODE統一運用、SoT優先順位の明文化、replay決定性の全体保証 |
docs/audits/docs-ai-pack-scan.md:137:docs/audits/docs-os-existing-file-scan.md:93:docs/status/ultimate-gold-progress-check.md:89:  - Contracts SSOT
docs/audits/docs-ai-pack-scan.md:138:docs/audits/docs-os-existing-file-scan.md:94:docs/plans/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/audits/docs-ai-pack-scan.md:139:docs/audits/docs-os-existing-file-scan.md:95:docs/plans/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/audits/docs-ai-pack-scan.md:140:docs/audits/docs-os-existing-file-scan.md:96:docs/README.md:12:- Parallel task safety (SSOT for parallel development safety): [`specs/parallel-task-safety.md`](specs/parallel-task-safety.md)
docs/audits/docs-ai-pack-scan.md:141:docs/audits/docs-os-existing-file-scan.md:98:docs/workplan/ultimate-gold-implementation-feature-list.md:51:### UGF-0-023 変更管理SSOT（DecisionLog/Assumptions/ChangeLog運用）
docs/audits/docs-ai-pack-scan.md:143:docs/audits/docs-os-existing-file-scan.md:101:docs/workplan/ultimate-gold-implementation-feature-list.md:63:| Notes/Links | `0-7/0-9 変更管理SSOT` |
docs/audits/docs-ai-pack-scan.md:144:docs/audits/docs-os-existing-file-scan.md:105:## Command: find docs -maxdepth 4 -type f \( -iname "*readme*" -o -iname "*ssot*" -o -iname "*handoff*" -o -iname "*decision*" -o -iname "*status*.json" -o -iname "*trace*" -o -iname "*locks*" -o -iname "*glossary*" \) -print
docs/audits/docs-ai-pack-scan.md:145:docs/audits/docs-audit-report.md:38:- `docs/specs/crosscut/parallel_task_safety_spec.md` → `rules/spec (parallel safety SSOT)`
docs/audits/docs-ai-pack-scan.md:146:docs/audits/docs-audit-report.md:72:### 判定: **部分的に固定（入口はあるが単一SSOTとしては未固定）**
docs/audits/docs-ai-pack-scan.md:147:docs/audits/docs-audit-report.md:83:### B-2. Epic / MRU / Contracts-SSOT / 運用原則の接続性
docs/audits/docs-ai-pack-scan.md:148:docs/audits/docs-audit-report.md:90:  - `roadmap` に Contracts SSOT guardrail がある。
docs/audits/docs-ai-pack-scan.md:149:docs/audits/docs-audit-report.md:93:  - 「Contractsが絶対SSOT」であることは docs内に分散記述され、統治入口（governance hub）が未固定。
docs/audits/docs-ai-pack-scan.md:150:docs/audits/docs-audit-report.md:100:2. **Decision Log の明示的SSOT入口**
docs/audits/docs-ai-pack-scan.md:151:docs/audits/docs-audit-report.md:104:4. **handoff運用の明示文書**
docs/audits/docs-ai-pack-scan.md:152:docs/audits/docs-audit-report.md:128:  - **監査レポート:** `docs/audits/*`（非SSOT）
docs/audits/docs-ai-pack-scan.md:153:docs/audits/docs-audit-report.md:161:  - **運用原則SSOT:** `docs/specs/crosscut/parallel_task_safety_spec.md`
docs/audits/docs-ai-pack-scan.md:154:docs/audits/docs-audit-report.md:194:  - **差分監査:** `ui-current-vs-spec.md`（非SSOT）
docs/audits/docs-ai-pack-scan.md:155:docs/audits/docs-audit-report.md:269:rg -i "status|progress|workplan|roadmap|spec|design|decision|handoff|mru" -n docs
docs/audits/docs-ai-pack-scan.md:156:docs/decisions/records/docs-rules-unify-decision.md:5:- Chosen approach: **Option A** (`docs/rules/` as rules SSOT namespace).
docs/audits/docs-ai-pack-scan.md:157:docs/decisions/records/docs-rules-unify-decision.md:6:- Reason: existing rule content is concentrated in `docs/specs/crosscut/parallel_task_safety_spec.md`; new required operational topics (Multi-AI / Credit-out / HANDOFF / status / trace / decisions / LOCK policy) should be explicit and discoverable under a dedicated operations-rule path.
docs/audits/docs-ai-pack-scan.md:158:docs/decisions/records/docs-rules-unify-decision.md:14:| Multi-AI / Credit-out / Handoff | `docs/specs/crosscut/parallel_task_safety_spec.md` (partial task-generation section only) | `docs/rules/task-generation-policy.md` | merge + extend | Add stop protocol + mandatory HANDOFF update semantics. |
docs/audits/docs-ai-pack-scan.md:159:docs/decisions/records/docs-rules-unify-decision.md:16:| Decision log policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Standardize decision fixation via `docs/status/decisions.md`. |
docs/audits/docs-ai-pack-scan.md:160:docs/decisions/records/docs-rules-unify-decision.md:17:| Traceability policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Set `docs/status/trace-index.md` as trace SSOT, avoid scattered links. |
docs/audits/docs-ai-pack-scan.md:161:docs/decisions/records/docs-rules-unify-decision.md:22:- `docs/context/README_AI.md` is created/updated as must-read entrypoint for AI operators.
docs/audits/docs-ai-pack-scan.md:162:docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/docs-ai-pack-scan.md:163:docs/decisions/records/docs-os-consolidation-decision.md:5:| SSOT entry for Docs Development OS | `docs/README.md` (general docs index), `docs/specs/crosscut/parallel_task_safety_spec.md` (parallel safety SSOT note), `docs/audits/docs-content-overlap.md` (canonicalization guidance) | `docs/context/README_AI.md` | **Merge** | Existing docs already describe partial SSOT/canonical concepts, but no single AI-first entrypoint exists. Create one canonical entrance and link all SSOT roles there. |
docs/audits/docs-ai-pack-scan.md:164:docs/decisions/records/docs-os-consolidation-decision.md:6:| Machine-readable current status | No direct candidate found in `docs/**` (`status.json` not present) | `docs/status/status.json` | **Keep (new canonical)** | Add a dedicated structured status file for active epic/task, locks, next actions, and PR state. |
docs/audits/docs-ai-pack-scan.md:165:docs/decisions/records/docs-os-consolidation-decision.md:7:| Handoff protocol | No direct candidate found in `docs/**` (`HANDOFF.json` not present) | `docs/handoff/HANDOFF.json` | **Keep (new canonical)** | Add explicit stop/credit-out handoff file with required fields for multi-AI continuity. |
docs/audits/docs-ai-pack-scan.md:167:docs/decisions/records/docs-os-consolidation-decision.md:9:| Traceability index | No direct candidate found in `docs/**` (`trace-index.json` absent) | `docs/status/trace-index.json` | **Keep (new canonical)** | Introduce one machine-readable trace SSOT for req↔progress↔PR/commit/spec/runbook/verification links. |
docs/audits/docs-ai-pack-scan.md:168:docs/decisions/records/docs-os-consolidation-decision.md:12:- No existing same-role canonical files were found for `status.json`, `HANDOFF.json`, `decisions.md`, or `trace-index.json`, so no stub replacement was required.
docs/audits/docs-ai-pack-scan.md:169:docs/decisions/records/docs-os-consolidation-decision.md:13:- `docs/README.md` remains a general docs hub; `docs/context/README_AI.md` becomes the canonical AI operating entrypoint and references role-specific SSOT paths.
docs/audits/docs-ai-pack-scan.md:170:docs/audits/docs-content-overlap.md:34:| Area ID | Suspected overlap area | Current files involved | Proposed canonical (SSOT) | Rationale | Follow-up touch plan (next PRs) |
docs/audits/docs-ai-pack-scan.md:171:docs/audits/docs-content-overlap.md:37:| OVL-02 | Ultimate Gold planning/progress duplication | `docs/workplan/ultimate-gold-implementation-feature-list.md`, `docs/status/ultimate-gold-progress-check.md`, `docs/status/progress-updates/UG-P0-*.md` | `docs/workplan/ultimate-gold-implementation-feature-list.md` = requirements/catalog SSOT; `docs/status/ultimate-gold-progress-check.md` = current status dashboard SSOT; `docs/status/progress-updates/*.md` = append-only evidence logs | The workplan and progress-check both contain requirement mapping + milestone status narrative; risk of drift is high. Split roles explicitly: "what should exist" vs "what currently exists" vs "event/evidence log". | 1) Remove requirement restatement from progress-check; keep live status only. 2) In progress updates, enforce short template + links to PR/commit only. 3) Add explicit cross-links among the 3 layers. |
docs/audits/docs-ai-pack-scan.md:172:docs/audits/docs-content-overlap.md:38:| OVL-03 | Dangerous operation rule definition split | `docs/specs/crosscut/dangerous_ops_taxonomy.md`, `docs/specs/crosscut/dangerous_ops_confirmation.md` | `docs/specs/crosscut/dangerous_ops_taxonomy.md` as policy/rule SSOT; `docs/specs/crosscut/dangerous_ops_confirmation.md` as UX/API confirmation flow spec only | Taxonomy (what is dangerous and why) and confirmation (how operator confirms) should be separated but non-overlapping. Current boundary is easy to blur. | 1) Keep taxonomy doc free of UI workflow details. 2) Keep confirmation doc free of category/policy duplication; only reference taxonomy IDs. |
docs/audits/docs-ai-pack-scan.md:173:docs/audits/docs-content-overlap.md:39:| OVL-04 | Execution specification layering | `docs/context/notes/execution.md`, `docs/context/notes/execution-gmo.md` | `docs/context/notes/execution.md` as provider-agnostic execution contract/behavior SSOT; `docs/context/notes/execution-gmo.md` as GMO adapter profile | Provider-specific constraints can leak into generic execution spec, causing multi-exchange evolution pain. | 1) Extract any GMO-only semantics out of generic spec into GMO profile. 2) Keep generic spec with neutral terms and extension hooks. |
docs/audits/docs-ai-pack-scan.md:174:docs/audits/docs-content-overlap.md:40:| OVL-05 | Bots API/UX status model duplication | `docs/specs/controlplane-bots.md`, `docs/specs/ui-bots.md` | `docs/specs/controlplane-bots.md` as API schema/status semantics SSOT; `docs/specs/ui-bots.md` as UI behavior/rendering SSOT | Bot status fields (e.g., degraded reasons, states) are often repeated in both docs. API semantics should live in controlplane spec, UI doc should reference them. | 1) In UI spec, replace duplicated field definitions with links/brief mapping tables. 2) Keep authoritative enums/field contracts in controlplane spec. |
docs/audits/docs-ai-pack-scan.md:175:docs/audits/docs-content-overlap.md:41:| OVL-06 | Paper E2E runbook duplication | `docs/runbooks/e2e-smoke-runbook.md`, `docs/runbooks/paper_e2e.md` | `docs/runbooks/e2e-smoke-runbook.md` as "one-command" operational runbook SSOT; `docs/runbooks/paper_e2e.md` as deep troubleshooting/reference | Both cover paper path checks; one should be quick path and the other should be expanded diagnostics. Distinguish by depth and audience. | 1) Ensure smoke runbook contains only happy-path + short triage. 2) Keep paper_e2e as detailed manual procedures and edge-case diagnostics. |
docs/audits/docs-ai-pack-scan.md:176:docs/audits/docs-content-overlap.md:42:| OVL-07 | Governance baseline duplication | `docs/assumptions.md`, `docs/plans/roadmap.md`, `README.md` | `docs/assumptions.md` as baseline assumptions/constraints SSOT | Guardrails and defaults appear in multiple places; assumptions should be centralized and referenced elsewhere. | 1) Keep roadmap guardrails short and link to assumptions. 2) Keep README to brief security/baseline pointers only. |
docs/audits/docs-ai-pack-scan.md:177:docs/audits/docs-rules-unify-scan.md:52:## 3) `rg -n "parallel-task-safety|タスク生成|task generation|Codexアジャイル|致命的事故|guardrail|LOCK:|handoff|HANDOFF|status\.json|trace-index|decisions|SSOT|README_AI" docs -S`
docs/audits/docs-ai-pack-scan.md:178:docs/audits/docs-rules-unify-scan.md:55:docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/audits/docs-ai-pack-scan.md:179:docs/audits/docs-rules-unify-scan.md:56:docs/changelog.md:220:- Updated README with contract SSOT and validation commands.
docs/audits/docs-ai-pack-scan.md:180:docs/audits/docs-rules-unify-scan.md:57:docs/workplan/ultimate-gold-implementation-feature-list.md:51:### UGF-0-023 変更管理SSOT（DecisionLog/Assumptions/ChangeLog運用）
docs/audits/docs-ai-pack-scan.md:182:docs/audits/docs-rules-unify-scan.md:59:docs/workplan/ultimate-gold-implementation-feature-list.md:63:| Notes/Links | `0-7/0-9 変更管理SSOT` |
docs/audits/docs-ai-pack-scan.md:183:docs/audits/docs-rules-unify-scan.md:62:docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/docs-ai-pack-scan.md:184:docs/audits/docs-rules-unify-scan.md:63:docs/audits/docs-content-overlap.md:34:| Area ID | Suspected overlap area | Current files involved | Proposed canonical (SSOT) | Rationale | Follow-up touch plan (next PRs) |
docs/audits/docs-ai-pack-scan.md:185:docs/audits/docs-rules-unify-scan.md:64:docs/audits/docs-content-overlap.md:37:| OVL-02 | Ultimate Gold planning/progress duplication | `docs/workplan/ultimate-gold-implementation-feature-list.md`, `docs/status/ultimate-gold-progress-check.md`, `docs/status/progress-updates/UG-P0-*.md` | `docs/workplan/ultimate-gold-implementation-feature-list.md` = requirements/catalog SSOT; `docs/status/ultimate-gold-progress-check.md` = current status dashboard SSOT; `docs/status/progress-updates/*.md` = append-only evidence logs | The workplan and progress-check both contain requirement mapping + milestone status narrative; risk of drift is high. Split roles explicitly: "what should exist" vs "what currently exists" vs "event/evidence log". | 1) Remove requirement restatement from progress-check; keep live status only. 2) In progress updates, enforce short template + links to PR/commit only. 3) Add explicit cross-links among the 3 layers. |
docs/audits/docs-ai-pack-scan.md:186:docs/audits/docs-rules-unify-scan.md:65:docs/audits/docs-content-overlap.md:38:| OVL-03 | Dangerous operation rule definition split | `docs/specs/crosscut/dangerous_ops_taxonomy.md`, `docs/specs/crosscut/dangerous_ops_confirmation.md` | `docs/specs/crosscut/dangerous_ops_taxonomy.md` as policy/rule SSOT; `docs/specs/crosscut/dangerous_ops_confirmation.md` as UX/API confirmation flow spec only | Taxonomy (what is dangerous and why) and confirmation (how operator confirms) should be separated but non-overlapping. Current boundary is easy to blur. | 1) Keep taxonomy doc free of UI workflow details. 2) Keep confirmation doc free of category/policy duplication; only reference taxonomy IDs. |
docs/audits/docs-ai-pack-scan.md:187:docs/audits/docs-rules-unify-scan.md:66:docs/audits/docs-content-overlap.md:39:| OVL-04 | Execution specification layering | `docs/context/notes/execution.md`, `docs/context/notes/execution-gmo.md` | `docs/context/notes/execution.md` as provider-agnostic execution contract/behavior SSOT; `docs/context/notes/execution-gmo.md` as GMO adapter profile | Provider-specific constraints can leak into generic execution spec, causing multi-exchange evolution pain. | 1) Extract any GMO-only semantics out of generic spec into GMO profile. 2) Keep generic spec with neutral terms and extension hooks. |
docs/audits/docs-ai-pack-scan.md:188:docs/audits/docs-rules-unify-scan.md:67:docs/audits/docs-content-overlap.md:40:| OVL-05 | Bots API/UX status model duplication | `docs/specs/controlplane-bots.md`, `docs/specs/ui-bots.md` | `docs/specs/controlplane-bots.md` as API schema/status semantics SSOT; `docs/specs/ui-bots.md` as UI behavior/rendering SSOT | Bot status fields (e.g., degraded reasons, states) are often repeated in both docs. API semantics should live in controlplane spec, UI doc should reference them. | 1) In UI spec, replace duplicated field definitions with links/brief mapping tables. 2) Keep authoritative enums/field contracts in controlplane spec. |
docs/audits/docs-ai-pack-scan.md:189:docs/audits/docs-rules-unify-scan.md:68:docs/audits/docs-content-overlap.md:41:| OVL-06 | Paper E2E runbook duplication | `docs/runbooks/e2e-smoke-runbook.md`, `docs/runbooks/paper_e2e.md` | `docs/runbooks/e2e-smoke-runbook.md` as "one-command" operational runbook SSOT; `docs/runbooks/paper_e2e.md` as deep troubleshooting/reference | Both cover paper path checks; one should be quick path and the other should be expanded diagnostics. Distinguish by depth and audience. | 1) Ensure smoke runbook contains only happy-path + short triage. 2) Keep paper_e2e as detailed manual procedures and edge-case diagnostics. |
docs/audits/docs-ai-pack-scan.md:190:docs/audits/docs-rules-unify-scan.md:69:docs/audits/docs-content-overlap.md:42:| OVL-07 | Governance baseline duplication | `docs/assumptions.md`, `docs/plans/roadmap.md`, `README.md` | `docs/assumptions.md` as baseline assumptions/constraints SSOT | Guardrails and defaults appear in multiple places; assumptions should be centralized and referenced elsewhere. | 1) Keep roadmap guardrails short and link to assumptions. 2) Keep README to brief security/baseline pointers only. |
docs/audits/docs-ai-pack-scan.md:191:docs/audits/docs-rules-unify-scan.md:70:docs/status/ultimate-gold-progress-check.md:54:| UG-00 | 全体NFR | P0 | In Progress | 55% | Contracts SSOT、idempotency、dead-man、監査ログ、health/capabilities基盤 | SAFE_MODE統一運用、SoT優先順位の明文化、replay決定性の全体保証 |
docs/audits/docs-ai-pack-scan.md:192:docs/audits/docs-rules-unify-scan.md:71:docs/status/ultimate-gold-progress-check.md:89:  - Contracts SSOT
docs/audits/docs-ai-pack-scan.md:193:docs/audits/docs-rules-unify-scan.md:72:docs/plans/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/audits/docs-ai-pack-scan.md:194:docs/audits/docs-rules-unify-scan.md:73:docs/plans/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/audits/docs-ai-pack-scan.md:195:docs/audits/docs-rules-unify-scan.md:74:docs/assumptions.md:13:1. OpenAPI uses version `1.0.0` as the first SSOT baseline for V2.5+ bootstrap.
docs/audits/docs-ai-pack-scan.md:196:docs/audits/docs-rules-unify-scan.md:75:docs/README.md:12:- Parallel task safety (SSOT for parallel development safety): [`specs/parallel-task-safety.md`](specs/parallel-task-safety.md)
docs/audits/docs-ai-pack-scan.md:197:docs/audits/docs-rules-unify-scan.md:76:docs/specs/crosscut/parallel_task_safety_spec.md:1:# Parallel Task Safety Spec (SSOT)
docs/audits/docs-ai-pack-scan.md:198:docs/audits/docs-rules-unify-scan.md:87:  -iname "*handoff*" -o -iname "*decision*" -o -iname "*status*.json" -o -iname "*trace*" \
docs/audits/docs-ai-pack-scan.md:203:docs/context/README_AI.md
docs/audits/docs-ai-pack-scan.md:204:docs/status/status.json
docs/audits/docs-rules-unify-scan.md:52:## 3) `rg -n "parallel-task-safety|タスク生成|task generation|Codexアジャイル|致命的事故|guardrail|LOCK:|handoff|HANDOFF|status\.json|trace-index|decisions|SSOT|README_AI" docs -S`
docs/audits/docs-rules-unify-scan.md:55:docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/audits/docs-rules-unify-scan.md:56:docs/changelog.md:220:- Updated README with contract SSOT and validation commands.
docs/audits/docs-rules-unify-scan.md:57:docs/workplan/ultimate-gold-implementation-feature-list.md:51:### UGF-0-023 変更管理SSOT（DecisionLog/Assumptions/ChangeLog運用）
docs/audits/docs-rules-unify-scan.md:59:docs/workplan/ultimate-gold-implementation-feature-list.md:63:| Notes/Links | `0-7/0-9 変更管理SSOT` |
docs/audits/docs-rules-unify-scan.md:62:docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/docs-rules-unify-scan.md:63:docs/audits/docs-content-overlap.md:34:| Area ID | Suspected overlap area | Current files involved | Proposed canonical (SSOT) | Rationale | Follow-up touch plan (next PRs) |
docs/audits/docs-rules-unify-scan.md:64:docs/audits/docs-content-overlap.md:37:| OVL-02 | Ultimate Gold planning/progress duplication | `docs/workplan/ultimate-gold-implementation-feature-list.md`, `docs/status/ultimate-gold-progress-check.md`, `docs/status/progress-updates/UG-P0-*.md` | `docs/workplan/ultimate-gold-implementation-feature-list.md` = requirements/catalog SSOT; `docs/status/ultimate-gold-progress-check.md` = current status dashboard SSOT; `docs/status/progress-updates/*.md` = append-only evidence logs | The workplan and progress-check both contain requirement mapping + milestone status narrative; risk of drift is high. Split roles explicitly: "what should exist" vs "what currently exists" vs "event/evidence log". | 1) Remove requirement restatement from progress-check; keep live status only. 2) In progress updates, enforce short template + links to PR/commit only. 3) Add explicit cross-links among the 3 layers. |
docs/audits/docs-rules-unify-scan.md:65:docs/audits/docs-content-overlap.md:38:| OVL-03 | Dangerous operation rule definition split | `docs/specs/crosscut/dangerous_ops_taxonomy.md`, `docs/specs/crosscut/dangerous_ops_confirmation.md` | `docs/specs/crosscut/dangerous_ops_taxonomy.md` as policy/rule SSOT; `docs/specs/crosscut/dangerous_ops_confirmation.md` as UX/API confirmation flow spec only | Taxonomy (what is dangerous and why) and confirmation (how operator confirms) should be separated but non-overlapping. Current boundary is easy to blur. | 1) Keep taxonomy doc free of UI workflow details. 2) Keep confirmation doc free of category/policy duplication; only reference taxonomy IDs. |
docs/audits/docs-rules-unify-scan.md:66:docs/audits/docs-content-overlap.md:39:| OVL-04 | Execution specification layering | `docs/context/notes/execution.md`, `docs/context/notes/execution-gmo.md` | `docs/context/notes/execution.md` as provider-agnostic execution contract/behavior SSOT; `docs/context/notes/execution-gmo.md` as GMO adapter profile | Provider-specific constraints can leak into generic execution spec, causing multi-exchange evolution pain. | 1) Extract any GMO-only semantics out of generic spec into GMO profile. 2) Keep generic spec with neutral terms and extension hooks. |
docs/audits/docs-rules-unify-scan.md:67:docs/audits/docs-content-overlap.md:40:| OVL-05 | Bots API/UX status model duplication | `docs/specs/controlplane-bots.md`, `docs/specs/ui-bots.md` | `docs/specs/controlplane-bots.md` as API schema/status semantics SSOT; `docs/specs/ui-bots.md` as UI behavior/rendering SSOT | Bot status fields (e.g., degraded reasons, states) are often repeated in both docs. API semantics should live in controlplane spec, UI doc should reference them. | 1) In UI spec, replace duplicated field definitions with links/brief mapping tables. 2) Keep authoritative enums/field contracts in controlplane spec. |
docs/audits/docs-rules-unify-scan.md:68:docs/audits/docs-content-overlap.md:41:| OVL-06 | Paper E2E runbook duplication | `docs/runbooks/e2e-smoke-runbook.md`, `docs/runbooks/paper_e2e.md` | `docs/runbooks/e2e-smoke-runbook.md` as "one-command" operational runbook SSOT; `docs/runbooks/paper_e2e.md` as deep troubleshooting/reference | Both cover paper path checks; one should be quick path and the other should be expanded diagnostics. Distinguish by depth and audience. | 1) Ensure smoke runbook contains only happy-path + short triage. 2) Keep paper_e2e as detailed manual procedures and edge-case diagnostics. |
docs/audits/docs-rules-unify-scan.md:69:docs/audits/docs-content-overlap.md:42:| OVL-07 | Governance baseline duplication | `docs/assumptions.md`, `docs/plans/roadmap.md`, `README.md` | `docs/assumptions.md` as baseline assumptions/constraints SSOT | Guardrails and defaults appear in multiple places; assumptions should be centralized and referenced elsewhere. | 1) Keep roadmap guardrails short and link to assumptions. 2) Keep README to brief security/baseline pointers only. |
docs/audits/docs-rules-unify-scan.md:70:docs/status/ultimate-gold-progress-check.md:54:| UG-00 | 全体NFR | P0 | In Progress | 55% | Contracts SSOT、idempotency、dead-man、監査ログ、health/capabilities基盤 | SAFE_MODE統一運用、SoT優先順位の明文化、replay決定性の全体保証 |
docs/audits/docs-rules-unify-scan.md:71:docs/status/ultimate-gold-progress-check.md:89:  - Contracts SSOT
docs/audits/docs-rules-unify-scan.md:72:docs/plans/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/audits/docs-rules-unify-scan.md:73:docs/plans/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/audits/docs-rules-unify-scan.md:74:docs/assumptions.md:13:1. OpenAPI uses version `1.0.0` as the first SSOT baseline for V2.5+ bootstrap.
docs/audits/docs-rules-unify-scan.md:75:docs/README.md:12:- Parallel task safety (SSOT for parallel development safety): [`specs/parallel-task-safety.md`](specs/parallel-task-safety.md)
docs/audits/docs-rules-unify-scan.md:76:docs/specs/crosscut/parallel_task_safety_spec.md:1:# Parallel Task Safety Spec (SSOT)
docs/audits/docs-rules-unify-scan.md:87:  -iname "*handoff*" -o -iname "*decision*" -o -iname "*status*.json" -o -iname "*trace*" \
```
