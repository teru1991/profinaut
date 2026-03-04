from __future__ import annotations

from pathlib import Path

import pytest

from libs.safety_core.access_review import generate_access_review


def test_access_review_generates_markdown(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    (tmp_path / "docs/policy").mkdir(parents=True, exist_ok=True)
    (tmp_path / "docs/policy/danger_ops_policy.json").write_text(
        '{"version":"1","allow":[],"danger_ops":["start_live"]}',
        encoding="utf-8",
    )
    (tmp_path / "docs/policy/change_mgmt_policy.json").write_text(
        '{"version":"1","controlled_changes":["policy_update"]}',
        encoding="utf-8",
    )
    (tmp_path / "docs/policy/asset_registry.json").write_text('{"version":"1","items":[]}', encoding="utf-8")
    monkeypatch.chdir(tmp_path)

    out = tmp_path / "out.md"
    rep = generate_access_review(out_path=out)
    assert out.exists()
    assert "Dangerous Ops Catalog" in rep.markdown
