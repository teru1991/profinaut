#!/usr/bin/env python3
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
JDIR = ROOT / "docs" / "specs" / "domains" / "J"

REQUIRED_FILES = [
    ROOT / "docs" / "specs" / "domains" / "J_risk_policy_gate.md",
    JDIR / "boundaries.yml",
    JDIR / "reason_codes.yml",
    JDIR / "mode_machine.yml",
    JDIR / "exception_templates.yml",
    JDIR / "observability_contract.yml",
    JDIR / "rbac_matrix.yml",
    JDIR / "quiet_hours.yml",
    JDIR / "forbidden_ops.yml",
    JDIR / "failure_modes.md",
    JDIR / "dependency_slo.yml",
    JDIR / "degraded_levels.yml",
    JDIR / "retention_redaction.md",
    JDIR / "bootstrap.md",
]

def fail(msg: str) -> int:
    print(f"[j_ssot_validate] ERROR: {msg}", file=sys.stderr)
    return 1

def ok(msg: str) -> None:
    print(f"[j_ssot_validate] OK: {msg}")

def main() -> int:
    for p in REQUIRED_FILES:
        if not p.exists():
            return fail(f"missing file: {p}")
        if p.is_file() and p.stat().st_size == 0:
            return fail(f"empty file: {p}")
    ok("all required SSOT files exist and are non-empty")

    try:
        import yaml  # type: ignore
    except Exception as e:
        ok(f"PyYAML not available; skip YAML parse checks ({e})")
        return 0

    def load_yaml(path: Path):
        with path.open("r", encoding="utf-8") as f:
            return yaml.safe_load(f)

    boundaries = load_yaml(JDIR / "boundaries.yml")
    for k in ["schema_version", "fail_close", "required_inputs", "execution", "portfolio"]:
        if k not in boundaries:
            return fail(f"boundaries.yml missing key: {k}")
    ok("boundaries.yml minimal keys")

    reasons = load_yaml(JDIR / "reason_codes.yml")
    if "schema_version" not in reasons or "reasons" not in reasons:
        return fail("reason_codes.yml missing schema_version/reasons")
    codes = [r.get("code") for r in reasons["reasons"]]
    if len(codes) != len(set(codes)):
        return fail("reason_codes.yml has duplicate codes")
    ok("reason_codes.yml minimal keys and unique codes")

    mm = load_yaml(JDIR / "mode_machine.yml")
    for k in ["schema_version", "states", "transitions"]:
        if k not in mm:
            return fail(f"mode_machine.yml missing key: {k}")
    ok("mode_machine.yml minimal keys")

    return 0

if __name__ == "__main__":
    raise SystemExit(main())
