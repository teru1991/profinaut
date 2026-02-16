from __future__ import annotations

import asyncio
import contextlib
import json
import logging
import os
import sqlite3
import uuid
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any, AsyncContextManager, Callable, Protocol

from libs.observability import audit_event
from services.marketdata.app.db.repository import MarketDataMetaRepository
from services.marketdata.app.db.schema import apply_migrations
from services.marketdata.app.routes.raw_ingest import ingest_raw_envelope
from services.marketdata.app.session_recorder import SessionRecorder

logger = logging.getLogger("marketdata")


class WsConnection(Protocol):
    async def send(self, message: str) -> None: ...

    def __aiter__(self) -> Any: ...


ConnectFn = Callable[[str], AsyncContextManager[WsConnection]]


@dataclass(frozen=True)
class GmoWsConfig:
    enabled: bool = os.getenv("GMO_WS_ENABLED", "0").strip() == "1"
    ws_url: str = os.getenv("GMO_WS_URL", "wss://api.coin.z.com/ws/public/v1")
    symbol: str = os.getenv("MARKETDATA_SYMBOL", "BTC_JPY")
    market_id: str = os.getenv("MARKETDATA_MARKET_ID", "spot")
    channels: tuple[str, ...] = tuple(
        value.strip() for value in os.getenv("GMO_WS_CHANNELS", "ticker,trades,orderbooks").split(",") if value.strip()
    )
    backoff_initial_seconds: float = float(os.getenv("GMO_WS_BACKOFF_INITIAL_SECONDS", "1"))
    backoff_max_seconds: float = float(os.getenv("GMO_WS_BACKOFF_MAX_SECONDS", "30"))
    reconnect_seconds: float = float(os.getenv("GMO_WS_RECONNECT_SECONDS", "1"))
    tenant_id: str = os.getenv("GMO_WS_TENANT_ID", "marketdata")


class GmoPublicWsConnector:
    def __init__(self, config: GmoWsConfig, *, connect_fn: ConnectFn | None = None):
        self._config = config
        self._connect_fn = connect_fn
        self._stop = asyncio.Event()
        self._backoff = config.backoff_initial_seconds
        self._degraded_reason: str | None = None
        self._session_recorder = self._build_session_recorder()

    @staticmethod
    def _build_session_recorder() -> SessionRecorder | None:
        db_dsn = os.getenv("DB_DSN")
        if not db_dsn or not db_dsn.startswith("sqlite:///"):
            return None
        db_path = db_dsn.removeprefix("sqlite:///")
        if db_path != ":memory:":
            Path(db_path).parent.mkdir(parents=True, exist_ok=True)
        conn = sqlite3.connect(db_path)
        conn.execute("PRAGMA foreign_keys = ON")
        apply_migrations(conn)
        return SessionRecorder(MarketDataMetaRepository(conn))

    @property
    def enabled(self) -> bool:
        return self._config.enabled

    @property
    def degraded_reason(self) -> str | None:
        return self._degraded_reason

    async def stop(self) -> None:
        self._stop.set()

    def _set_degraded_reason(self, reason: str | None) -> None:
        self._degraded_reason = reason

    async def _connect(self) -> AsyncContextManager[WsConnection]:
        if self._connect_fn is not None:
            return self._connect_fn(self._config.ws_url)

        from websockets.asyncio.client import connect  # type: ignore[import-not-found]

        return connect(self._config.ws_url)

    @staticmethod
    def _extract_event_ts(payload: dict[str, Any]) -> str | None:
        for key in ("timestamp", "ts", "time", "eventTime"):
            value = payload.get(key)
            if isinstance(value, str) and value:
                return value
        return None

    async def _ingest_message(self, stream_name: str, payload: dict[str, Any]) -> None:
        received_ts = datetime.now(UTC).isoformat().replace("+00:00", "Z")
        status, body = await asyncio.to_thread(
            ingest_raw_envelope,
            {
                "tenant_id": self._config.tenant_id,
                "source_type": "WS_PUBLIC",
                "received_ts": received_ts,
                "payload_json": payload,
                "venue_id": "gmo",
                "market_id": self._config.market_id,
                "stream_name": stream_name,
                "endpoint": self._config.ws_url,
                "event_ts": self._extract_event_ts(payload),
                "source_msg_key": None,
                "seq": payload.get("sequence") or payload.get("seq"),
            },
        )

        if self._session_recorder is not None:
            self._session_recorder.record_received_message(
                dup_suspect=bool(body.get("dup_suspect")) if isinstance(body, dict) else False,
                gap_suspect=False,
            )

        if status == 200:
            self._set_degraded_reason(None)
            self._backoff = self._config.backoff_initial_seconds
            return

        self._set_degraded_reason("RAW_INGEST_FAILED")
        logger.warning("gmo_ws_ingest_failed status=%s body=%s", status, body)

    async def _run_session(self) -> None:
        session_id = f"gmo-ws-{uuid.uuid4()}"
        started_at = datetime.now(UTC).isoformat().replace("+00:00", "Z")
        if self._session_recorder is not None:
            self._session_recorder.start_session(
                session_id=session_id,
                venue_id="gmo",
                market_id=self._config.market_id,
                started_at=started_at,
            )

        close_reason: str | None = None
        try:
            async with await self._connect() as ws:
                audit_event(service="marketdata", event="gmo_ws_connected", endpoint=self._config.ws_url)

                for channel in self._config.channels:
                    await ws.send(json.dumps({"command": "subscribe", "channel": channel, "symbol": self._config.symbol}))
                    if self._session_recorder is not None:
                        self._session_recorder.record_subscription(
                            stream_name=channel,
                            meta_json={"symbol": self._config.symbol, "channel": channel, "market_id": self._config.market_id},
                        )

                async for message in ws:
                    if self._stop.is_set():
                        close_reason = "stopped"
                        break
                    if not isinstance(message, str):
                        continue

                    payload = json.loads(message)
                    if not isinstance(payload, dict):
                        continue

                    stream_name = str(payload.get("channel") or payload.get("command") or "unknown")
                    await self._ingest_message(stream_name, payload)
        finally:
            if self._session_recorder is not None:
                self._session_recorder.end_session(close_reason=close_reason)

    async def run_forever(self) -> None:
        if not self._config.enabled:
            return

        while not self._stop.is_set():
            try:
                await self._run_session()
                self._set_degraded_reason(None)
                if not self._stop.is_set():
                    await asyncio.sleep(self._config.reconnect_seconds)
            except asyncio.CancelledError:
                raise
            except Exception as exc:
                self._set_degraded_reason("UPSTREAM_UNREACHABLE")
                audit_event(
                    service="marketdata",
                    event="gmo_ws_error",
                    degraded_reason=self._degraded_reason,
                    error=str(exc),
                    backoff_seconds=self._backoff,
                )
                await asyncio.sleep(self._backoff)
                self._backoff = min(self._backoff * 2, self._config.backoff_max_seconds)

        with contextlib.suppress(Exception):
            audit_event(service="marketdata", event="gmo_ws_stopped")
