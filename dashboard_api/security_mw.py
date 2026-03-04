from __future__ import annotations

from typing import Callable

from libs.safety_core.audit import AuditEvent, JsonlAuditWriter
from libs.safety_core.audit_health import AuditHealth
from libs.safety_core.authz import Authz
from libs.safety_core.danger_ops import confirm, issue_challenge
from libs.safety_core.session import Actor, Session


class SecurityMiddleware:
    def __init__(self) -> None:
        self._authz = Authz()
        self._audit_health = AuditHealth()
        self._audit = JsonlAuditWriter(audit_health=self._audit_health)

    def _session_from_headers(self, headers: dict[str, str]) -> Session:
        actor_kind = headers.get("x-actor-kind", "human")
        actor_id = headers.get("x-actor-id", "unknown")
        session_id = headers.get("x-session-id", "unknown")
        mode = headers.get("x-mode", "dev")
        strength = headers.get("x-auth-strength", "basic")
        return Session(
            actor=Actor(kind=actor_kind, actor_id=actor_id),  # type: ignore[arg-type]
            auth_strength=strength,  # type: ignore[arg-type]
            session_id=session_id,
            mode=mode,
        )

    def handle(
        self,
        *,
        headers: dict[str, str],
        op: str,
        scope: str,
        danger_token: str | None,
        handler: Callable[[], dict],
    ) -> dict:
        sess = self._session_from_headers(headers)

        self._authz.require(op=op, actor_kind=sess.actor.kind, mode=sess.mode, scope=scope)

        if self._authz.is_dangerous(op):
            if danger_token is None:
                ch = issue_challenge(session=sess, op=op, scope=scope)
                return {"error": "danger_ops_required", "challenge": {"token": ch.token, "expires_at": ch.expires_at}}
            confirm(session=sess, op=op, scope=scope, token=danger_token, audit_health=self._audit_health)

        try:
            self._audit.write_event(
                AuditEvent(
                    event_type="op_call",
                    actor=f"{sess.actor.kind}:{sess.actor.actor_id}",
                    scope=scope,
                    mode_from=sess.mode,
                    mode_to=sess.mode,
                    reason=f"op={op}",
                    ttl=60,
                    evidence_ref={"session_id": sess.session_id},
                )
            )
        except Exception:
            pass

        return handler()
