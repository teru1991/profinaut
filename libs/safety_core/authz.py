from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from libs.safety_core.change_mgmt import ChangeMgmtPolicy
from libs.safety_core.errors import err

E_AUTHZ_DENY = "E_AUTHZ_DENY"
E_AUTHZ_POLICY_INVALID = "E_AUTHZ_POLICY_INVALID"


@dataclass(frozen=True, slots=True)
class AuthzDecision:
    allowed: bool
    reason: str


class Authz:
    def __init__(self, *, policy_path: Path | None = None) -> None:
        self._path = policy_path or Path("docs/policy/danger_ops_policy.json")
        self._policy = self._load()
        self._change_mgmt = ChangeMgmtPolicy.load()

    def _load(self) -> dict[str, Any]:
        if not self._path.exists():
            return {"version": "1", "allow": [], "danger_ops": []}
        try:
            obj = json.loads(self._path.read_text(encoding="utf-8"))
            if not isinstance(obj, dict):
                raise ValueError("policy must be object")
            return obj
        except Exception as e:
            raise err(E_AUTHZ_POLICY_INVALID, "authz policy invalid", error=str(e)) from None

    def is_dangerous(self, op: str) -> bool:
        d = self._policy.get("danger_ops", [])
        return (op in set(map(str, d))) or self._change_mgmt.is_controlled(op)

    def authorize(self, *, op: str, actor_kind: str, mode: str, scope: str) -> AuthzDecision:
        allow = self._policy.get("allow", [])
        for it in allow:
            if str(it.get("op")) != op:
                continue
            if str(it.get("actor")) != actor_kind:
                continue
            modes = set(map(str, it.get("modes", [])))
            if modes and mode not in modes:
                continue
            scopes = set(map(str, it.get("scopes", [])))
            if scopes and ("*" not in scopes) and (scope not in scopes):
                continue
            return AuthzDecision(True, "policy_allow")
        return AuthzDecision(False, "deny_by_default")

    def require(self, *, op: str, actor_kind: str, mode: str, scope: str) -> None:
        dec = self.authorize(op=op, actor_kind=actor_kind, mode=mode, scope=scope)
        if not dec.allowed:
            raise err(E_AUTHZ_DENY, "operation not allowed", op=op, reason=dec.reason, mode=mode, scope=scope)
