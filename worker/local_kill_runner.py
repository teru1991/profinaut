from __future__ import annotations

from datetime import UTC, datetime
import json
from pathlib import Path

from libs.safety_core.kill import apply_local_kill
from libs.safety_core.runtime import audit_writer, store


DEFAULT_KILL_FILE = Path('/var/run/profinaut_kill_switch.json')


def run_local_kill_if_requested(kill_file: Path = DEFAULT_KILL_FILE) -> bool:
    if not kill_file.exists():
        return False

    payload = json.loads(kill_file.read_text(encoding='utf-8'))
    reason = payload.get('reason', 'local_kill_file_triggered')
    evidence = {
        'trace_id': payload.get('trace_id', 'local-kill-file'),
        'audit_id': payload.get('audit_id', 'local-kill-file-audit'),
    }
    apply_local_kill(store=store, audit=audit_writer, reason=reason, evidence_ref=evidence, now=datetime.now(UTC))
    return True
