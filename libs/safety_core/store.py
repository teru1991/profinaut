from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import asdict
from datetime import datetime
import json
import os
from pathlib import Path

from libs.safety_core.models import SafetyDirective, SafetyMode, SafetyStateV1, ScopeKind


class SafetyStore(ABC):
    @abstractmethod
    def get_directives(self) -> list[SafetyDirective]: ...

    @abstractmethod
    def put_directive(self, directive: SafetyDirective) -> None: ...

    @abstractmethod
    def expire_directives(self, now: datetime) -> list[SafetyDirective]: ...

    @abstractmethod
    def get_current_state(self) -> SafetyStateV1 | None: ...

    @abstractmethod
    def set_current_state(self, state: SafetyStateV1) -> None: ...


class InMemorySafetyStore(SafetyStore):
    def __init__(self) -> None:
        self._directives: list[SafetyDirective] = []
        self._state: SafetyStateV1 | None = None

    def get_directives(self) -> list[SafetyDirective]:
        return list(self._directives)

    def put_directive(self, directive: SafetyDirective) -> None:
        self._directives.append(directive)

    def expire_directives(self, now: datetime) -> list[SafetyDirective]:
        expired: list[SafetyDirective] = []
        active: list[SafetyDirective] = []
        for directive in self._directives:
            if directive.expires_at() <= now:
                expired.append(directive)
            else:
                active.append(directive)
        self._directives = active
        return expired

    def get_current_state(self) -> SafetyStateV1 | None:
        return self._state

    def set_current_state(self, state: SafetyStateV1) -> None:
        self._state = state


class JsonFileSafetyStore(SafetyStore):
    def __init__(self, path: Path) -> None:
        self._path = path
        self._path.parent.mkdir(parents=True, exist_ok=True)
        self._state: SafetyStateV1 | None = None
        self._directives: list[SafetyDirective] = []
        self._load()

    def _load(self) -> None:
        if not self._path.exists():
            return
        try:
            data = json.loads(self._path.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, OSError):
            self._state = SafetyStateV1(mode=SafetyMode.SAFE, reason="store_recovery_fail_closed")
            self._directives = []
            return
        state = data.get("state")
        if state:
            self._state = SafetyStateV1(
                schema_version=state.get("schema_version", 1),
                state_id=state["state_id"],
                mode=SafetyMode(state["mode"]),
                reason=state["reason"],
                activated_by=state.get("activated_by"),
                activated_at=state["activated_at"],
            )
        self._directives = [
            SafetyDirective(
                scope_kind=ScopeKind(item["scope_kind"]),
                selector=item["selector"],
                mode=SafetyMode(item["mode"]),
                ttl_seconds=item["ttl_seconds"],
                reason=item["reason"],
                actor=item["actor"],
                evidence=item["evidence"],
                issued_at=item["issued_at"],
            )
            for item in data.get("directives", [])
        ]

    def _persist(self) -> None:
        payload = {
            "state": asdict(self._state) if self._state else None,
            "directives": [asdict(d) for d in self._directives],
        }
        tmp_path = self._path.with_suffix(self._path.suffix + ".tmp")
        with tmp_path.open("w", encoding="utf-8") as f:
            json.dump(payload, f, ensure_ascii=False)
            f.flush()
            os.fsync(f.fileno())
        tmp_path.replace(self._path)

    def get_directives(self) -> list[SafetyDirective]:
        return list(self._directives)

    def put_directive(self, directive: SafetyDirective) -> None:
        self._directives.append(directive)
        self._persist()

    def expire_directives(self, now: datetime) -> list[SafetyDirective]:
        expired: list[SafetyDirective] = []
        active: list[SafetyDirective] = []
        for directive in self._directives:
            if directive.expires_at() <= now:
                expired.append(directive)
            else:
                active.append(directive)
        self._directives = active
        self._persist()
        return expired

    def get_current_state(self) -> SafetyStateV1 | None:
        return self._state

    def set_current_state(self, state: SafetyStateV1) -> None:
        self._state = state
        self._persist()
