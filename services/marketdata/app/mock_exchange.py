from __future__ import annotations

import asyncio
import json
import os
import random
import time
from dataclasses import dataclass, field
from datetime import UTC, datetime
from typing import Any

from fastapi import APIRouter, WebSocket
from fastapi.responses import JSONResponse


def _now() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


@dataclass
class MockScenario:
    enabled: bool = os.getenv("MOCK_ENABLED", "0").strip() == "1"
    gap_every: int = int(os.getenv("MOCK_GAP_EVERY", "0"))
    disconnect_every: int = int(os.getenv("MOCK_DISCONNECT_EVERY", "0"))
    silence_ms: int = int(os.getenv("MOCK_SILENCE_MS", "0"))
    mongo_down_ms: int = int(os.getenv("MOCK_MONGO_DOWN_MS", "0"))
    binary_rate: float = float(os.getenv("MOCK_BINARY_RATE", "0"))


@dataclass
class MockRuntime:
    scenario: MockScenario
    ingest_messages_total: int = 0
    reconnect_count: int = 0
    spool_bytes: int = 0
    spool_segments: int = 0
    spool_replay_backlog: int = 0
    dedup_dropped_total: int = 0
    last_msg_time: str | None = None
    last_reconnect_time: str | None = None
    connection_state: str = "INIT"
    _tick: int = 0
    _mongo_down_until_ms: float = 0

    async def run(self, stop_event: asyncio.Event) -> None:
        while not stop_event.is_set():
            await asyncio.sleep(0.1)
            self._tick += 1
            self.ingest_messages_total += 1
            self.last_msg_time = _now()
            self.connection_state = "RUNNING"

            if self.scenario.disconnect_every > 0 and self._tick % self.scenario.disconnect_every == 0:
                self.reconnect_count += 1
                self.last_reconnect_time = _now()

            current_time_ms = time.time() * 1000
            if self.scenario.mongo_down_ms > 0:
                if current_time_ms >= self._mongo_down_until_ms: # Mongo is currently UP or just recovered
                    if self._mongo_down_until_ms > 0: # If it just recovered, reset to 0
                        self._mongo_down_until_ms = 0
                    elif self.scenario.disconnect_every > 0 and self._tick % self.scenario.disconnect_every == 0: # If up, and time to start new downtime
                        self._mongo_down_until_ms = current_time_ms + self.scenario.mongo_down_ms
            mongo_down = current_time_ms < self._mongo_down_until_ms
            if mongo_down:
                self.spool_bytes += 512
                self.spool_replay_backlog += 1
                self.spool_segments = max(1, self.spool_bytes // (1024 * 4))
            elif self.spool_bytes > 0:
                self.spool_bytes = max(0, self.spool_bytes - 1024)
                self.spool_replay_backlog = max(0, self.spool_replay_backlog - 2)
                if self.spool_replay_backlog > 0:
                    self.dedup_dropped_total += 1

            if self.scenario.silence_ms > 0:
                await asyncio.sleep(self.scenario.silence_ms / 1000)

    def health(self) -> dict[str, Any]:
        mongo_down = time.time() * 1000 < self._mongo_down_until_ms
        return {
            "service": "marketdata",
            "version": "0.1.0",
            "connector_instance_id": "mock-connector-1",
            "config_loaded": True,
            "descriptors_loaded_count": 1,
            "instances": [
                {
                    "name": "mock-v4",
                    "enabled": self.scenario.enabled,
                    "descriptor": "config/exchanges/mock_v4.toml",
                    "validation_status": "ok",
                    "instance_state": self.connection_state,
                }
            ],
            "connections": [
                {
                    "conn_id": "mock-public-0",
                    "state": self.connection_state,
                    "url_index": 0,
                    "last_msg_time": self.last_msg_time,
                    "last_reconnect_time": self.last_reconnect_time,
                }
            ],
            "persistence": {
                "mongo_state": "DOWN" if mongo_down else "OK",
                "spool": {
                    "enabled": True,
                    "bytes": self.spool_bytes,
                    "segments": self.spool_segments,
                    "replay_backlog": self.spool_replay_backlog,
                },
                "dedup": {"enabled": True, "config": {"window_seconds": 30}},
            },
            "time_quality": {"server_time_ratio": 1.0, "skew_ms_p95": 0.0},
        }

    def metrics(self) -> dict[str, float]:
        return {
            "ingest_messages_total": float(self.ingest_messages_total),
            "reconnect_count": float(self.reconnect_count),
            "spool_bytes": float(self.spool_bytes),
            "spool_segments": float(self.spool_segments),
            "spool_replay_backlog": float(self.spool_replay_backlog),
            "dedup_dropped_total": float(self.dedup_dropped_total),
        }


def build_router(runtime: MockRuntime) -> APIRouter:
    router = APIRouter()

    @router.get("/mock/rest/time")
    async def rest_time() -> JSONResponse:
        return JSONResponse({"server_time": _now()})

    @router.get("/mock/rest/snapshot")
    async def rest_snapshot(symbol: str = "BTC_JPY") -> JSONResponse:
        return JSONResponse({"symbol": symbol, "bid": "100", "ask": "101", "ts": _now()})

    @router.websocket("/mock/ws/public")
    async def ws_public(websocket: WebSocket) -> None:
        await websocket.accept()
        ack_id = 0
        while True:
            payload = await websocket.receive()
            if payload.get("type") == "websocket.disconnect":
                break
            if payload.get("bytes"):
                continue
            text = payload.get("text")
            if not text:
                continue
            msg = json.loads(text)
            if msg.get("op") == "ping":
                await websocket.send_text(json.dumps({"op": "pong", "ts": _now()}))
                continue
            if msg.get("op") in {"subscribe", "unsubscribe"}:
                ack_id += 1
                await websocket.send_text(
                    json.dumps({"type": "ack", "op": msg.get("op"), "correlation_id": msg.get("correlation_id"), "ack_id": ack_id})
                )
                seq = runtime.ingest_messages_total + 1
                event = {"channel": msg.get("channel", "trades"), "seq": seq, "price": "100.5", "size": "0.1", "ts": _now()}
                if runtime.scenario.gap_every > 0 and seq % runtime.scenario.gap_every == 0:
                    event["seq"] = seq + 1
                if runtime.scenario.binary_rate > 0 and random.random() < runtime.scenario.binary_rate:
                    await websocket.send_bytes(json.dumps(event).encode("utf-8"))
                else:
                    await websocket.send_text(json.dumps(event))
                if runtime.scenario.disconnect_every > 0 and seq % runtime.scenario.disconnect_every == 0:
                    await websocket.close(code=1011, reason="mock disconnect")
                    break

    @router.websocket("/mock/ws/private")
    async def ws_private(websocket: WebSocket) -> None:
        await websocket.accept()
        authenticated = False
        while True:
            payload = await websocket.receive_json()
            if payload.get("op") == "auth" and payload.get("token"):
                authenticated = True
                await websocket.send_json({"type": "auth_ack", "ok": True})
                continue
            if payload.get("op") == "subscribe":
                await websocket.send_json({"type": "ack", "authorized": authenticated, "correlation_id": payload.get("correlation_id")})

    return router
