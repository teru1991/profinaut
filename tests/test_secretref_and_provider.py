from __future__ import annotations

import os
import time
from pathlib import Path

import pytest

from libs.safety_core.errors import SecError
from libs.safety_core.secrets_provider import Secrets
from libs.safety_core.secrets_ref import parse_secretref


def write_registry(tmp_path: Path) -> Path:
    p = tmp_path / "docs/policy/asset_registry.json"
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(
        """{
  "version":"1",
  "items":[
    {"registry_id":"r1","allowed_schemes":["env"],"scopes":["venue:x:dev"],"max_ttl_seconds":10},
    {"registry_id":"r2","allowed_schemes":["fileenc"],"scopes":["venue:x:dev"],"max_ttl_seconds":30}
  ]
}""",
        encoding="utf-8",
    )
    return p


def test_parse_secretref_requires_fields() -> None:
    with pytest.raises(SecError):
        parse_secretref("env://x#A?registry_id=r1")
    with pytest.raises(SecError):
        parse_secretref("env://x?registry_id=r1&scope=venue:x:dev")
    with pytest.raises(SecError):
        parse_secretref("unknown://x#A?registry_id=r1&scope=venue:x:dev")


def test_registry_enforcement_and_prod_env_forbidden(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    write_registry(tmp_path)
    monkeypatch.chdir(tmp_path)
    s_prod = Secrets(mode="prod", fileenc_base_dir=tmp_path, default_ttl_seconds=1, vault_enabled=False, enabled=True)

    os.environ["ENV_KEY"] = "VALUE1"
    ref_env = s_prod.parse("env://ignored#ENV_KEY?registry_id=r1&scope=venue:x:dev")
    with pytest.raises(SecError) as e:
        s_prod.resolve(ref_env)
    assert e.value.code == "E_SECRET_ENV_FORBIDDEN"


def test_unregistered_registry_is_denied(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    (tmp_path / "docs/policy").mkdir(parents=True, exist_ok=True)
    (tmp_path / "docs/policy/asset_registry.json").write_text('{"version":"1","items":[]}', encoding="utf-8")
    monkeypatch.chdir(tmp_path)

    s = Secrets(mode="dev", fileenc_base_dir=tmp_path, default_ttl_seconds=1)
    os.environ["ENV_KEY"] = "VALUE1"
    ref = s.parse("env://x#ENV_KEY?registry_id=missing&scope=venue:x:dev")
    with pytest.raises(SecError) as e:
        s.resolve(ref)
    assert e.value.code == "E_SECRET_REGISTRY_DENY"


def test_ttl_cache_respects_expiry(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    (tmp_path / "docs/policy").mkdir(parents=True, exist_ok=True)
    (tmp_path / "docs/policy/asset_registry.json").write_text(
        '{"version":"1","items":[{"registry_id":"r1","allowed_schemes":["env"],"scopes":["venue:x:dev"],"max_ttl_seconds":2}]}',
        encoding="utf-8",
    )
    monkeypatch.chdir(tmp_path)

    s = Secrets(mode="dev", fileenc_base_dir=tmp_path, default_ttl_seconds=1)

    os.environ["ENV_KEY"] = "VALUE1"
    ref = s.parse("env://x#ENV_KEY?registry_id=r1&scope=venue:x:dev")
    v1 = s.resolve(ref)
    os.environ["ENV_KEY"] = "VALUE2"
    v_cached = s.resolve(ref)
    assert v1 == "VALUE1"
    assert v_cached == "VALUE1"

    time.sleep(1.2)
    v2 = s.resolve(ref)
    assert v2 == "VALUE2"


def test_fileenc_plaintext_allowed_in_dev_but_forbidden_in_prod(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    (tmp_path / "docs/policy").mkdir(parents=True, exist_ok=True)
    (tmp_path / "docs/policy/asset_registry.json").write_text(
        '{"version":"1","items":[{"registry_id":"r2","allowed_schemes":["fileenc"],"scopes":["venue:x:dev"],"max_ttl_seconds":30}]}',
        encoding="utf-8",
    )
    monkeypatch.chdir(tmp_path)

    (tmp_path / "secrets").mkdir(parents=True, exist_ok=True)
    (tmp_path / "secrets/ex.json").write_text('{"K":"V"}', encoding="utf-8")

    s_dev = Secrets(mode="dev", fileenc_base_dir=tmp_path)
    ref = s_dev.parse("fileenc://secrets/ex.json#K?registry_id=r2&scope=venue:x:dev")
    assert s_dev.resolve(ref) == "V"

    s_prod = Secrets(mode="prod", fileenc_base_dir=tmp_path)
    with pytest.raises(SecError) as e:
        s_prod.resolve(ref)
    assert e.value.code == "E_SECRET_PROVIDER_NOT_CONFIGURED"
