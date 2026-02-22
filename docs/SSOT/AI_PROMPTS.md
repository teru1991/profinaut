# AI_PROMPTS (Copy/Paste templates for multi-AI execution)

This file provides operational prompt templates for ChatGPT/Claude/Gemini/Copilot.
It is a usage library (non-SSOT) aligned to Docs Development OS.

- Canonical entrypoint: `docs/SSOT/README_AI.md`
- Status SSOT: `docs/status/status.json`
- Trace SSOT: `docs/status/trace-index.json`

---

## 1) Magic Prompt (detailed, universal)

Use this full prompt when starting any task:

- Read in strict order: `docs/SSOT/README_AI.md` → `docs/status/status.json` → `docs/handoff/HANDOFF.json` → `docs/decisions/decisions.md`.
- Before planning or editing, inspect `docs/status/status.json` keys: `active_task`, `open_prs`, `locks_held`, `owner`, `state`, `last_updated`.
- If `locks_held` conflicts with intended scope, STOP and return a lock-conflict response.
- Build a task card that includes: Task ID, Scope, Allowed paths, Forbidden paths, Required LOCKS, Dependencies, Verification steps.
- Only edit files inside Allowed paths. Do not touch forbidden paths.
- If pausing/stopping/credit-out before completion, update `docs/handoff/HANDOFF.json` with `what_done`, `what_next`, `errors`, `commands_next`.
- Any "progress made" claim requires evidence updates in `docs/status/status.json` and/or `docs/status/progress-updates/*`.
- Record evidence links in `docs/status/trace-index.json` (SSOT) as the single canonical trace index.
- If uncertain, add/update assumptions or decisions via the existing mechanism (`docs/assumptions.md`, `docs/decisions/decisions.md`) before irreversible changes.

---

## 2) Role Prompts

### Planner Prompt

- Read order: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
- Produce smallest safe task split with dependencies and required LOCKS.
- Include Allowed/Forbidden paths in every task card.
- If LOCK collision is detected, stop and send back replan request (no execution).
- Ensure each task has DoD, test/verification commands, and rollback notes.
- If assumptions are needed, record them in assumptions/decisions flow before execution.
- Do not claim progress unless `status.json`/progress-updates are updated.

### Builder Prompt

- Read order: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
- Implement only within Allowed paths and declared scope.
- Re-check `active_task`, `open_prs`, `locks_held` immediately before editing.
- If lock conflict/forbidden touch is needed, stop and return for replanning.
- Keep diffs minimal; preserve SSOT boundaries and avoid duplicate docs.
- On stop/credit-out, update HANDOFF required fields before exit.
- Any progress statement must be backed by `status.json` or progress update entries.

### Verifier Prompt

- Read order: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
- Validate Definition of Done with reproducible commands and expected results.
- Check scope compliance (Allowed/Forbidden) and LOCK safety.
- If evidence is missing from `status.json`/progress updates/trace index, fail verification.
- If uncertain behavior exists, request assumptions/decisions update before pass.
- Produce concise repro steps, pass/fail matrix, and residual risk notes.

### Reviewer Prompt

- Read order: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
- Review PR for SSOT integrity, lock safety, and rollback risk.
- Reject if LOCK conflicts were ignored or forbidden paths changed.
- Verify that stop/handoff obligations are satisfied for interrupted work.
- Verify progress claims have concrete evidence updates (`status.json`, progress updates, trace index).
- Flag decision gaps and require explicit decision log updates when ambiguity remains.

---

## 3) Short Copy/Paste Blocks by model

### ChatGPT short (1 paragraph)

Read `docs/SSOT/README_AI.md`, then `docs/status/status.json`, `docs/handoff/HANDOFF.json`, then `docs/decisions/decisions.md`; treat `status.json` as SSOT and check `active_task/open_prs/locks_held` before action, declare Allowed/Forbidden paths and Required LOCKS in your task card, stop on LOCK conflict, record trace links in `docs/status/trace-index.json`, and if stopping early update HANDOFF with `what_done/what_next/errors/commands_next`.

### Claude short (1 paragraph)

Follow OS read order `README_AI → status.json → HANDOFF → decisions`; use `status.json` as authoritative runtime state (`active_task`, `open_prs`, `locks_held`), operate only in Allowed paths, stop and return on LOCK conflict, require evidence updates for any progress claim (`status.json` or progress updates), write trace evidence to `docs/status/trace-index.json`, and update HANDOFF fields before any pause/credit-out.

### Gemini short (1 paragraph)

Start by reading `docs/SSOT/README_AI.md`, `docs/status/status.json`, `docs/handoff/HANDOFF.json`, and `docs/decisions/decisions.md`; then create a task card with scope + Allowed/Forbidden + LOCKS, execute only safe paths, halt on lock collisions, ensure progress claims are backed by status/progress updates, keep trace links in `docs/status/trace-index.json`, and always write HANDOFF before stopping mid-task.

---

## 4) Detailed Copy/Paste Blocks by model

### ChatGPT detailed

- Read: `README_AI` → `status.json` → `HANDOFF` → `decisions`.
- Inspect status fields: `active_task`, `open_prs`, `locks_held`, `owner`, `state`.
- Draft task card: Task ID / Scope / Allowed / Forbidden / Required LOCKS / Dependencies / DoD.
- Stop if LOCK conflict exists.
- Edit only allowed files; keep SSOT unique.
- If uncertain, write assumption/decision updates first.
- Update evidence: `status.json` or `progress-updates` + `trace-index.json`.
- If pausing, update HANDOFF required keys.

### Claude detailed

- Read in canonical order and summarize constraints before doing work.
- Validate active ownership and lock status in `status.json`.
- Produce plan with explicit dependency/lock graph.
- Enforce forbidden path guardrails strictly.
- Fail fast on lock collisions or missing decision authority.
- Verify every progress claim has evidence artifact updates.
- Record trace links in `trace-index.json`.
- On credit-out, update HANDOFF with actionable next commands.

### Gemini detailed

- Load OS docs in required sequence and restate SSOT boundaries.
- Parse runtime status keys (`active_task`, `open_prs`, `locks_held`) before editing.
- Generate execution checklist with Allowed/Forbidden and LOCK requirements.
- If lock conflict appears, return STOP + replan message.
- Keep changes minimal and scoped.
- Ensure DoD verification commands are reproducible.
- Sync progress evidence to status/progress files and trace index.
- On interruption, complete HANDOFF fields before ending response.
