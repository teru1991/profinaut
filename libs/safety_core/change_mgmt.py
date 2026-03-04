from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

from libs.safety_core.errors import err

E_CHANGE_POLICY_INVALID = "E_CHANGE_POLICY_INVALID"


@dataclass(frozen=True, slots=True)
class ChangeMgmtPolicy:
    controlled_changes: set[str]

    @staticmethod
    def load(path: Path | None = None) -> "ChangeMgmtPolicy":
        p = path or Path("docs/policy/change_mgmt_policy.json")
        if not p.exists():
            return ChangeMgmtPolicy(controlled_changes=set())
        try:
            obj = json.loads(p.read_text(encoding="utf-8"))
            return ChangeMgmtPolicy(controlled_changes=set(map(str, obj.get("controlled_changes", []))))
        except Exception as e:
            raise err(E_CHANGE_POLICY_INVALID, "change mgmt policy invalid", error=str(e)) from None

    def is_controlled(self, change_op: str) -> bool:
        return change_op in self.controlled_changes
