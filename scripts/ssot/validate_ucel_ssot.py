#!/usr/bin/env python3
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EXCHANGES_DIR = ROOT / "docs" / "exchanges"
COVERAGE_DIR = ROOT / "ucel" / "coverage"


def collect_catalog_ids(node, out):
    if isinstance(node, dict):
        value = node.get("id")
        if isinstance(value, str):
            out.add(value)
        for v in node.values():
            collect_catalog_ids(v, out)
    elif isinstance(node, list):
        for item in node:
            collect_catalog_ids(item, out)


def parse_scalar(value: str):
    v = value.strip()
    if v in ("true", "false"):
        return v == "true"
    return v.strip('"\'')


def parse_coverage(path: Path):
    data = {"entries": []}
    current = None
    in_entries = False
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.split("#", 1)[0].rstrip()
        if not line.strip():
            continue
        stripped = line.strip()
        if stripped == "entries:":
            in_entries = True
            current = None
            continue
        if stripped == "entries: []":
            data["entries"] = []
            in_entries = False
            current = None
            continue
        if not in_entries:
            if ":" in stripped:
                key, val = stripped.split(":", 1)
                data[key.strip()] = parse_scalar(val)
            continue
        if stripped.startswith("- "):
            current = {}
            data["entries"].append(current)
            content = stripped[2:]
            if ":" in content:
                key, val = content.split(":", 1)
                current[key.strip()] = parse_scalar(val)
            continue
        if current is not None and ":" in stripped:
            key, val = stripped.split(":", 1)
            current[key.strip()] = parse_scalar(val)

    return data


def main():
    errors = []
    catalogs = {}

    for d in sorted(EXCHANGES_DIR.iterdir()):
        if not d.is_dir():
            continue
        catalog_path = d / "catalog.json"
        if not catalog_path.exists():
            continue
        venue = d.name
        payload = json.loads(catalog_path.read_text(encoding="utf-8"))
        ids = set()
        collect_catalog_ids(payload, ids)
        catalogs[venue] = ids

    coverages = {p.stem: p for p in COVERAGE_DIR.glob("*.yaml")}

    for venue, ids in catalogs.items():
        if venue not in coverages:
            errors.append(f"venue={venue} reason=missing coverage file")
            continue
        coverage = parse_coverage(coverages[venue])

        if coverage.get("venue") and coverage.get("venue") != venue:
            errors.append(
                f"venue={venue} reason=coverage venue mismatch got={coverage.get('venue')}"
            )

        scope = coverage.get("scope")
        if scope and scope not in {"public_only", "public_private"}:
            errors.append(f"venue={venue} reason=invalid scope={scope}")

        for entry in coverage.get("entries", []):
            if not isinstance(entry, dict):
                errors.append(f"venue={venue} id=<invalid> reason=entry must be mapping")
                continue
            entry_id = entry.get("id")
            if not entry_id:
                errors.append(f"venue={venue} id=<missing> reason=entry id missing")
                continue
            if entry_id not in ids:
                errors.append(
                    f"venue={venue} id={entry_id} reason=coverage id not present in catalog"
                )

        if coverage.get("strict") is True:
            if coverage.get("implemented") is False:
                errors.append(f"venue={venue} reason=strict requires implemented=true")
            if coverage.get("tested") is False:
                errors.append(f"venue={venue} reason=strict requires tested=true")
            for entry in coverage.get("entries", []):
                if not isinstance(entry, dict):
                    errors.append(f"venue={venue} id=<invalid> reason=entry must be mapping")
                    continue
                eid = entry.get("id", "<missing>")
                if entry.get("implemented") is not True:
                    errors.append(
                        f"venue={venue} id={eid} reason=strict requires entry implemented=true"
                    )
                if entry.get("tested") is not True:
                    errors.append(
                        f"venue={venue} id={eid} reason=strict requires entry tested=true"
                    )

    if errors:
        print("UCEL SSOT validation failed:")
        for err in errors:
            print(f"- {err}")
        return 1

    print("UCEL SSOT validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
