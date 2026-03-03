from __future__ import annotations

from libs.safety_core.crypto.rot import RotUnavailable


def build_macos_keychain_rot() -> RotUnavailable:
    return RotUnavailable(reason="macOS Keychain RoT not configured in this environment")
