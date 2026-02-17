from __future__ import annotations

import hashlib
import json
import os
import sqlite3
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any, Callable

from services.marketdata.app.db.repository import MarketDataMetaRepository
from services.marketdata.app.db.schema import apply_migrations


@dataclass(frozen=True)
class BackfillSummary:
    venue: str
    market: str
    tf: str
    from_ts: str
    to_ts: str
    pages_processed: int
    candles_seen: int
    candles_written: int
    resumed_from_page: int
    next_page: int | None
    completed: bool


@dataclass(frozen=True)
class BackfillPage:
    candles: list[dict[str, Any]]
    has_more: bool


def _parse_rfc3339(ts: str) -> datetime:
    return datetime.fromisoformat(ts.replace("Z", "+00:00")).astimezone(UTC)


def _connect_repo(db_dsn: str) -> MarketDataMetaRepository:
    if not db_dsn.startswith("sqlite:///"):
        raise ValueError("Only sqlite:/// DSN is supported in v0.1")
    db_path = db_dsn.removeprefix("sqlite:///")
    if db_path != ":memory:":
        Path(db_path).parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(db_path)
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn)
    return MarketDataMetaRepository(conn)


def _cursor_file(override: str | None = None) -> Path:
    if override:
        return Path(override)
    raw = os.getenv("OHLCV_BACKFILL_CURSOR_FILE", "services/marketdata/.state/ohlcv_backfill_cursor.json")
    return Path(raw)


def _cursor_key(*, venue: str, market: str, tf: str, from_ts: str, to_ts: str) -> str:
    return f"{venue.lower()}:{market.lower()}:{tf}:{from_ts}:{to_ts}"


def _load_cursor(*, venue: str, market: str, tf: str, from_ts: str, to_ts: str, cursor_file: str | None = None) -> int:
    path = _cursor_file(cursor_file)
    if not path.exists():
        return 1
    payload = json.loads(path.read_text(encoding="utf-8") or "{}")
    value = payload.get(_cursor_key(venue=venue, market=market, tf=tf, from_ts=from_ts, to_ts=to_ts))
    try:
        return max(int(value), 1)
    except (TypeError, ValueError):
        return 1


def _save_cursor(*, venue: str, market: str, tf: str, from_ts: str, to_ts: str, page: int | None, cursor_file: str | None = None) -> None:
    path = _cursor_file(cursor_file)
    path.parent.mkdir(parents=True, exist_ok=True)
    state: dict[str, Any] = {}
    if path.exists():
        state = json.loads(path.read_text(encoding="utf-8") or "{}")
    key = _cursor_key(venue=venue, market=market, tf=tf, from_ts=from_ts, to_ts=to_ts)
    if page is None:
        state.pop(key, None)
    else:
        state[key] = int(page)
    path.write_text(json.dumps(state, separators=(",", ":"), ensure_ascii=False), encoding="utf-8")


def _stable_raw_msg_id(*, venue: str, market: str, tf: str, open_ts: str, close: float) -> str:
    source = f"{venue}|{market}|{tf}|{open_ts}|{close}"
    return hashlib.sha256(source.encode("utf-8")).hexdigest()[:26].upper()


def _parse_candle(item: dict[str, Any], *, tf: str) -> dict[str, Any] | None:
    open_ts = item.get("openTime") or item.get("open_ts") or item.get("timestamp")
    if not isinstance(open_ts, str) or not open_ts:
        return None

    try:
        return {
            "open_ts": open_ts,
            "open": float(item.get("open")),
            "high": float(item.get("high")),
            "low": float(item.get("low")),
            "close": float(item.get("close")),
            "volume": float(item.get("volume") or 0.0),
            "is_final": bool(item.get("is_final", True)),
            "timeframe": str(item.get("timeframe") or tf),
        }
    except (TypeError, ValueError):
        return None


def _default_fetch_page(*, symbol: str, tf: str, page: int) -> BackfillPage:
    interval = {"1m": "1min", "5m": "5min", "15m": "15min", "1h": "1hour", "1d": "1day"}.get(tf, tf)
    base = os.getenv("GMO_MARKETDATA_BASE_URL", "https://api.coin.z.com/public/v1").rstrip("/")
    query = urllib.parse.urlencode({"symbol": symbol, "interval": interval, "page": page})
    url = f"{base}/klines?{query}"
    with urllib.request.urlopen(url, timeout=float(os.getenv("OHLCV_BACKFILL_HTTP_TIMEOUT_SECONDS", "10"))) as resp:
        payload = json.loads(resp.read().decode("utf-8"))
    data = payload.get("data") if isinstance(payload, dict) else []
    candles = data if isinstance(data, list) else []
    has_more = len(candles) > 0
    return BackfillPage(candles=[c for c in candles if isinstance(c, dict)], has_more=has_more)


def run_backfill_ohlcv(
    *,
    venue: str,
    market: str,
    tf: str,
    from_ts: str,
    to_ts: str,
    db_dsn: str,
    max_pages_per_run: int = 5,
    symbol: str | None = None,
    fetch_page: Callable[[str, str, int], BackfillPage] | None = None,
    sleep_fn: Callable[[float], None] = time.sleep,
    cursor_file: str | None = None,
) -> BackfillSummary:
    repo = _connect_repo(db_dsn)
    from_dt = _parse_rfc3339(from_ts)
    to_dt = _parse_rfc3339(to_ts)
    symbol_value = symbol or os.getenv("MARKETDATA_SYMBOL", "BTC_JPY")
    page = _load_cursor(venue=venue, market=market, tf=tf, from_ts=from_ts, to_ts=to_ts, cursor_file=cursor_file)
    resumed_from_page = page
    pages_processed = 0
    candles_seen = 0
    candles_written = 0
    fetch = fetch_page or _default_fetch_page

    while pages_processed < max_pages_per_run:
        retry = 0
        while True:
            try:
                batch = fetch(symbol_value, tf, page)
                break
            except urllib.error.HTTPError as exc:
                if exc.code != 429 or retry >= 3:
                    raise
                sleep_fn(min(2**retry, 8))
                retry += 1
            except urllib.error.URLError:
                if retry >= 3:
                    raise
                sleep_fn(min(2**retry, 8))
                retry += 1

        pages_processed += 1
        for item in batch.candles:
            parsed = _parse_candle(item, tf=tf)
            if parsed is None:
                continue
            open_dt = _parse_rfc3339(parsed["open_ts"])
            if open_dt < from_dt or open_dt > to_dt:
                continue
            candles_seen += 1
            raw_msg_id = _stable_raw_msg_id(
                venue=venue,
                market=market,
                tf=tf,
                open_ts=parsed["open_ts"],
                close=parsed["close"],
            )
            before = repo._conn.total_changes
            repo.insert_md_ohlcv(
                raw_msg_id=raw_msg_id,
                venue_id=venue,
                market_id=market,
                instrument_id=symbol_value,
                timeframe=parsed["timeframe"],
                open_ts=parsed["open_ts"],
                open_price=parsed["open"],
                high=parsed["high"],
                low=parsed["low"],
                close=parsed["close"],
                volume=parsed["volume"],
                is_final=parsed["is_final"],
                extra_json={"source": "backfill", "symbol": symbol_value, "page": page},
            )
            if repo._conn.total_changes > before:
                candles_written += 1

        if not batch.has_more:
            _save_cursor(venue=venue, market=market, tf=tf, from_ts=from_ts, to_ts=to_ts, page=None, cursor_file=cursor_file)
            return BackfillSummary(
                venue=venue,
                market=market,
                tf=tf,
                from_ts=from_ts,
                to_ts=to_ts,
                pages_processed=pages_processed,
                candles_seen=candles_seen,
                candles_written=candles_written,
                resumed_from_page=resumed_from_page,
                next_page=None,
                completed=True,
            )

        page += 1

    _save_cursor(venue=venue, market=market, tf=tf, from_ts=from_ts, to_ts=to_ts, page=page, cursor_file=cursor_file)
    return BackfillSummary(
        venue=venue,
        market=market,
        tf=tf,
        from_ts=from_ts,
        to_ts=to_ts,
        pages_processed=pages_processed,
        candles_seen=candles_seen,
        candles_written=candles_written,
        resumed_from_page=resumed_from_page,
        next_page=page,
        completed=False,
    )



def run_ohlcv_backfill(**kwargs: Any) -> BackfillSummary:
    """Compatibility wrapper for CLI/tests."""
    return run_backfill_ohlcv(**kwargs)
