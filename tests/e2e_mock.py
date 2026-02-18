from __future__ import annotations

import os
import signal
import subprocess
import time
from pathlib import Path

import requests


def _poll_json(url: str, timeout_s: float = 10.0) -> dict:
    deadline = time.time() + timeout_s
    while time.time() < deadline:
        try:
            response = requests.get(url, timeout=1)
            if response.status_code == 200:
                return response.json()
        except Exception:
            pass
        time.sleep(0.2)
    raise AssertionError(f"timeout polling {url}")


def _parse_metrics(text: str) -> dict[str, float]:
    metrics: dict[str, float] = {}
    for line in text.splitlines():
        if not line or line.startswith("#"):
            continue
        key, value = line.split(" ", 1)
        metrics[key] = float(value)
    return metrics


def test_e2e_mock_metrics_and_health() -> None:
    port = 18181
    env = os.environ.copy()
    env["PYTHONPATH"] = str(Path(__file__).resolve().parents[1])

    proc = subprocess.Popen(
        [
            "python",
            "-m",
            "services.marketdata.app.main",
            "--port",
            str(port),
            "--config",
            "config/collector.toml",
            "--mock",
            "--mock-disconnect-every",
            "4",
            "--mock-mongo-down-ms",
            "1200",
            "--mock-binary-rate",
            "0.5",
        ],
        env=env,
    )
    try:
        health = _poll_json(f"http://127.0.0.1:{port}/healthz", timeout_s=15)
        assert health["service"] == "marketdata"
        assert health["descriptors_loaded_count"] >= 1

        try:
            m1 = requests.get(f"http://127.0.0.1:{port}/metrics", timeout=2)
        except requests.RequestException as exc:
            raise AssertionError(f"failed to fetch initial metrics from http://127.0.0.1:{port}/metrics") from exc
        assert m1.status_code == 200
        before = _parse_metrics(m1.text)

        deadline = time.time() + 8
        after = before
        while time.time() < deadline:
            try:
                m2 = requests.get(f"http://127.0.0.1:{port}/metrics", timeout=2)
            except requests.RequestException:
                time.sleep(0.3)
                continue
            after = _parse_metrics(m2.text)
            if after.get("ingest_messages_total", 0) > before.get("ingest_messages_total", 0):
                break
            time.sleep(0.3)

        assert after["ingest_messages_total"] > before.get("ingest_messages_total", 0)
        assert after["reconnect_count"] >= before.get("reconnect_count", 0)
        assert after["dedup_dropped_total"] >= before.get("dedup_dropped_total", 0)
    finally:
        proc.send_signal(signal.SIGINT)
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
