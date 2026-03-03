#!/usr/bin/env python3
import argparse, hashlib, json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path("ucel") / "fixtures" / "golden"
OUT = ROOT / "manifest.json"
ALLOW_EXT = {".json", ".txt", ".ndjson"}


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def collect_files():
    files = []
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file() or path.name == "manifest.json":
            continue
        if path.suffix.lower() not in ALLOW_EXT:
            continue
        rel = path.relative_to(ROOT).as_posix()
        files.append({"path": rel, "sha256": sha256_file(path), "bytes": path.stat().st_size})
    return files


def check_only(expected):
    if not OUT.exists():
        return 1
    actual = json.loads(OUT.read_text(encoding="utf-8"))
    return 0 if actual.get("files") == expected.get("files") else 1


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--accept", action="store_true", help="rewrite manifest")
    args = parser.parse_args()

    manifest = {
        "version": 1,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "files": collect_files(),
    }

    if args.accept:
        OUT.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
        print(f"WROTE {OUT} ({len(manifest['files'])} files)")
        return 0

    if check_only(manifest) == 0:
        print("golden manifest is up to date")
        return 0
    print("golden manifest out of date; run: python3 scripts/ucel/update_golden_manifest.py --accept")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
