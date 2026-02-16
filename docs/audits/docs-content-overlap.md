# DOCS-003: Documentation Content Overlap Audit

Task ID: `DOCS-003`  
Scope: `docs-overlap-audit`  
Mode: safe audit only (no deletions, no content migration in this PR)

## 1) Definition of "overlap" and audit goals

### Overlap definition (for this repository)
This audit treats content as **overlap** when one or more of the following are true:

1. **Same purpose overlap**: different files explain the same user intent (e.g., "how to run paper E2E").
2. **Same procedure overlap**: different files provide near-identical command flows or step lists.
3. **Same spec/rule overlap**: policy, behavior, or API semantics are described in multiple places with competing authority.
4. **Same status/progress overlap**: roadmap/progress/changelog files all provide partial project status and can diverge.

### Goals
- **Reduce duplication** so docs stay consistent.
- **Preserve link stability** so existing bookmarks/URLs do not break.
- **Enable safe follow-up PRs** by clearly identifying canonical (source-of-truth) docs before any move/refactor.

---

## 2) Scan method and suspected overlap areas

Repository scan basis:
- Root: `README.md`
- Docs tree under `docs/` (including `specs/`, `runbooks/`, `status/`, `workplan/`, `audits/`)

The overlap candidates below are based on filename inventory + spot checks of document purpose/sections.

## 3) Overlap candidates, canonical proposals, and rationale

| Area ID | Suspected overlap area | Current files involved | Proposed canonical (SSOT) | Rationale | Follow-up touch plan (next PRs) |
|---|---|---|---|---|---|
| OVL-01 | Project progress / delivered scope summary | `README.md`, `docs/roadmap.md`, `docs/changelog.md` | `docs/roadmap.md` for plan + current step state; `docs/changelog.md` for chronological release log; `README.md` only as concise entrypoint | README currently includes long delivered-feature lists that are also represented by roadmap/changelog. Keep README thin, and point to roadmap/changelog for authoritative detail. | 1) Trim README "included" section to short summary + links. 2) Keep roadmap as plan/status. 3) Keep changelog purely date-ordered change history. |
| OVL-02 | Ultimate Gold planning/progress duplication | `docs/workplan/ultimate-gold-implementation-feature-list.md`, `docs/status/ultimate-gold-progress-check.md`, `docs/status/progress-updates/UG-P0-*.md` | `docs/workplan/ultimate-gold-implementation-feature-list.md` = requirements/catalog SSOT; `docs/status/ultimate-gold-progress-check.md` = current status dashboard SSOT; `docs/status/progress-updates/*.md` = append-only evidence logs | The workplan and progress-check both contain requirement mapping + milestone status narrative; risk of drift is high. Split roles explicitly: "what should exist" vs "what currently exists" vs "event/evidence log". | 1) Remove requirement restatement from progress-check; keep live status only. 2) In progress updates, enforce short template + links to PR/commit only. 3) Add explicit cross-links among the 3 layers. |
| OVL-03 | Dangerous operation rule definition split | `docs/specs/dangerous-ops-taxonomy.md`, `docs/specs/dangerous-ops-confirmation.md` | `docs/specs/dangerous-ops-taxonomy.md` as policy/rule SSOT; `docs/specs/dangerous-ops-confirmation.md` as UX/API confirmation flow spec only | Taxonomy (what is dangerous and why) and confirmation (how operator confirms) should be separated but non-overlapping. Current boundary is easy to blur. | 1) Keep taxonomy doc free of UI workflow details. 2) Keep confirmation doc free of category/policy duplication; only reference taxonomy IDs. |
| OVL-04 | Execution specification layering | `docs/specs/execution.md`, `docs/specs/execution-gmo.md` | `docs/specs/execution.md` as provider-agnostic execution contract/behavior SSOT; `docs/specs/execution-gmo.md` as GMO adapter profile | Provider-specific constraints can leak into generic execution spec, causing multi-exchange evolution pain. | 1) Extract any GMO-only semantics out of generic spec into GMO profile. 2) Keep generic spec with neutral terms and extension hooks. |
| OVL-05 | Bots API/UX status model duplication | `docs/specs/controlplane-bots.md`, `docs/specs/ui-bots.md` | `docs/specs/controlplane-bots.md` as API schema/status semantics SSOT; `docs/specs/ui-bots.md` as UI behavior/rendering SSOT | Bot status fields (e.g., degraded reasons, states) are often repeated in both docs. API semantics should live in controlplane spec, UI doc should reference them. | 1) In UI spec, replace duplicated field definitions with links/brief mapping tables. 2) Keep authoritative enums/field contracts in controlplane spec. |
| OVL-06 | Paper E2E runbook duplication | `docs/runbooks/e2e-smoke-runbook.md`, `docs/runbooks/paper_e2e.md` | `docs/runbooks/e2e-smoke-runbook.md` as "one-command" operational runbook SSOT; `docs/runbooks/paper_e2e.md` as deep troubleshooting/reference | Both cover paper path checks; one should be quick path and the other should be expanded diagnostics. Distinguish by depth and audience. | 1) Ensure smoke runbook contains only happy-path + short triage. 2) Keep paper_e2e as detailed manual procedures and edge-case diagnostics. |
| OVL-07 | Governance baseline duplication | `docs/assumptions.md`, `docs/roadmap.md`, `README.md` | `docs/assumptions.md` as baseline assumptions/constraints SSOT | Guardrails and defaults appear in multiple places; assumptions should be centralized and referenced elsewhere. | 1) Keep roadmap guardrails short and link to assumptions. 2) Keep README to brief security/baseline pointers only. |

---

## 4) Safe migration policy ("do not delete")

For all follow-up cleanup PRs, use this policy:

1. **No file deletion in first-pass cleanup.**
2. Convert superseded docs into **thin stubs**:
   - Keep file path unchanged.
   - Keep title.
   - Add top banner: "This document moved" with canonical link.
   - Keep only a brief summary (3-10 lines).
3. Add/update backlinks from canonical doc to major stubs where needed.
4. If content is split (not fully moved), state scope boundary explicitly:
   - "This file covers X only. For Y see <link>."
5. Delay hard deletions until:
   - at least one release cycle passes, and
   - inbound links are audited.

### Stub template (recommended)

```md
# <Old Title>

> ⚠️ This document has been consolidated into: `<relative-link-to-canonical>`

## Summary (kept for link stability)
- <1-3 bullets retained for context>

For authoritative details, see the canonical document above.
```

---

## 5) Actionable follow-up task map

Use one small PR per overlap area:

- `DOCS-004` → OVL-01 (README/roadmap/changelog role split)
- `DOCS-005` → OVL-02 (Ultimate Gold 3-layer model cleanup)
- `DOCS-006` → OVL-03 (dangerous-op taxonomy vs confirmation boundary)
- `DOCS-007` → OVL-04 (execution generic vs GMO profile split)
- `DOCS-008` → OVL-05 (controlplane-bots vs ui-bots contract ownership)
- `DOCS-009` → OVL-06 (paper E2E runbook dedup)
- `DOCS-010` → OVL-07 (assumptions centralization)

Each follow-up PR should:
- edit only target docs,
- preserve paths via stub policy,
- include before/after link map,
- avoid cross-domain code/config changes.

---

## 6) Codex-ready English prompt (for next cleanup PR)

```text
TASK ID: DOCS-00X
Title: Canonicalize overlapping documentation for <OVL-XX area>
Scope: docs-canonicalization

Execution mode: SINGLE-RUN
Depends-on: DOCS-003

Goal:
Reduce duplication for <OVL-XX> while preserving stable links.

Allowed paths:
- <explicit list of docs files for this area>

Forbidden paths:
- contracts/**
- services/**
- apps/**
- .github/**
- docker-compose.yml
- any lockfiles
- root package.json

Requirements:
1) Keep one canonical source-of-truth document for this area.
2) Do NOT delete old files; convert superseded docs into thin stubs.
3) Add top-of-file "moved" links in stubs.
4) Remove duplicated normative text from non-canonical docs.
5) Add cross-links so navigation still works from old entry points.

Deliverables:
- Updated canonical doc.
- Stubbed non-canonical docs with migration banner.
- Short "link map" section listing old path -> canonical path.

Acceptance criteria:
- No forbidden paths touched.
- No docs deleted.
- Canonical ownership is explicit in each touched doc.
- Markdown renders correctly on GitHub.

Verification:
- Run: git diff --name-only
- Run: <markdown lint/check command available in repo>
- Confirm all moved links are relative and valid.
```

---

## 7) GitHub rendering check note

This file uses standard GitHub Markdown features only (ATX headings, fenced code blocks, and pipe tables) and is structured to render without extensions.
