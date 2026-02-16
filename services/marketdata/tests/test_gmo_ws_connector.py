from __future__ import annotations

import asyncio
import json
import sqlite3
from pathlib import Path
from typing import Any

from services.marketdata.app.gmo_ws_connector import GmoPublicWsConnector, GmoWsConfig


class _FakeWs:
    def __init__(self, messages: list[str]):
        self._messages = list(messages)
        self.sent: list[str] = []

    async def send(self, message: str) -> None:
        self.sent.append(message)

    def __aiter__(self) -> "_FakeWs":
        return self

    async def __anext__(self) -> str:
        if not self._messages:
            raise StopAsyncIteration
        return self._messages.pop(0)


class _FakeConnect:
    def __init__(self, ws: _FakeWs):
        self.ws = ws
        self.calls = 0

    def __call__(self, _url: str) -> "_FakeConnect":
        self.calls += 1
        return self

    async def __aenter__(self) -> _FakeWs:
        return self.ws

    async def __aexit__(self, exc_type: Any, exc: Any, tb: Any) -> bool:
        return False


def test_connector_disabled_does_not_connect() -> None:
    ws = _FakeWs([])
    connect = _FakeConnect(ws)
    connector = GmoPublicWsConnector(GmoWsConfig(enabled=False), connect_fn=connect)

    asyncio.run(connector.run_forever())

    assert connect.calls == 0


def test_connector_ingests_raw_messages_into_db(monkeypatch, tmp_path: Path) -> None:
    db_file = tmp_path / "md.sqlite3"
    bronze_root = tmp_path / "bronze"

    monkeypatch.setenv("OBJECT_STORE_BACKEND", "fs")
    monkeypatch.setenv("DB_DSN", f"sqlite:///{db_file}")
    monkeypatch.setenv("BRONZE_FS_ROOT", str(bronze_root))

    ws = _FakeWs([
        json.dumps(
            {
                "channel": "ticker",
                "symbol": "BTC_JPY",
                "timestamp": "2026-02-16T00:00:00.000Z",
                "price": "123.45",
            }
        )
    ])
    connect = _FakeConnect(ws)
    connector = GmoPublicWsConnector(
        GmoWsConfig(
            enabled=True,
            channels=("ticker",),
            backoff_initial_seconds=0.01,
            backoff_max_seconds=0.01,
            reconnect_seconds=0.01,
        ),
        connect_fn=connect,
    )

    async def _run_once() -> None:
        task = asyncio.create_task(connector.run_forever())
        await asyncio.sleep(0.02)
        await connector.stop()
        await asyncio.wait_for(task, timeout=1)

    asyncio.run(_run_once())

    assert connect.calls >= 1
    assert len(ws.sent) == 1
    assert json.loads(ws.sent[0])["command"] == "subscribe"

    conn = sqlite3.connect(db_file)
    row = conn.execute("SELECT venue_id, stream_name, endpoint FROM raw_ingest_meta").fetchone()
    sub_row = conn.execute("SELECT meta_json FROM ws_subscriptions LIMIT 1").fetchone()
    assert sub_row is not None
    assert "market_id" in sub_row[0]

    assert row is not None
    assert row[0] == "gmo"
    assert row[1] == "ticker"
    assert row[2].startswith("wss://")
