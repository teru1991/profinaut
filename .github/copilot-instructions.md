# Copilot Instructions: Parallel-Safety Guardrails

## Objective
Keep pull requests single-scope to avoid conflicts during parallel development.

## Scope map
- `dashboard`: `apps/web/**` OR `services/dashboard-api/**`
- `contracts`: `contracts/**`
- `sdk`: `sdk/python/**`
- `docs`: `docs/**`
- `repo-hygiene` (exact files only):
  - `.github/CODEOWNERS`
  - `.github/pull_request_template.md`
  - `.github/copilot-instructions.md`
  - `.github/workflows/scope-guard.yml`
  - `docs/specs/parallel-task-safety.md`

## Required behavior
- Keep each PR within exactly one scope.
- Refuse mixed-scope edits in one PR.
- If user asks for mixed-scope changes, propose split PRs.
- Never bypass `scope-guard` workflow expectations.
