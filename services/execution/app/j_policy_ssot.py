from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any, Literal

from app.j_policy_yaml_min import YamlMinError, load_yaml_min


class JPolicySSOTError(RuntimeError):
    pass


Decision = Literal["ALLOW", "DENY", "CANCEL_ONLY", "HALT"]


@dataclass(frozen=True)
class Reason:
    code: str
    severity: str
    default_action: Decision
    description: str


@dataclass(frozen=True)
class Transition:
    from_state: str
    to_state: str
    when_all: list[str]
    latch: bool
    reason_code: str


@dataclass(frozen=True)
class JPolicySSOT:
    root: Path
    boundaries: dict[str, Any]
    reasons: dict[str, Reason]
    states: list[str]
    transitions: list[Transition]
    exceptions: dict[str, Any]
    observability: dict[str, Any]
    rbac: dict[str, Any]
    quiet_hours: dict[str, Any]
    forbidden_ops: dict[str, Any]
    dependency_slo: dict[str, Any]
    degraded_levels: dict[str, Any]

    @staticmethod
    def load(root: Path) -> "JPolicySSOT":
        try:
            boundaries = load_yaml_min(root / "boundaries.yml")
            reason_codes = load_yaml_min(root / "reason_codes.yml")
            mode_machine = load_yaml_min(root / "mode_machine.yml")
            exceptions = load_yaml_min(root / "exception_templates.yml")
            observability = load_yaml_min(root / "observability_contract.yml")
            rbac = load_yaml_min(root / "rbac_matrix.yml")
            quiet_hours = load_yaml_min(root / "quiet_hours.yml")
            forbidden_ops = load_yaml_min(root / "forbidden_ops.yml")
            dependency_slo = load_yaml_min(root / "dependency_slo.yml")
            degraded_levels = load_yaml_min(root / "degraded_levels.yml")
        except (YamlMinError, OSError) as e:
            raise JPolicySSOTError(f"SSOT load failed: {e}") from e

        for k in ("schema_version", "fail_close", "required_inputs", "execution", "portfolio"):
            if k not in boundaries:
                raise JPolicySSOTError(f"boundaries.yml missing key: {k}")

        if "reasons" not in reason_codes or not isinstance(reason_codes["reasons"], list):
            raise JPolicySSOTError("reason_codes.yml missing reasons list")
        reasons: dict[str, Reason] = {}
        for r in reason_codes["reasons"]:
            if not isinstance(r, dict):
                raise JPolicySSOTError("reason_codes.yml reasons must be maps")
            code = str(r.get("code", "")).strip()
            if not code:
                raise JPolicySSOTError("reason_codes.yml: empty code")
            if code in reasons:
                raise JPolicySSOTError(f"reason_codes.yml: duplicate code {code}")
            default_action = str(r.get("default_action", "")).strip()
            if default_action not in ("ALLOW", "DENY", "CANCEL_ONLY", "HALT"):
                raise JPolicySSOTError(f"reason_codes.yml: invalid default_action for {code}")
            reasons[code] = Reason(
                code=code,
                severity=str(r.get("severity", "")).strip(),
                default_action=default_action,  # type: ignore[arg-type]
                description=str(r.get("description", "")).strip(),
            )

        for k in ("schema_version", "states", "transitions"):
            if k not in mode_machine:
                raise JPolicySSOTError(f"mode_machine.yml missing key: {k}")
        states = mode_machine["states"]
        if not isinstance(states, list) or any(not isinstance(s, str) for s in states):
            raise JPolicySSOTError("mode_machine.yml states must be list[str]")
        required = {"SAFE", "WARN", "DEGRADED", "CANCEL_ONLY", "HALT"}
        if not required.issubset(set(states)):
            raise JPolicySSOTError("mode_machine.yml must include SAFE,WARN,DEGRADED,CANCEL_ONLY,HALT")

        transitions: list[Transition] = []
        for tr in mode_machine["transitions"]:
            if not isinstance(tr, dict):
                raise JPolicySSOTError("mode_machine.yml transitions must be maps")
            fs = str(tr.get("from", "")).strip()
            ts = str(tr.get("to", "")).strip()
            when_all = tr.get("when_all", [])
            latch = tr.get("latch", False)
            rc = str(tr.get("reason_code", "")).strip()
            if fs not in states or ts not in states:
                raise JPolicySSOTError(f"mode_machine.yml: invalid state in transition {fs}->{ts}")
            if not isinstance(when_all, list) or any(not isinstance(c, str) for c in when_all):
                raise JPolicySSOTError("mode_machine.yml: when_all must be list[str]")
            if rc not in reasons:
                raise JPolicySSOTError(f"mode_machine.yml: unknown reason_code {rc}")
            transitions.append(Transition(from_state=fs, to_state=ts, when_all=when_all, latch=bool(latch), reason_code=rc))

        return JPolicySSOT(
            root=root,
            boundaries=boundaries,
            reasons=reasons,
            states=states,
            transitions=transitions,
            exceptions=exceptions,
            observability=observability,
            rbac=rbac,
            quiet_hours=quiet_hours,
            forbidden_ops=forbidden_ops,
            dependency_slo=dependency_slo,
            degraded_levels=degraded_levels,
        )


def ssot_root_from_repo(repo_root: Path) -> Path:
    return repo_root / "docs" / "specs" / "domains" / "J"
