from __future__ import annotations

from libs.safety_core.crypto.rot import RotUnavailable


def build_windows_dpapi_rot() -> RotUnavailable:
    return RotUnavailable(reason="windows DPAPI RoT not configured in this environment")
