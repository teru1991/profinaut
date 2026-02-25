#!/usr/bin/env python3
import argparse
import json
import re
from pathlib import Path

MD_LINK_DOC_RE = re.compile(r"\]\((docs/[A-Za-z0-9_./*-]+)\)")
INLINE_DOC_RE = re.compile(r"`(docs/[A-Za-z0-9_./*-]+)`")
FORBIDDEN_CTX_RE = re.compile(r"\b(api_key|secret|token|private_key|authorization|cookie)\b\s*[:=]", re.IGNORECASE)

SKIP_EXACT = {
    "docs/verification/docs-inventory-before.txt",
    "docs/verification/docs-inventory-after.txt",
    "docs/verification/docs-scan-before.txt",
    "docs/verification/docs-lint-result.txt",
}
LEGACY_REF_ALLOWLIST = {
    'docs/README.md',
    'docs/specs/system/terminology.md',
    'docs/roadmap.md',
}

SKIP_PREFIXES = (
    "docs/legacy/",
    "docs/status/progress-updates/",
    "docs/audits/",
    "docs/exchanges/",
    "docs/data-platform/",
)


def strip_code_blocks(text: str) -> str:
    out, in_fence = [], False
    for line in text.splitlines():
        if line.strip().startswith("```"):
            in_fence = not in_fence
            continue
        if not in_fence:
            out.append(line)
    return "\n".join(out)


def should_skip(rel: str) -> bool:
    if rel in SKIP_EXACT:
        return True
    return any(rel.startswith(p) for p in SKIP_PREFIXES)


def iter_docs_files(root: Path):
    for p in sorted((root / "docs").rglob("*")):
        if p.is_file():
            rel = str(p.relative_to(root)).replace("\\", "/")
            if should_skip(rel):
                continue
            yield p


def safe_exists(root: Path, ref: str) -> bool:
    ref = ref.rstrip("/")
    if "*" in ref or "..." in ref:
        return True
    rp = (root / ref).resolve()
    try:
        rp.relative_to(root.resolve())
    except Exception:
        return False
    return rp.exists()


def check_trace_paths(root: Path, trace_path: Path, errors: list[str]):
    data = json.loads(trace_path.read_text(encoding="utf-8"))
    tasks = data.get("tasks", {})
    if not isinstance(tasks, dict):
        errors.append("TRACE: tasks must be an object/dict")
        return

    for task_id, task in tasks.items():
        if not isinstance(task, dict):
            continue
        val = task.get("path")
        if isinstance(val, str) and val and not (root / val).exists():
            errors.append(f"TRACE missing path [{task_id}]: {val}")
        for key in ("artifacts", "verification_evidence"):
            lst = task.get(key)
            if isinstance(lst, list):
                for item in lst:
                    if isinstance(item, str) and item and not (root / item).exists():
                        errors.append(f"TRACE missing {key} [{task_id}]: {item}")


def extract_refs(line: str):
    for rx in (MD_LINK_DOC_RE, INLINE_DOC_RE):
        for m in rx.finditer(line):
            yield m.group(1)


def check_docs(root: Path, errors: list[str]):
    for path in iter_docs_files(root):
        rel = str(path.relative_to(root)).replace("\\", "/")
        text = strip_code_blocks(path.read_text(encoding="utf-8", errors="ignore"))
        is_stub = "NOT CANONICAL" in text

        for i, line in enumerate(text.splitlines(), start=1):
            if rel.endswith(".md") and FORBIDDEN_CTX_RE.search(line):
                errors.append(f"FORBIDDEN key-context: {rel}:{i}")

            for ref in extract_refs(line):
                if ref.startswith("docs/legacy/") and not is_stub and rel not in LEGACY_REF_ALLOWLIST:
                    errors.append(f"LEGACY REF: {rel}:{i}: {ref}")
                    continue
                if not safe_exists(root, ref):
                    errors.append(f"MISSING REF: {rel}:{i}: {ref}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".")
    ap.add_argument("--trace", default="docs/status/trace-index.json")
    args = ap.parse_args()

    root = Path(args.root).resolve()
    trace_path = root / args.trace
    errors: list[str] = []

    if not trace_path.exists():
        errors.append(f"TRACE: file not found: {args.trace}")
    else:
        try:
            check_trace_paths(root, trace_path, errors)
        except Exception as exc:
            errors.append(f"TRACE: parse/check error: {exc}")

    check_docs(root, errors)

    if errors:
        print("FAIL")
        for e in errors:
            print(f"- {e}")
        raise SystemExit(1)
    print("PASS")


if __name__ == "__main__":
    main()
