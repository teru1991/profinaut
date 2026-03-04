from __future__ import annotations

import re
import sys
from pathlib import Path

SCAN_ROOTS = [Path("libs"), Path("services"), Path("tests")]
PRINT_OR_LOGGER = re.compile(r"\b(print\s*\(|logger\.(debug|info|warning|error|exception)\s*\()")
SECRET_PATTERN = re.compile(r"(?i)(authorization|api[_-]?key|secret|bearer\s+[a-z0-9\-_\.]+)")


def should_scan(path: Path) -> bool:
    text_path = str(path)
    if "fixtures" in text_path or "vendor" in text_path or "__pycache__" in text_path:
        return False
    return path.suffix in {".py"}


def main() -> int:
    offenders: list[tuple[str, int, str]] = []
    for root in SCAN_ROOTS:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if not path.is_file() or not should_scan(path):
                continue
            content = path.read_text(encoding="utf-8", errors="ignore").splitlines()
            for lineno, line in enumerate(content, start=1):
                if PRINT_OR_LOGGER.search(line) and SECRET_PATTERN.search(line) and "***" not in line:
                    offenders.append((str(path), lineno, line.strip()))

    if offenders:
        print("redaction lint failed. Potential raw secret patterns in direct output lines:", file=sys.stderr)
        for path, lineno, line in offenders[:100]:
            print(f"- {path}:{lineno} :: {line}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
