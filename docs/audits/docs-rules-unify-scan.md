# DOC-RULES-UNIFY-001 Scan Log

Date: 2026-02-17 (UTC)

## 1) `git status --short`

```bash
?? services/marketdata-rs/Cargo.lock
```

## 2) `git ls-tree -r --name-only HEAD docs`

```bash
docs/README.md
docs/assumptions.md
docs/audits/docs-content-overlap.md
docs/audits/repo-progress-audit-2026-02-14.md
docs/audits/ui-current-vs-spec.md
docs/changelog.md
docs/roadmap.md
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
docs/status/progress-updates/UG-P0-101.md
docs/status/progress-updates/UG-P0-102.md
docs/status/progress-updates/UG-P0-103.md
docs/status/progress-updates/UG-P0-104.md
docs/status/progress-updates/UG-P0-105.md
docs/status/progress-updates/UG-P0-106.md
docs/status/progress-updates/UG-P0-110.md
docs/status/progress-updates/UG-P0-111.md
docs/status/progress-updates/UG-P0-112.md
docs/status/ultimate-gold-progress-check.md
docs/troubleshooting/bots-502.md
docs/verification/marketdata-data-platform-smoke-results.md
docs/verification/marketdata-data-platform-smoke.md
docs/workplan/ultimate-gold-implementation-feature-list.md
```

## 3) `rg -n "parallel-task-safety|タスク生成|task generation|Codexアジャイル|致命的事故|guardrail|LOCK:|handoff|HANDOFF|status\.json|trace-index|decisions|SSOT|README_AI" docs -S`

```bash
docs/changelog.md:205:## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
docs/changelog.md:220:- Updated README with contract SSOT and validation commands.
docs/workplan/ultimate-gold-implementation-feature-list.md:51:### UGF-0-023 変更管理SSOT（DecisionLog/Assumptions/ChangeLog運用）
docs/workplan/ultimate-gold-implementation-feature-list.md:60:| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
docs/workplan/ultimate-gold-implementation-feature-list.md:63:| Notes/Links | `0-7/0-9 変更管理SSOT` |
docs/workplan/ultimate-gold-implementation-feature-list.md:83:| Scope | `supply-chain-guardrails-ci` |
docs/runbooks/supply-chain-security.md:7:On every pull request (and manual dispatch), the workflow runs three guardrails:
docs/audits/repo-progress-audit-2026-02-14.md:65:  - Contracts are SSOT and validated in CI.
docs/audits/docs-content-overlap.md:34:| Area ID | Suspected overlap area | Current files involved | Proposed canonical (SSOT) | Rationale | Follow-up touch plan (next PRs) |
docs/audits/docs-content-overlap.md:37:| OVL-02 | Ultimate Gold planning/progress duplication | `docs/workplan/ultimate-gold-implementation-feature-list.md`, `docs/status/ultimate-gold-progress-check.md`, `docs/status/progress-updates/UG-P0-*.md` | `docs/workplan/ultimate-gold-implementation-feature-list.md` = requirements/catalog SSOT; `docs/status/ultimate-gold-progress-check.md` = current status dashboard SSOT; `docs/status/progress-updates/*.md` = append-only evidence logs | The workplan and progress-check both contain requirement mapping + milestone status narrative; risk of drift is high. Split roles explicitly: "what should exist" vs "what currently exists" vs "event/evidence log". | 1) Remove requirement restatement from progress-check; keep live status only. 2) In progress updates, enforce short template + links to PR/commit only. 3) Add explicit cross-links among the 3 layers. |
docs/audits/docs-content-overlap.md:38:| OVL-03 | Dangerous operation rule definition split | `docs/specs/crosscut/dangerous_ops_taxonomy.md`, `docs/specs/crosscut/dangerous_ops_confirmation.md` | `docs/specs/crosscut/dangerous_ops_taxonomy.md` as policy/rule SSOT; `docs/specs/crosscut/dangerous_ops_confirmation.md` as UX/API confirmation flow spec only | Taxonomy (what is dangerous and why) and confirmation (how operator confirms) should be separated but non-overlapping. Current boundary is easy to blur. | 1) Keep taxonomy doc free of UI workflow details. 2) Keep confirmation doc free of category/policy duplication; only reference taxonomy IDs. |
docs/audits/docs-content-overlap.md:39:| OVL-04 | Execution specification layering | `docs/context/notes/execution.md`, `docs/context/notes/execution-gmo.md` | `docs/context/notes/execution.md` as provider-agnostic execution contract/behavior SSOT; `docs/context/notes/execution-gmo.md` as GMO adapter profile | Provider-specific constraints can leak into generic execution spec, causing multi-exchange evolution pain. | 1) Extract any GMO-only semantics out of generic spec into GMO profile. 2) Keep generic spec with neutral terms and extension hooks. |
docs/audits/docs-content-overlap.md:40:| OVL-05 | Bots API/UX status model duplication | `docs/specs/controlplane-bots.md`, `docs/specs/ui-bots.md` | `docs/specs/controlplane-bots.md` as API schema/status semantics SSOT; `docs/specs/ui-bots.md` as UI behavior/rendering SSOT | Bot status fields (e.g., degraded reasons, states) are often repeated in both docs. API semantics should live in controlplane spec, UI doc should reference them. | 1) In UI spec, replace duplicated field definitions with links/brief mapping tables. 2) Keep authoritative enums/field contracts in controlplane spec. |
docs/audits/docs-content-overlap.md:41:| OVL-06 | Paper E2E runbook duplication | `docs/runbooks/e2e-smoke-runbook.md`, `docs/runbooks/paper_e2e.md` | `docs/runbooks/e2e-smoke-runbook.md` as "one-command" operational runbook SSOT; `docs/runbooks/paper_e2e.md` as deep troubleshooting/reference | Both cover paper path checks; one should be quick path and the other should be expanded diagnostics. Distinguish by depth and audience. | 1) Ensure smoke runbook contains only happy-path + short triage. 2) Keep paper_e2e as detailed manual procedures and edge-case diagnostics. |
docs/audits/docs-content-overlap.md:42:| OVL-07 | Governance baseline duplication | `docs/assumptions.md`, `docs/roadmap.md`, `README.md` | `docs/assumptions.md` as baseline assumptions/constraints SSOT | Guardrails and defaults appear in multiple places; assumptions should be centralized and referenced elsewhere. | 1) Keep roadmap guardrails short and link to assumptions. 2) Keep README to brief security/baseline pointers only. |
docs/status/ultimate-gold-progress-check.md:54:| UG-00 | 全体NFR | P0 | In Progress | 55% | Contracts SSOT、idempotency、dead-man、監査ログ、health/capabilities基盤 | SAFE_MODE統一運用、SoT優先順位の明文化、replay決定性の全体保証 |
docs/status/ultimate-gold-progress-check.md:89:  - Contracts SSOT
docs/roadmap.md:8:- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
docs/roadmap.md:32:- `contracts/` is SSOT and enforced in CI.
docs/assumptions.md:13:1. OpenAPI uses version `1.0.0` as the first SSOT baseline for V2.5+ bootstrap.
docs/README.md:12:- Parallel task safety (SSOT for parallel development safety): [`specs/parallel-task-safety.md`](specs/parallel-task-safety.md)
docs/specs/crosscut/parallel_task_safety_spec.md:1:# Parallel Task Safety Spec (SSOT)
docs/specs/crosscut/parallel_task_safety_spec.md:45:## 4. Profinaut開発：Codexアジャイル「タスク生成」方針
```

## 4) Rule-like file discovery

Command:

```bash
find docs -maxdepth 4 -type f \( \
  -iname "*safety*" -o -iname "*rule*" -o -iname "*guard*" -o -iname "*parallel*" -o \
  -iname "*handoff*" -o -iname "*decision*" -o -iname "*status*.json" -o -iname "*trace*" \
\) -print
```

Output:

```bash
docs/specs/crosscut/parallel_task_safety_spec.md
```
