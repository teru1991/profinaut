from __future__ import annotations

from dataclasses import dataclass
from datetime import UTC, datetime

from services.marketdata.app.db.repository import MarketDataMetaRepository


@dataclass
class _SessionState:
    session_id: str
    venue_id: str | None
    market_id: str | None
    started_at: str
    recv_count: int = 0
    dup_suspect_count: int = 0
    gap_suspect_count: int = 0
    lag_min_ms: float | None = None
    lag_max_ms: float | None = None
    lag_last_ms: float | None = None
    lag_count: int = 0


class SessionRecorder:
    def __init__(self, repo: MarketDataMetaRepository):
        self._repo = repo
        self._session: _SessionState | None = None
        self._subscriptions: dict[str, str] = {}

    def _now_ts(self) -> str:
        return datetime.now(UTC).isoformat().replace("+00:00", "Z")

    def start_session(
        self,
        *,
        session_id: str,
        venue_id: str | None,
        market_id: str | None,
        started_at: str | None = None,
    ) -> None:
        start = started_at or self._now_ts()
        self._repo.insert_ws_session(
            session_id=session_id,
            venue_id=venue_id,
            market_id=market_id,
            started_at=start,
            ended_at=None,
            close_reason=None,
            recv_count=0,
            dup_suspect_count=0,
            gap_suspect_count=0,
            lag_stats_json={},
        )
        self._session = _SessionState(
            session_id=session_id,
            venue_id=venue_id,
            market_id=market_id,
            started_at=start,
        )

    def record_subscription(
        self,
        *,
        stream_name: str,
        meta_json: dict[str, str | int | float | bool | None],
        subscribed_at: str | None = None,
    ) -> None:
        session = self._require_active_session()
        subscribed = subscribed_at or self._now_ts()
        self._repo.insert_ws_subscription(
            session_id=session.session_id,
            stream_name=stream_name,
            subscribed_at=subscribed,
            unsubscribed_at=None,
            meta_json=dict(meta_json),
        )
        self._subscriptions[stream_name] = subscribed

    def record_unsubscribe(self, *, stream_name: str, unsubscribed_at: str | None = None) -> None:
        session = self._require_active_session()
        subscribed = self._subscriptions.pop(stream_name, None)
        if subscribed is None:
            return
        self._repo.update_ws_subscription_end(
            session_id=session.session_id,
            stream_name=stream_name,
            subscribed_at=subscribed,
            unsubscribed_at=unsubscribed_at or self._now_ts(),
        )

    def record_received_message(
        self,
        *,
        dup_suspect: bool = False,
        gap_suspect: bool = False,
        lag_ms: float | None = None,
    ) -> None:
        session = self._require_active_session()
        session.recv_count += 1
        if dup_suspect:
            session.dup_suspect_count += 1
        if gap_suspect:
            session.gap_suspect_count += 1

        if lag_ms is not None:
            session.lag_count += 1
            session.lag_last_ms = lag_ms
            session.lag_min_ms = lag_ms if session.lag_min_ms is None else min(session.lag_min_ms, lag_ms)
            session.lag_max_ms = lag_ms if session.lag_max_ms is None else max(session.lag_max_ms, lag_ms)

    def end_session(self, *, close_reason: str | None, ended_at: str | None = None) -> None:
        session = self._require_active_session()
        end = ended_at or self._now_ts()

        for stream_name in list(self._subscriptions):
            self.record_unsubscribe(stream_name=stream_name, unsubscribed_at=end)

        self._repo.update_ws_session_end(
            session_id=session.session_id,
            ended_at=end,
            close_reason=close_reason,
            recv_count=session.recv_count,
            dup_suspect_count=session.dup_suspect_count,
            gap_suspect_count=session.gap_suspect_count,
            lag_stats_json={
                "count": session.lag_count,
                "min_ms": session.lag_min_ms,
                "max_ms": session.lag_max_ms,
                "last_ms": session.lag_last_ms,
            },
        )
        self._session = None

    def _require_active_session(self) -> _SessionState:
        if self._session is None:
            raise RuntimeError("Session has not been started")
        return self._session
