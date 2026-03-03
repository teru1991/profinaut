#!/usr/bin/env python3
import json
from pathlib import Path

OUT = Path("ucel") / "fixtures" / "support_bundle" / "manifest.json"

def main() -> int:
    manifest = json.loads(OUT.read_text(encoding="utf-8"))
    OUT.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f"normalized {OUT}")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
