# Docs Finalize One Shot — Verification Evidence

What this task ensures:
- Decisions are canonical under docs/decisions/** (audits does not host decisions, only results/stubs)
- Safety rules are canonical under docs/specs/crosscut/**
- task-generation policy is canonical as a single enforced doc; others are legacy/stubs
- Reference-0 NOT CANONICAL stubs are pruned
- Canonical consolidation completed for progress/workplan/troubleshooting
- trace-index paths exist
- docs lint gate passes

Quick checks:
```bash
test -f docs/status/trace-index.json
test -f tools/docs_lint.py
python tools/docs_lint.py --root . --trace docs/status/trace-index.json
```
