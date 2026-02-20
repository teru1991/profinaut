from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import os
import random
import sqlite3
import statistics
import tempfile
import time
from dataclasses import asdict, dataclass
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Any

from fastapi.testclient import TestClient

from services.marketdata.app import main
from services.marketdata.app.db.schema import apply_migrations
from services.marketdata.app.gold_cache import HotCache
from services.marketdata.app.gold_materializer import materialize_gold
from services.marketdata.app.routes import raw_ingest
from services.marketdata.app.settings import ServiceSettings
from services.marketdata.app.silver import normalizer
from services.marketdata.app.storage.object_store import ObjectStore


@dataclass(frozen=True)
class HarnessConfig:
    seed: int
    rate: int
    duration_s: int


@dataclass(frozen=True)
class HarnessSummary:
    seed: int
    generated: int
    accepted: int
    rejected: int
    dedupe_dropped: int
    bronze_lines: int
    silver_trades: int
    silver_bba: int
    silver_ohlcv: int
    silver_events: int
    anomalies: dict[str, int]
    restart_no_growth: bool
    throughput_eps: float
    bronze_p95_ms: float
    api_hit_p95_ms: float
    api_miss_p95_ms: float
    objectstore_degraded: bool
    objectstore_spool_bounded: bool
    clickhouse_degraded_safe: bool
    valkey_degraded_safe: bool
    api_unavailable_status: int
    queue_depth_stable: bool
    deterministic_digest: str
    pass_all: bool


class _FailingObjectStore(ObjectStore):
    def put_bytes(self, key: str, data: bytes, content_type: str = "application/octet-stream") -> None:
        raise RuntimeError("store down")

    def get_bytes(self, key: str) -> bytes:
        raise RuntimeError("store down")

    def list(self, prefix: str) -> list[str]:
        return []


async def _idle_poller() -> None:
    return None


def _iso(ts: datetime) -> str:
    return ts.isoformat().replace("+00:00", "Z")


def _build_event(ts_event: datetime, ts_recv: datetime, event_type: str, idx: int) -> dict[str, Any]:
    base = {
        "tenant_id": "tenant-e2e",
        "source_type": "WS_PUBLIC",
        "received_ts": _iso(ts_recv),
        "event_ts": _iso(ts_event),
        "venue_id": "gmo",
        "market_id": "spot",
        "instrument_id": "btc_jpy",
        "stream_name": event_type,
    }
    if event_type == "trade":
        return {
            **base,
            "source_event_id": f"tr-{idx}",
            "idempotency_key": f"gmo:spot:tr-{idx}",
            "payload_json": {"symbol": "BTC_JPY", "price": 100 + idx, "qty": 0.1, "side": "buy", "trade_id": idx},
        }
    if event_type == "ticker":
        return {
            **base,
            "source_event_id": f"tk-{idx}",
            "idempotency_key": f"gmo:spot:tk-{idx}",
            "payload_json": {"bid_px": 100 + idx, "ask_px": 101 + idx, "bid_qty": 1.0, "ask_qty": 2.0, "symbol": "BTC_JPY"},
        }
    return {
        **base,
        "source_event_id": f"ob-{idx}",
        "idempotency_key": f"gmo:spot:ob-{idx}",
        "seq": idx,
        "payload_json": {"symbol": "BTC_JPY", "bids": [[str(100 + idx), "1"]], "asks": [[str(101 + idx), "1"]], "sequence": idx},
    }


def _generate_stream(cfg: HarnessConfig) -> list[dict[str, Any]]:
    rng = random.Random(cfg.seed)
    start = datetime(2026, 1, 1, tzinfo=UTC)
    events: list[dict[str, Any]] = []
    total = cfg.rate * cfg.duration_s
    for i in range(total):
        recv = start + timedelta(milliseconds=i * (1000 // max(cfg.rate, 1)))
        kind = ("trade", "ticker", "orderbook_delta")[i % 3]
        evt = _build_event(recv, recv, kind, i)
        events.append(evt)

    events.append(dict(events[3]))  # duplicate idempotency

    oo = dict(events[7])
    oo["source_event_id"] = "tr-ooo"
    oo["idempotency_key"] = "gmo:spot:tr-ooo"
    oo["event_ts"] = _iso(start - timedelta(seconds=30))
    events.append(oo)

    gap = _build_event(start + timedelta(seconds=2), start + timedelta(seconds=2), "orderbook_delta", 999)
    gap["seq"] = 999
    gap["payload_json"]["sequence"] = 999
    events.append(gap)

    late = _build_event(start - timedelta(minutes=10), start + timedelta(minutes=1), "trade", 1000)
    late["source_event_id"] = "tr-late"
    late["idempotency_key"] = "gmo:spot:tr-late"
    events.append(late)

    malformed = {
        "tenant_id": "tenant-e2e",
        "source_type": "WS_PUBLIC",
        "received_ts": _iso(start + timedelta(seconds=3)),
        "event_ts": _iso(start + timedelta(seconds=3)),
        "payload_json": "bad",
    }
    events.append(malformed)

    secret = _build_event(start + timedelta(seconds=4), start + timedelta(seconds=4), "trade", 1001)
    secret["source_event_id"] = "tr-secret"
    secret["idempotency_key"] = "gmo:spot:tr-secret"
    secret["payload_json"]["api_key"] = "secret-token"
    events.append(secret)

    rng.shuffle(events)
    return events


def _summary_digest(summary: dict[str, Any]) -> str:
    stable = {
        "seed": summary["seed"],
        "generated": summary["generated"],
        "accepted": summary["accepted"],
        "rejected": summary["rejected"],
        "dedupe_dropped": summary["dedupe_dropped"],
        "bronze_lines": summary["bronze_lines"],
        "silver_trades": summary["silver_trades"],
        "silver_bba": summary["silver_bba"],
        "silver_ohlcv": summary["silver_ohlcv"],
        "silver_events": summary["silver_events"],
        "anomalies": summary["anomalies"],
        "restart_no_growth": summary["restart_no_growth"],
        "objectstore_degraded": summary["objectstore_degraded"],
        "objectstore_spool_bounded": summary["objectstore_spool_bounded"],
        "clickhouse_degraded_safe": summary["clickhouse_degraded_safe"],
        "valkey_degraded_safe": summary["valkey_degraded_safe"],
        "api_unavailable_status": summary["api_unavailable_status"],
        "queue_depth_stable": summary["queue_depth_stable"],
    }
    blob = json.dumps(stable, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    return hashlib.sha256(blob.encode("utf-8")).hexdigest()


def _count_bronze_lines(bronze_root: Path) -> int:
    total = 0
    for path in bronze_root.rglob("*.jsonl.gz"):
        for line in gzip.decompress(path.read_bytes()).decode("utf-8").splitlines():
            if line.strip():
                total += 1
                parsed = json.loads(line)
                blob = json.dumps(parsed)
                assert "api_key" not in blob.lower()
    return total


def _p95(values: list[float]) -> float:
    if not values:
        return 0.0
    if len(values) == 1:
        return values[0]
    return float(statistics.quantiles(values, n=100, method="inclusive")[94])


def run_harness(cfg: HarnessConfig) -> HarnessSummary:
    with tempfile.TemporaryDirectory(prefix="dataplat-e2e-") as td:
        root = Path(td)
        db_file = root / "md.sqlite3"
        bronze_root = root / "bronze"
        idem_file = root / "idem.sqlite3"
        os.environ["OBJECT_STORE_BACKEND"] = "fs"
        os.environ["DB_DSN"] = f"sqlite:///{db_file}"
        os.environ["BRONZE_FS_ROOT"] = str(bronze_root)
        os.environ["BRONZE_IDEMPOTENCY_DB"] = str(idem_file)
        os.environ["SILVER_ENABLED"] = "1"

        raw_ingest._BRONZE_WRITER = None
        raw_ingest._BRONZE_WRITER_ROOT = None
        normalizer._ORDERBOOK_ENGINES.clear()
        normalizer._ORDERBOOK_LAST_SEQ.clear()
        normalizer._ORDERBOOK_REQUIRE_SNAPSHOT.clear()

        conn = sqlite3.connect(db_file)
        apply_migrations(conn)
        conn.close()

        settings = ServiceSettings(db_dsn=f"sqlite:///{db_file}", object_store_backend="fs", ingest_raw_enabled=True, silver_enabled=True, degraded=False, degraded_reasons=[])
        events = _generate_stream(cfg)
        latencies: list[float] = []
        accepted = rejected = dedupe = 0
        start = time.perf_counter()
        for event in events:
            t0 = time.perf_counter()
            code, body = raw_ingest.ingest_raw_envelope(event, settings=settings)
            latencies.append((time.perf_counter() - t0) * 1000)
            if code == 200:
                accepted += 1
                if body.get("object_key") == "dedupe://dropped":
                    dedupe += 1
            else:
                rejected += 1
        elapsed = max(time.perf_counter() - start, 1e-6)

        conn = sqlite3.connect(db_file)
        materialize_gold(conn)

        anomalies: dict[str, int] = {}
        for reason, count in conn.execute("SELECT json_extract(extra_json, '$.reason'), COUNT(*) FROM md_events_json GROUP BY 1").fetchall():
            anomalies[str(reason)] = int(count)

        before_restart = _count_bronze_lines(bronze_root)

        writer = raw_ingest._get_bronze_writer()
        writer.close()
        raw_ingest._BRONZE_WRITER = None
        raw_ingest._BRONZE_WRITER_ROOT = None
        for event in events[:10]:
            raw_ingest.ingest_raw_envelope(event, settings=settings)
        after_restart = _count_bronze_lines(bronze_root)

        writer = raw_ingest._get_bronze_writer()
        writer._store = _FailingObjectStore()  # failure simulation
        raw_ingest.ingest_raw_envelope(_build_event(datetime.now(UTC), datetime.now(UTC), "trade", 5000), settings=settings)
        degraded_health = writer.health()
        degraded = bool(degraded_health.get("degraded"))
        spool_bytes = int(degraded_health.get("spool_bytes", 0))
        spool_bounded = spool_bytes <= (64 * 1024 * 1024)
        queue_depth_start = int(degraded_health.get("queue_depth", 0))

        main._gold_cache = HotCache(default_ttl_seconds=5.0, jitter_seconds=0.0)
        main._valkey_cache = main.ValkeyHotCache(main._gold_cache)
        main._poller.run_forever = _idle_poller  # type: ignore[method-assign]

        class _DownClickhouse:
            def query_ticker_latest(self, **_: Any) -> dict[str, Any] | None:
                raise RuntimeError("clickhouse down")

            def query_bba_latest(self, **_: Any) -> dict[str, Any] | None:
                raise RuntimeError("clickhouse down")

            def query_ohlcv_range(self, **_: Any) -> list[dict[str, Any]]:
                raise RuntimeError("clickhouse down")

        class _DownValkey:
            def get_or_load(self, key, loader, *, ttl_seconds: float | None = None):
                raise RuntimeError("valkey down")

            def invalidate(self, key: str) -> None:
                return None

            def stats(self) -> dict[str, int]:
                return {"hit": 0, "miss": 0}

        with TestClient(main.app) as client:
            t_hit: list[float] = []
            for _ in range(6):
                s = time.perf_counter()
                r = client.get("/markets/ticker/latest?venue=gmo&symbol=BTC_JPY")
                _ = r.status_code
                t_hit.append((time.perf_counter() - s) * 1000)

            t_miss: list[float] = []
            for _ in range(3):
                s = time.perf_counter()
                r = client.get("/markets/ticker/latest?venue=gmo&symbol=ETH_JPY")
                _ = r.status_code
                t_miss.append((time.perf_counter() - s) * 1000)

            main._gold_cache.invalidate("ticker_latest:gmo:BTC_JPY")
            os.environ["DB_DSN"] = "postgres://down"
            unavailable_status = client.get("/markets/ticker/latest?venue=gmo&symbol=BTC_JPY").status_code

            os.environ["DB_DSN"] = f"sqlite:///{db_file}"
            original_ch = main._clickhouse_store
            main._clickhouse_store = _DownClickhouse()
            clickhouse_fallback_status = client.get("/markets/ticker/latest?venue=gmo&symbol=btc_jpy").status_code
            main._clickhouse_store = original_ch

            original_valkey = main._valkey_cache
            original_ch_for_valkey = main._clickhouse_store
            main._clickhouse_store = _DownClickhouse()
            main._valkey_cache = _DownValkey()
            valkey_fallback_status = client.get("/markets/ticker/latest?venue=gmo&symbol=btc_jpy").status_code
            main._valkey_cache = original_valkey
            main._clickhouse_store = original_ch_for_valkey

            queue_depth_end = int(writer.health().get("queue_depth", 0))

        bronze_lines = _count_bronze_lines(bronze_root)
        silver_trades = conn.execute("SELECT COUNT(*) FROM md_trades").fetchone()[0]
        silver_bba = conn.execute("SELECT COUNT(*) FROM md_best_bid_ask").fetchone()[0]
        silver_ohlcv = conn.execute("SELECT COUNT(*) FROM md_ohlcv").fetchone()[0]
        silver_events = conn.execute("SELECT COUNT(*) FROM md_events_json").fetchone()[0]
        conn.close()

        queue_depth_stable = max(queue_depth_start, queue_depth_end) <= 2
        clickhouse_degraded_safe = clickhouse_fallback_status == 200
        valkey_degraded_safe = valkey_fallback_status == 200
        restart_no_growth = before_restart == after_restart
        pass_all = all([
            bronze_lines >= accepted - dedupe,
            restart_no_growth,
            degraded,
            spool_bounded,
            clickhouse_degraded_safe,
            valkey_degraded_safe,
            queue_depth_stable,
            unavailable_status == 503,
            rejected >= 2,
        ])

        summary = HarnessSummary(
            seed=cfg.seed,
            generated=len(events),
            accepted=accepted,
            rejected=rejected,
            dedupe_dropped=dedupe,
            bronze_lines=bronze_lines,
            silver_trades=silver_trades,
            silver_bba=silver_bba,
            silver_ohlcv=silver_ohlcv,
            silver_events=silver_events,
            anomalies=anomalies,
            restart_no_growth=restart_no_growth,
            throughput_eps=accepted / elapsed,
            bronze_p95_ms=_p95(latencies),
            api_hit_p95_ms=_p95(t_hit),
            api_miss_p95_ms=_p95(t_miss),
            objectstore_degraded=degraded,
            objectstore_spool_bounded=spool_bounded,
            clickhouse_degraded_safe=clickhouse_degraded_safe,
            valkey_degraded_safe=valkey_degraded_safe,
            api_unavailable_status=unavailable_status,
            queue_depth_stable=queue_depth_stable,
            deterministic_digest="",
            pass_all=pass_all,
        )
        payload = asdict(summary)
        payload["deterministic_digest"] = _summary_digest(payload)
        return HarnessSummary(**payload)


def main_cli(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="marketdata dataplat-e2e")
    parser.add_argument("--seed", type=int, default=7)
    parser.add_argument("--rate", type=int, default=50)
    parser.add_argument("--duration", type=int, default=3)
    args = parser.parse_args(argv)

    result = run_harness(HarnessConfig(seed=args.seed, rate=args.rate, duration_s=args.duration))
    print(json.dumps(asdict(result), sort_keys=True))
    print("PASS" if result.pass_all else "FAIL")
    return 0 if result.pass_all else 1


if __name__ == "__main__":
    raise SystemExit(main_cli())
