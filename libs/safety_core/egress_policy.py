from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

from libs.safety_core.errors import err

E_EGRESS_POLICY_INVALID = "E_EGRESS_POLICY_INVALID"


@dataclass(frozen=True, slots=True)
class EgressPolicy:
    allow_targets: set[str]
    deny_targets: set[str]
    allow_if_redacted: bool
    max_payload_bytes: int

    @staticmethod
    def load(path: Path | None = None) -> "EgressPolicy":
        p = path or Path("docs/policy/llm_egress_policy.json")
        if not p.exists():
            return EgressPolicy(
                allow_targets=set(),
                deny_targets={"llm", "public_http"},
                allow_if_redacted=False,
                max_payload_bytes=200000,
            )
        try:
            obj = json.loads(p.read_text(encoding="utf-8"))
            return EgressPolicy(
                allow_targets=set(map(str, obj.get("allow_targets", []))),
                deny_targets=set(map(str, obj.get("deny_targets", ["llm", "public_http"]))),
                allow_if_redacted=bool(obj.get("allow_if_redacted", True)),
                max_payload_bytes=int(obj.get("max_payload_bytes", 200000)),
            )
        except Exception as e:
            raise err(E_EGRESS_POLICY_INVALID, "egress policy invalid", error=str(e)) from None
