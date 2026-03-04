from __future__ import annotations

import hashlib
import hmac
import json
import time
from dataclasses import dataclass
from typing import Any

from libs.safety_core.audit_health import AuditHealth
from libs.safety_core.crypto.rot_passphrase import require_passphrase
from libs.safety_core.errors import err
from libs.safety_core.redaction import safe_str
from libs.safety_core.session import Session

E_DANGER_OPS_REQUIRED = "E_DANGER_OPS_REQUIRED"
E_DANGER_OPS_TOKEN_INVALID = "E_DANGER_OPS_TOKEN_INVALID"
E_DANGER_OPS_EXPIRED = "E_DANGER_OPS_EXPIRED"
E_DANGER_OPS_STEP_UP_REQUIRED = "E_DANGER_OPS_STEP_UP_REQUIRED"


@dataclass(frozen=True, slots=True)
class Challenge:
    token: str
    expires_at: float


def _sign(payload: bytes) -> str:
    key = hashlib.sha256(require_passphrase().encode("utf-8")).digest()
    sig = hmac.new(key, payload, hashlib.sha256).hexdigest()
    return sig


def _pack(obj: dict[str, Any]) -> bytes:
    s = json.dumps(obj, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
    return s.encode("utf-8")


def issue_challenge(*, session: Session, op: str, scope: str, ttl_seconds: int = 120) -> Challenge:
    exp = time.time() + max(10, int(ttl_seconds))
    obj = {
        "v": 1,
        "op": op,
        "scope": scope,
        "actor": session.actor.actor_id,
        "actor_kind": session.actor.kind,
        "session_id": session.session_id,
        "mode": session.mode,
        "exp": exp,
    }
    payload = _pack(obj)
    sig = _sign(payload)
    token = json.dumps({"payload": obj, "sig": sig}, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
    return Challenge(token=token, expires_at=exp)


def confirm(*, session: Session, op: str, scope: str, token: str, audit_health: AuditHealth) -> None:
    audit_health.require_ok_for_danger_ops()

    if session.auth_strength != "step_up":
        raise err(E_DANGER_OPS_STEP_UP_REQUIRED, "step-up authentication required for dangerous operations", op=op)

    try:
        obj = json.loads(token)
        payload = obj["payload"]
        sig = obj["sig"]
        payload_bytes = _pack(payload)
        expected = _sign(payload_bytes)
        if not hmac.compare_digest(str(sig), expected):
            raise ValueError("sig mismatch")
    except Exception as e:
        raise err(E_DANGER_OPS_TOKEN_INVALID, "danger ops token invalid", error=safe_str(str(e))) from None

    exp = float(payload.get("exp", 0))
    if time.time() > exp:
        raise err(E_DANGER_OPS_EXPIRED, "danger ops token expired", op=op)

    if payload.get("op") != op or payload.get("scope") != scope:
        raise err(E_DANGER_OPS_TOKEN_INVALID, "danger ops token mismatch", op=op, scope=scope)
    if payload.get("actor") != session.actor.actor_id or payload.get("session_id") != session.session_id:
        raise err(E_DANGER_OPS_TOKEN_INVALID, "danger ops token not bound to session")
