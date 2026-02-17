from __future__ import annotations

import argparse
import json

from services.marketdata.app.backfill import run_backfill_ohlcv
from services.marketdata.app.replay import run_replay


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

    parser.error(f"Unsupported command: {args.command}")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
