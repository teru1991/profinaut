from __future__ import annotations

import os
from pathlib import Path

import pytest

from libs.safety_core.crypto.fileenc_format import FileEncContext, loads_record
from libs.safety_core.errors import SecError
from libs.safety_core.redaction import safe_str
from libs.safety_core.secrets_provider import Secrets


def setup_registry(tmp_path: Path) -> None:
    (tmp_path / "docs/policy").mkdir(parents=True, exist_ok=True)
    (tmp_path / "docs/policy/asset_registry.json").write_text(
        '{"version":"1","items":[{"registry_id":"r2","allowed_schemes":["fileenc"],"scopes":["venue:x:prod"],"max_ttl_seconds":30}]}',
        encoding="utf-8",
    )


def test_encrypt_decrypt_roundtrip_and_tamper(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.chdir(tmp_path)
    setup_registry(tmp_path)

    (tmp_path / "secrets").mkdir(parents=True, exist_ok=True)
    src = tmp_path / "secrets/ex.json"
    src.write_text('{"K":"VALUE_DUMMY","X":"Y"}', encoding="utf-8")

    os.environ["FILEENC_PASSPHRASE"] = "passphrase-test-123456"

    from scripts.fileenc_tool import encrypt_json

    ctx = FileEncContext(path="secrets/ex.enc", field="K", registry_id="r2", scope="venue:x:prod", version_hint=None)
    dst = tmp_path / "secrets/ex.enc"
    encrypt_json(src=src, dst=dst, ctx=ctx)

    rec = loads_record(dst.read_bytes())
    assert rec["magic"] == "UCEL-FILEENC"
    assert int(rec["v"]) == 1

    s = Secrets(mode="prod", fileenc_base_dir=tmp_path, default_ttl_seconds=1)
    ref = s.parse("fileenc://secrets/ex.enc#K?registry_id=r2&scope=venue:x:prod")
    assert s.resolve(ref) == "VALUE_DUMMY"

    b = bytearray(dst.read_bytes())
    b[-10] ^= 0xFF
    dst.write_bytes(bytes(b))
    s.purge()
    with pytest.raises(SecError) as e:
        _ = s.resolve(ref)
    assert "VALUE_DUMMY" not in safe_str(str(e.value))


def test_aad_mismatch_rejected(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.chdir(tmp_path)
    setup_registry(tmp_path)
    (tmp_path / "secrets").mkdir(parents=True, exist_ok=True)
    (tmp_path / "secrets/ex.json").write_text('{"K":"VALUE_DUMMY"}', encoding="utf-8")
    os.environ["FILEENC_PASSPHRASE"] = "passphrase-test-abcdef"

    from scripts.fileenc_tool import encrypt_json

    ctx = FileEncContext(path="secrets/ex.enc", field="K", registry_id="r2", scope="venue:x:prod", version_hint=None)
    dst = tmp_path / "secrets/ex.enc"
    encrypt_json(src=tmp_path / "secrets/ex.json", dst=dst, ctx=ctx)

    s = Secrets(mode="prod", fileenc_base_dir=tmp_path, default_ttl_seconds=1)
    bad_ref = s.parse("fileenc://secrets/ex.enc#K?registry_id=r2&scope=venue:x:prodX")
    with pytest.raises(SecError):
        _ = s.resolve(bad_ref)


def test_wrong_passphrase_rejected(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.chdir(tmp_path)
    setup_registry(tmp_path)
    (tmp_path / "secrets").mkdir(parents=True, exist_ok=True)
    (tmp_path / "secrets/ex.json").write_text('{"K":"VALUE_DUMMY"}', encoding="utf-8")

    os.environ["FILEENC_PASSPHRASE"] = "passphrase-OK"
    from scripts.fileenc_tool import encrypt_json

    ctx = FileEncContext(path="secrets/ex.enc", field="K", registry_id="r2", scope="venue:x:prod", version_hint=None)
    dst = tmp_path / "secrets/ex.enc"
    encrypt_json(src=tmp_path / "secrets/ex.json", dst=dst, ctx=ctx)

    os.environ["FILEENC_PASSPHRASE"] = "passphrase-WRONG"
    s = Secrets(mode="prod", fileenc_base_dir=tmp_path)
    ref = s.parse("fileenc://secrets/ex.enc#K?registry_id=r2&scope=venue:x:prod")
    with pytest.raises(SecError) as e:
        _ = s.resolve(ref)
    assert "passphrase-OK" not in safe_str(str(e.value))
    assert "VALUE_DUMMY" not in safe_str(str(e.value))
