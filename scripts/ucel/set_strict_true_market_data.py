#!/usr/bin/env python3
"""
Patch ucel/coverage/*.yaml: change `strict: false` -> `strict: true` (text replacement, minimum diff).

Scope: ucel/coverage/ (v1 schema, Market Data H category).
       ucel/coverage_v2/ is skipped — already all strict: true.

Rules:
- Only replaces the line `strict: false` at the start of a line (anchored match).
- Does NOT reformat or re-dump YAML (avoids diff explosion).
- Files where strict is already true are left untouched (no diff).
- Unknown file extensions are skipped (safe default).
"""

import os
import re
import sys
from pathlib import Path

# Match the top-level `strict: false` YAML line (must start at line beginning).
STRICT_FALSE_RE = re.compile(r'^(strict:\s*)false(\s*)$', re.MULTILINE)

ALLOWED_EXTS = {'.yaml', '.yml', '.json', '.jsonc', '.toml'}


def patch_file(path: Path) -> int:
    """Return 1 if file was changed, 0 otherwise. Raises on error."""
    if path.suffix.lower() not in ALLOWED_EXTS:
        return 0
    txt = path.read_text(encoding='utf-8')
    if 'strict' not in txt:
        return 0
    if not STRICT_FALSE_RE.search(txt):
        return 0
    new = STRICT_FALSE_RE.sub(r'\1true\2', txt)
    if new == txt:
        return 0
    path.write_text(new, encoding='utf-8')
    print(f'  patched: {path}')
    return 1


def main() -> int:
    # Resolve root relative to script location or CWD
    script_dir = Path(__file__).resolve().parent
    # scripts/ucel/ -> repo root
    repo_root = script_dir.parent.parent
    coverage_dir = repo_root / 'ucel' / 'coverage'

    if not coverage_dir.exists():
        print(f'ERROR: coverage dir not found: {coverage_dir}', file=sys.stderr)
        return 1

    changed = 0
    for p in sorted(coverage_dir.iterdir()):
        if p.is_dir():
            continue
        try:
            changed += patch_file(p)
        except Exception as e:
            print(f'ERROR: failed to patch {p}: {e}', file=sys.stderr)
            return 2

    print(f'Done. patched files: {changed}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
