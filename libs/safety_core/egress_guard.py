from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from libs.safety_core.egress_policy import EgressPolicy
from libs.safety_core.errors import err
from libs.safety_core.redaction import redact, safe_json, scan_obj

E_EGRESS_DENY = "E_EGRESS_DENY"
E_EGRESS_TOO_LARGE = "E_EGRESS_TOO_LARGE"


@dataclass(frozen=True, slots=True)
class EgressResult:
    allowed: bool
    payload_json: str


def prepare_egress(*, target: str, payload: Any, policy: EgressPolicy | None = None) -> EgressResult:
    pol = policy or EgressPolicy.load()

    redacted = redact(payload)
    payload_json = safe_json(redacted)
    if len(payload_json.encode("utf-8")) > pol.max_payload_bytes:
        raise err(E_EGRESS_TOO_LARGE, "egress payload too large", target=target)

    findings = scan_obj(redacted)
    if target in pol.deny_targets:
        raise err(E_EGRESS_DENY, "egress target denied", target=target)

    if findings and not pol.allow_if_redacted:
        raise err(E_EGRESS_DENY, "egress contains secret indicators", target=target)

    if pol.allow_targets and (target not in pol.allow_targets):
        raise err(E_EGRESS_DENY, "egress target not allowlisted", target=target)

    return EgressResult(allowed=True, payload_json=payload_json)
