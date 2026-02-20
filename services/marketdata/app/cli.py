from __future__ import annotations

import argparse
import json
from datetime import UTC, datetime
from pathlib import Path

from services.marketdata.app.backfill import run_backfill_ohlcv
from services.marketdata.app.replay import run_replay
from services.marketdata.app.silver.iceberg_pipeline import run_diff, run_recompute
from services.marketdata.app.gold_materializer import materialize_gold
from services.marketdata.app.e2e_harness import main_cli as e2e_main_cli
import sqlite3


def _parse_ts(value: str) -> datetime:
    return datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(UTC)


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="marketdata", description="MarketData operations CLI")
    subparsers = parser.add_subparsers(dest="command", required=True)

    replay = subparsers.add_parser("replay", help="Replay raw bronze payloads into silver/event tables")
    replay.add_argument("--from-ts", required=True, help="RFC3339 start timestamp")
    replay.add_argument("--to-ts", required=True, help="RFC3339 end timestamp")
    replay.add_argument("--db-dsn", required=True, help="DB DSN (sqlite:///...)")
    replay.add_argument("--bronze-root", default="./data/bronze", help="Bronze filesystem root")
    replay.add_argument("--venue", default=None, help="Optional venue filter")
    replay.add_argument("--market", default=None, help="Optional market filter")
    replay.add_argument("--source-type", default=None, help="Optional source type filter")
    replay.add_argument("--parser-version", default="v0.1", help="Parser version to stamp")
    replay.add_argument("--dry-run", action="store_true", help="Count only, do not write")
    replay.add_argument("--write", action="store_true", help="Enable write mode (default is read-only)")

    backfill = subparsers.add_parser("backfill", help="Backfill marketdata")
    backfill_sub = backfill.add_subparsers(dest="backfill_command", required=True)
    ohlcv = backfill_sub.add_parser("ohlcv", help="Backfill OHLCV by REST paging")
    ohlcv.add_argument("--venue", required=True)
    ohlcv.add_argument("--market", required=True)
    ohlcv.add_argument("--tf", required=True)
    ohlcv.add_argument("--from", dest="from_ts", required=True)
    ohlcv.add_argument("--to", dest="to_ts", required=True)
    ohlcv.add_argument("--db-dsn", required=True, help="DB DSN (sqlite:///...)")
    ohlcv.add_argument("--symbol", default=None, help="Upstream symbol (default from env)")
    ohlcv.add_argument("--max-pages-per-run", type=int, default=5)
    ohlcv.add_argument("--state-path", default="services/marketdata/.state/ohlcv_backfill_cursor.json")

    silver = subparsers.add_parser("silver", help="Silver Iceberg tools")
    silver_sub = silver.add_subparsers(dest="silver_command", required=True)
    silver_backfill = silver_sub.add_parser("backfill", help="Recompute Silver Iceberg outputs from Bronze")
    silver_diff = silver_sub.add_parser("diff", help="Diff two Silver Iceberg outputs with deterministic hashes")

    recompute = subparsers.add_parser("silver-recompute", help="Deprecated alias for 'silver backfill'")
    e2e = subparsers.add_parser("dataplat-e2e", help="Run deterministic data platform e2e harness")
    e2e.add_argument("--seed", type=int, default=7)
    e2e.add_argument("--rate", type=int, default=50)
    e2e.add_argument("--duration", type=int, default=3)
    gold = subparsers.add_parser("gold-materialize", help="Materialize Gold marts from Silver tables")
    gold.add_argument("--db-dsn", required=True, help="DB DSN (sqlite:///...)")
    gold.add_argument("--watermark-ts", default=None, help="Optional watermark timestamp")
    for parser_obj in (recompute, silver_backfill):
        parser_obj.add_argument("--bronze-root", required=True, help="Bronze root directory")
        parser_obj.add_argument("--silver-root", required=True, help="Silver output root directory")
        parser_obj.add_argument("--from-ts", required=True, help="RFC3339 start timestamp")
        parser_obj.add_argument("--to-ts", required=True, help="RFC3339 end timestamp")
        parser_obj.add_argument("--venue", default=None, help="Venue filter")
        parser_obj.add_argument("--event-type", action="append", default=[], help="Repeatable event type filter")
        parser_obj.add_argument("--symbol", action="append", default=[], help="Repeatable symbol filter")
        parser_obj.add_argument("--batch-size", type=int, default=10000, help="Max rows per output parquet part")
        parser_obj.add_argument("--compact", action="store_true", help="Use larger output batches to mitigate small files")

    silver_diff.add_argument("--baseline-silver-root", required=True, help="Baseline silver root directory")
    silver_diff.add_argument("--candidate-silver-root", required=True, help="Candidate silver root directory")

    return parser


def main() -> int:
    parser = _build_parser()
    args = parser.parse_args()

    if args.command == "replay":
        summary = run_replay(
            from_ts=args.from_ts,
            to_ts=args.to_ts,
            db_dsn=args.db_dsn,
            bronze_root=args.bronze_root,
            venue=args.venue,
            market=args.market,
            source_type=args.source_type,
            dry_run=args.dry_run,
            write=args.write,
            parser_version=args.parser_version,
        )
        print(
            json.dumps(
                {
                    "from_ts": args.from_ts,
                    "to_ts": args.to_ts,
                    "venue": args.venue,
                    "market": args.market,
                    "source_type": args.source_type,
                    "dry_run": args.dry_run,
                    "write": args.write,
                    "parser_version": args.parser_version,
                    "read_count": summary.read_count,
                    "silver_count": summary.silver_count,
                    "events_count": summary.events_count,
                    "skipped_count": summary.skipped_count,
                },
                separators=(",", ":"),
                ensure_ascii=False,
            )
        )
        return 0

    if args.command == "backfill" and args.backfill_command == "ohlcv":
        summary = run_backfill_ohlcv(
            venue=args.venue,
            market=args.market,
            tf=args.tf,
            from_ts=args.from_ts,
            to_ts=args.to_ts,
            db_dsn=args.db_dsn,
            max_pages_per_run=args.max_pages_per_run,
            symbol=args.symbol,
            cursor_file=args.state_path,
        )
        print(json.dumps(summary.__dict__, separators=(",", ":"), ensure_ascii=False))
        return 0

    if args.command == "silver-recompute" or (args.command == "silver" and args.silver_command == "backfill"):
        report = run_recompute(
            bronze_root=Path(args.bronze_root),
            silver_root=Path(args.silver_root),
            start=_parse_ts(args.from_ts),
            end=_parse_ts(args.to_ts),
            venue=args.venue,
            symbols=tuple(args.symbol or []),
            event_types=tuple(args.event_type or []),
            batch_size=args.batch_size,
            compact=args.compact,
        )
        print(json.dumps(report.__dict__, separators=(",", ":"), ensure_ascii=False, sort_keys=True))
        return 0

    if args.command == "silver" and args.silver_command == "diff":
        report = run_diff(
            baseline_silver_root=Path(args.baseline_silver_root),
            candidate_silver_root=Path(args.candidate_silver_root),
        )
        print(json.dumps(report.__dict__, separators=(",", ":"), ensure_ascii=False, sort_keys=True))
        return 0

    if args.command == "dataplat-e2e":
        return e2e_main_cli(["--seed", str(args.seed), "--rate", str(args.rate), "--duration", str(args.duration)])

    if args.command == "gold-materialize":
        if not str(args.db_dsn).startswith("sqlite:///"):
            raise SystemExit("gold-materialize currently supports sqlite:/// only")
        conn = sqlite3.connect(str(args.db_dsn).removeprefix("sqlite:///"))
        result = materialize_gold(conn, watermark_ts=args.watermark_ts)
        print(json.dumps(result.__dict__, separators=(",", ":"), ensure_ascii=False, sort_keys=True))
        return 0

    parser.error(f"Unsupported command: {args.command}")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
