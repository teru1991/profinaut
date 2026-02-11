from __future__ import annotations

import json
import sys
from pathlib import Path

from jsonschema import Draft202012Validator


def validate_schema_files(schema_dir: Path) -> int:
    errors = 0
    for schema_file in sorted(schema_dir.glob("*.schema.json")):
        with schema_file.open("r", encoding="utf-8") as handle:
            schema = json.load(handle)
        try:
            Draft202012Validator.check_schema(schema)
            print(f"[ok] {schema_file}")
        except Exception as exc:  # noqa: BLE001
            errors += 1
            print(f"[error] {schema_file}: {exc}")
    return errors


def main() -> int:
    repo_root = Path(__file__).resolve().parent.parent
    schema_dir = repo_root / "contracts" / "schemas"
    if not schema_dir.exists():
        print(f"[error] missing schema directory: {schema_dir}")
        return 1

    errors = validate_schema_files(schema_dir)
    if errors:
        print(f"\nValidation failed with {errors} schema error(s).")
        return 1

    print("\nAll JSON Schema files are valid Draft 2020-12 schemas.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
