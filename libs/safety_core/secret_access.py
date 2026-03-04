from __future__ import annotations

import os
from pathlib import Path

from libs.safety_core.redaction import safe_str
from libs.safety_core.secrets_provider import Secrets

_SECRETS = Secrets(mode=os.environ.get("APP_MODE", "dev"), fileenc_base_dir=Path("."), default_ttl_seconds=30)


def get_secret_by_ref(ref_str: str) -> str:
    ref = _SECRETS.parse(ref_str)
    return _SECRETS.resolve(ref)


def get_secretref_from_env(env_name: str) -> str:
    v = os.environ.get(env_name)
    if not v:
        raise RuntimeError(f"missing secretref env: {safe_str(env_name)}")
    return v


def get_secret_from_env_ref(env_name: str) -> str:
    return get_secret_by_ref(get_secretref_from_env(env_name))
