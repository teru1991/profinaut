from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

from app.j_policy_ssot import JPolicySSOT, JPolicySSOTError, ssot_root_from_repo


@dataclass(frozen=True)
class SelfCheckItem:
    ok: bool
    name: str
    detail: str
    evidence: dict[str, Any]


def selfcheck(repo_root: Path) -> list[SelfCheckItem]:
    items: list[SelfCheckItem] = []
    ssot_root = ssot_root_from_repo(repo_root)

    required = [
        ssot_root / "boundaries.yml",
        ssot_root / "reason_codes.yml",
        ssot_root / "mode_machine.yml",
        ssot_root / "exception_templates.yml",
        ssot_root / "observability_contract.yml",
        ssot_root / "rbac_matrix.yml",
        ssot_root / "quiet_hours.yml",
        ssot_root / "forbidden_ops.yml",
        ssot_root / "dependency_slo.yml",
        ssot_root / "degraded_levels.yml",
    ]
    missing = [str(p) for p in required if not p.exists()]
    items.append(
        SelfCheckItem(
            ok=(len(missing) == 0),
            name="j_ssot_files_exist",
            detail="all required J SSOT files exist" if not missing else "missing J SSOT files",
            evidence={"missing": missing},
        )
    )

    try:
        _ = JPolicySSOT.load(ssot_root)
        items.append(SelfCheckItem(ok=True, name="j_ssot_load_validate", detail="SSOT loaded and validated", evidence={}))
    except JPolicySSOTError as e:
        items.append(SelfCheckItem(ok=False, name="j_ssot_load_validate", detail="SSOT invalid", evidence={"error": str(e)}))

    return items
