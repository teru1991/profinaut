from __future__ import annotations

import os
from pathlib import Path

import pytest

from libs.safety_core.errors import SecError
from libs.safety_core.secrets_provider import Secrets


def test_prod_rejects_env_and_plaintext(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    (tmp_path / "docs/policy").mkdir(parents=True, exist_ok=True)
    (tmp_path / "docs/policy/asset_registry.json").write_text(
        '{"version":"1","items":[{"registry_id":"r1","allowed_schemes":["env"],"scopes":["venue:x:dev"],"max_ttl_seconds":10},{"registry_id":"r2","allowed_schemes":["fileenc"],"scopes":["venue:x:prod"],"max_ttl_seconds":30}]}',
        encoding="utf-8",
    )
    monkeypatch.chdir(tmp_path)

    os.environ["ENV_KEY"] = "VALUE1"
    s = Secrets(mode="prod", fileenc_base_dir=tmp_path)

    ref_env = s.parse("env://x#ENV_KEY?registry_id=r1&scope=venue:x:dev")
    with pytest.raises(SecError):
        _ = s.resolve(ref_env)

    (tmp_path / "secrets").mkdir(parents=True, exist_ok=True)
    (tmp_path / "secrets/ex.json").write_text('{"K":"V"}', encoding="utf-8")
    ref_plain = s.parse("fileenc://secrets/ex.json#K?registry_id=r2&scope=venue:x:prod")
    with pytest.raises(SecError):
        _ = s.resolve(ref_plain)
