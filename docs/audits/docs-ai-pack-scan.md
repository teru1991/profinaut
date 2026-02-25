# DOC-AI-PACK Scan

## Command: git status --short
```
?? docs/audits/docs-ai-pack-scan.md
?? services/marketdata-rs/Cargo.lock
```

## Command: git ls-tree -r --name-only HEAD docs
```
docs/README.md
docs/context/README_AI.md
docs/assumptions.md
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

## Command: rg -n "CURRENT_STATUS|AI_READ_ME|TECH_CONTEXT|README_AI|SSOT|status\.json|trace-index|handoff|HANDOFF|decisions|LOCKS|glossary|context pack" docs -S
```
docs/decisions/decisions.md:6:- 2026-02-17: contracts additive-only (see contracts policy SSOT where applicable) / Prevents breaking downstream consumers / Contract evolution must preserve compatibility.
docs/decisions/decisions.md:7:- 2026-02-17: docs canonical入口 is `docs/context/README_AI.md` / Single entrypoint prevents SSOT ambiguity / AI and human operators start from one canonical path.
docs/decisions/decisions.md:8:- 2026-02-17: stopping requires `docs/handoff/HANDOFF.json` update / Multi-agent and interrupted work need explicit continuity state / No stop/handoff without handoff state refresh.
docs/workplan/ultimate-gold-implementation-feature-list.md:51:### UGF-0-023 変更管理SSOT（DecisionLog/Assumptions/ChangeLog運用）
docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/workplan/ultimate-gold-implementation-feature-list.md:63:| Notes/Links | `0-7/0-9 変更管理SSOT` |
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
docs/status/trace-index.json:19:        "docs/decisions/decisions.md"
docs/status/ultimate-gold-progress-check.md:54:| UG-00 | 全体NFR | P0 | In Progress | 55% | Contracts SSOT、idempotency、dead-man、監査ログ、health/capabilities基盤 | SAFE_MODE統一運用、SoT優先順位の明文化、replay決定性の全体保証 |
docs/status/ultimate-gold-progress-check.md:89:  - Contracts SSOT
docs/status/decisions.md:1:# Decisions Log (SSOT)
docs/context/README_AI.md:1:# README_AI (SSOT entrypoint for operators)
docs/context/README_AI.md:7:1. This file: `docs/context/README_AI.md`
docs/context/README_AI.md:8:2. Runtime status: `docs/status/status.json`
docs/context/README_AI.md:9:3. Handoff state: `docs/status/HANDOFF.json`
docs/context/README_AI.md:10:4. Decision baseline: `docs/status/decisions.md`
docs/context/README_AI.md:11:5. Rules SSOT:
docs/context/README_AI.md:17:- **Stop protocol (Multi-AI / Credit-out):** if stopping, pausing, or handing over, update `docs/status/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
docs/context/README_AI.md:18:- **Single active task:** only one active task is tracked in `docs/status/status.json`.
docs/context/README_AI.md:19:- **Progress claim evidence:** update `docs/status/status.json` or append `docs/status/progress-updates/*` before claiming progress.
docs/context/README_AI.md:20:- **LOCK areas:** lock-sensitive areas include contracts/ci/infra/lockfiles and shared-docs (`docs/context/**`, `docs/rules/**`, docs hubs). Declare `Required Locks` in task cards.
docs/context/README_AI.md:21:- **Trace SSOT:** `docs/status/trace-index.md` is the canonical place for PR/commit/evidence links.
docs/context/README_AI.md:22:- **Decisions are binding:** use `docs/status/decisions.md` to fix assumptions and supersede only via explicit new decision entries.
docs/rules/task-generation-policy.md:1:# Task Generation Policy (SSOT)
docs/rules/task-generation-policy.md:9:1. `docs/context/README_AI.md`
docs/rules/task-generation-policy.md:10:2. `docs/status/status.json`
docs/rules/task-generation-policy.md:11:3. `docs/status/HANDOFF.json`
docs/rules/task-generation-policy.md:12:4. `docs/status/decisions.md`
docs/rules/task-generation-policy.md:19:- Active task ownership and lifecycle are tracked in `docs/status/status.json`.
docs/rules/task-generation-policy.md:45:- Canonical runtime status must be updated through `docs/status/status.json` only.
docs/rules/task-generation-policy.md:50:When an operator must stop early (time, token, context, incident, rate limits), that stop is treated as **Credit-out** and requires a handoff update before exiting.
docs/rules/task-generation-policy.md:52:### 4.3 Mandatory HANDOFF update on stop/switch
docs/rules/task-generation-policy.md:54:Before stopping or switching owner, update `docs/status/HANDOFF.json` with all of:
docs/rules/task-generation-policy.md:66:  - an update to `docs/status/status.json`, **or**
docs/rules/task-generation-policy.md:72:- Use `docs/status/decisions.md` to lock assumptions and decisions that affect ongoing work.
docs/rules/task-generation-policy.md:78:- `docs/status/trace-index.md` is the SSOT for trace links (PRs, commits, issues, run artifacts).
docs/rules/task-generation-policy.md:80:- Other docs may include convenience links, but must treat trace-index as canonical.
docs/specs/crosscut/safety_interlock_spec.md:1:# Parallel Development Safety (SSOT)
docs/specs/crosscut/safety_interlock_spec.md:43:- shared-docs (`docs/context/**`, `docs/rules/**`, docs hubs/indexes)
docs/specs/crosscut/safety_interlock_spec.md:49:- `Required Locks: docs/context/**`
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
docs/audits/docs-os-existing-file-scan.md:92:docs/status/ultimate-gold-progress-check.md:54:| UG-00 | 全体NFR | P0 | In Progress | 55% | Contracts SSOT、idempotency、dead-man、監査ログ、health/capabilities基盤 | SAFE_MODE統一運用、SoT優先順位の明文化、replay決定性の全体保証 |
docs/audits/docs-os-existing-file-scan.md:93:docs/status/ultimate-gold-progress-check.md:89:  - Contracts SSOT
docs/audits/docs-os-existing-file-scan.md:94:docs/plans/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/audits/docs-os-existing-file-scan.md:95:docs/plans/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/audits/docs-os-existing-file-scan.md:96:docs/README.md:12:- Parallel task safety (SSOT for parallel development safety): [`specs/parallel-task-safety.md`](specs/parallel-task-safety.md)
docs/audits/docs-os-existing-file-scan.md:98:docs/workplan/ultimate-gold-implementation-feature-list.md:51:### UGF-0-023 変更管理SSOT（DecisionLog/Assumptions/ChangeLog運用）
docs/audits/docs-os-existing-file-scan.md:100:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
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
docs/decisions/records/docs-rules-unify-decision.md:16:| Decision log policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Standardize decision fixation via `docs/status/decisions.md`. |
docs/decisions/records/docs-rules-unify-decision.md:17:| Traceability policy | no dedicated active SSOT in scan result | `docs/rules/task-generation-policy.md` | add new canonical | Set `docs/status/trace-index.md` as trace SSOT, avoid scattered links. |
docs/decisions/records/docs-rules-unify-decision.md:22:- `docs/context/README_AI.md` is created/updated as must-read entrypoint for AI operators.
docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/decisions/records/docs-os-consolidation-decision.md:5:| SSOT entry for Docs Development OS | `docs/README.md` (general docs index), `docs/specs/crosscut/parallel_task_safety_spec.md` (parallel safety SSOT note), `docs/audits/docs-content-overlap.md` (canonicalization guidance) | `docs/context/README_AI.md` | **Merge** | Existing docs already describe partial SSOT/canonical concepts, but no single AI-first entrypoint exists. Create one canonical entrance and link all SSOT roles there. |
docs/decisions/records/docs-os-consolidation-decision.md:6:| Machine-readable current status | No direct candidate found in `docs/**` (`status.json` not present) | `docs/status/status.json` | **Keep (new canonical)** | Add a dedicated structured status file for active epic/task, locks, next actions, and PR state. |
docs/decisions/records/docs-os-consolidation-decision.md:7:| Handoff protocol | No direct candidate found in `docs/**` (`HANDOFF.json` not present) | `docs/handoff/HANDOFF.json` | **Keep (new canonical)** | Add explicit stop/credit-out handoff file with required fields for multi-AI continuity. |
docs/decisions/records/docs-os-consolidation-decision.md:8:| Decision log | No direct candidate found in `docs/**` (`docs/decisions/*.md` absent). Related mentions in `docs/changelog.md` and workplan notes. | `docs/decisions/decisions.md` | **Merge** | Keep changelog/workplan for their original purpose, and establish one decision log canonical path for short decision entries. |
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
docs/audits/docs-rules-unify-scan.md:52:## 3) `rg -n "parallel-task-safety|タスク生成|task generation|Codexアジャイル|致命的事故|guardrail|LOCK:|handoff|HANDOFF|status\.json|trace-index|decisions|SSOT|README_AI" docs -S`
docs/audits/docs-rules-unify-scan.md:55:docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/audits/docs-rules-unify-scan.md:56:docs/changelog.md:220:- Updated README with contract SSOT and validation commands.
docs/audits/docs-rules-unify-scan.md:57:docs/workplan/ultimate-gold-implementation-feature-list.md:51:### UGF-0-023 変更管理SSOT（DecisionLog/Assumptions/ChangeLog運用）
docs/audits/docs-rules-unify-scan.md:58:docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
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

## Command: find docs -maxdepth 5 -type f \( -iname "*current*status*" -o -iname "*tech*context*" -o -iname "*ai*read*" -o -iname "*readme*ai*" -o -iname "*status*.json" \) -print
```
docs/context/README_AI.md
docs/status/status.json
```
