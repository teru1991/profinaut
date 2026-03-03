from __future__ import annotations

from datetime import UTC, datetime, timedelta
import json
from pathlib import Path

from libs.safety_core.slo import SloRecorder


def test_halt_to_block_metric_recorded(tmp_path: Path) -> None:
    out = tmp_path / "slo.jsonl"
    recorder = SloRecorder(out)
    start = datetime.now(UTC)
    stop = start + timedelta(milliseconds=45)
    value = recorder.record_halt_to_block_ms(start, stop)
    assert value == 45.0
    line = out.read_text(encoding="utf-8").splitlines()[0]
    payload = json.loads(line)
    assert payload["metric"] == "halt_to_block_ms"


def test_interlock_detect_to_escalate_metric_recorded() -> None:
    recorder = SloRecorder()
    start = datetime.now(UTC)
    stop = start + timedelta(milliseconds=22)
    value = recorder.record_interlock_detect_to_escalate_ms(start, stop)
    assert value == 22.0
    alerts = recorder.evaluate_alerts("interlock_detect_to_escalate_ms", value)
    assert alerts == []
