from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

AuthStrength = Literal["none", "basic", "step_up"]


@dataclass(frozen=True, slots=True)
class Actor:
    kind: Literal["human", "service"]
    actor_id: str


@dataclass(frozen=True, slots=True)
class Session:
    actor: Actor
    auth_strength: AuthStrength
    session_id: str
    mode: str
