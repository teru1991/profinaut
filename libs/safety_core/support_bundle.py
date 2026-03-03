from __future__ import annotations

from datetime import UTC, datetime
import hashlib
import json
from pathlib import Path

from libs.safety_core.redaction import redact
from libs.safety_core.store import SafetyStore


def collect(
    store: SafetyStore,
    now: datetime | None = None,
    include_recent_audit: bool = True,
    include_health: bool = True,
    include_config_hash: bool = True,
    audit_log_path: Path | None = None,
    output_root: Path | None = None,
) -> Path:
    ts = now or datetime.now(UTC)
    root = output_root or Path("worker/.state/support_bundles")
    bundle_dir = root / f"bundle-{ts.strftime('%Y%m%dT%H%M%SZ')}"
    bundle_dir.mkdir(parents=True, exist_ok=True)

    state = store.get_current_state()
    directives = store.get_directives()

    (bundle_dir / "safety_state.json").write_text(
        json.dumps(redact(state.__dict__ if state else {}), ensure_ascii=False, indent=2),
        encoding="utf-8",
    )
    (bundle_dir / "active_directives.json").write_text(
        json.dumps(redact([d.__dict__ for d in directives]), ensure_ascii=False, indent=2),
        encoding="utf-8",
    )

    if include_recent_audit:
        source = audit_log_path or Path("libs/safety_core/_audit_log/audit.jsonl")
        audit_out = bundle_dir / "recent_audit.jsonl"
        if source.exists():
            lines = source.read_text(encoding="utf-8").splitlines()[-200:]
            redacted_lines: list[str] = []
            for line in lines:
                try:
                    redacted_lines.append(json.dumps(redact(json.loads(line)), ensure_ascii=False))
                except json.JSONDecodeError:
                    continue
            audit_out.write_text("\n".join(redacted_lines) + ("\n" if redacted_lines else ""), encoding="utf-8")
        else:
            audit_out.write_text("", encoding="utf-8")

    if include_health:
        health = {"status": "captured", "captured_at": ts.isoformat(), "component": "safety_core"}
        (bundle_dir / "health_snapshot.json").write_text(json.dumps(health, indent=2), encoding="utf-8")

    if include_config_hash:
        payload = json.dumps([d.__dict__ for d in directives], sort_keys=True, ensure_ascii=False)
        digest = hashlib.sha256(payload.encode("utf-8")).hexdigest()
        (bundle_dir / "config_hash.txt").write_text(digest + "\n", encoding="utf-8")

    return bundle_dir
