from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from libs.safety_core.redaction import redact, safe_str


@dataclass(frozen=True, slots=True)
class AccessReviewReport:
    markdown: str


def _load_json(p: Path) -> Any:
    if not p.exists():
        return None
    return json.loads(p.read_text(encoding="utf-8"))


def generate_access_review(*, out_path: Path | None = None) -> AccessReviewReport:
    danger = _load_json(Path("docs/policy/danger_ops_policy.json")) or {}
    change = _load_json(Path("docs/policy/change_mgmt_policy.json")) or {}
    reg = _load_json(Path("docs/policy/asset_registry.json")) or {}

    danger = redact(danger)
    change = redact(change)
    reg = redact(reg)

    lines: list[str] = []
    lines.append("# Access Review Report (Monthly)")
    lines.append("")
    lines.append("## Summary")
    lines.append(f"- allow rules: {len(danger.get('allow', []) or [])}")
    lines.append(f"- dangerous ops: {len(danger.get('danger_ops', []) or [])}")
    lines.append(f"- controlled changes: {len(change.get('controlled_changes', []) or [])}")
    lines.append(f"- registry items: {len(reg.get('items', []) or [])}")
    lines.append("")
    lines.append("## Dangerous Ops Catalog")
    for op in sorted(set(map(str, danger.get("danger_ops", []) or []))):
        lines.append(f"- {op}")
    lines.append("")
    lines.append("## Controlled Changes (Extended Change Mgmt)")
    for op in sorted(set(map(str, change.get("controlled_changes", []) or []))):
        lines.append(f"- {op}")
    lines.append("")
    lines.append("## Allow Rules")
    for it in danger.get("allow", []) or []:
        op = safe_str(it.get("op"))
        actor = safe_str(it.get("actor"))
        modes = ", ".join(map(str, it.get("modes", []) or []))
        scopes = ", ".join(map(str, it.get("scopes", []) or []))
        lines.append(f"- op={op} actor={actor} modes=[{modes}] scopes=[{scopes}]")
    lines.append("")
    lines.append("## Asset Registry")
    for it in reg.get("items", []) or []:
        rid = safe_str(it.get("registry_id"))
        schemes = ", ".join(map(str, it.get("allowed_schemes", []) or []))
        scopes = ", ".join(map(str, it.get("scopes", []) or []))
        max_ttl = safe_str(it.get("max_ttl_seconds"))
        lines.append(f"- {rid}: schemes=[{schemes}] scopes=[{scopes}] max_ttl={max_ttl}")

    md = "\n".join(lines) + "\n"
    if out_path:
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(md, encoding="utf-8")
    return AccessReviewReport(markdown=md)
