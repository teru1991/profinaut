# Parallel Task Safety Specification

## Purpose
This document defines guardrails to keep concurrent coding tasks safe, reviewable, and conflict-resistant.

## Principles
1. **Single-scope PRs only**: each PR must belong to exactly one scope.
2. **Minimal blast radius**: avoid cross-domain edits in one PR.
3. **Explicit ownership**: CODEOWNERS routes review to responsible maintainers.
4. **Deterministic checks**: CI must block multi-scope changes.

## Repository Scopes
- `dashboard`: `apps/web/**` or `services/dashboard-api/**`
- `contracts`: `contracts/**`
- `sdk`: `sdk/python/**`
- `docs`: `docs/**`
- `repo-hygiene` (exact files only):
  - `.github/CODEOWNERS`
  - `.github/pull_request_template.md`
  - `.github/copilot-instructions.md`
  - `.github/workflows/scope-guard.yml`
  - `docs/specs/parallel-task-safety.md`

## Rules for Parallel Work
- Open one PR per scope.
- Do not mix source scopes in the same PR.
- Keep PR title and description aligned to the single scope.
- If a change is truly cross-scope, split into stacked PRs.

## CI Enforcement
`scope-guard` workflow:
- computes changed files in PR range,
- maps each file to one of the approved scopes,
- fails on unknown paths,
- fails when more than one scope is present,
- passes only when exactly one scope is detected.

## Operational Guidance
- Rebase frequently to reduce overlap.
- Land infra/repo-hygiene changes separately before feature work.
- Prefer small, incremental PRs to reduce merge conflicts.
