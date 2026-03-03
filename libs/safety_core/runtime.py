from __future__ import annotations

from pathlib import Path

from libs.safety_core.audit import JsonlAuditWriter
from libs.safety_core.interlock_engine import InterlockEngine
from libs.safety_core.slo import SloRecorder
from libs.safety_core.store import JsonFileSafetyStore

STORE_PATH = Path("worker/.state/safety_state.json")

store = JsonFileSafetyStore(STORE_PATH)
audit_writer = JsonlAuditWriter()
interlock_engine = InterlockEngine()
slo_recorder = SloRecorder()
idempotency_keys: set[str] = set()
