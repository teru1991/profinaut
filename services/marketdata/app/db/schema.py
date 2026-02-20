from __future__ import annotations

import sqlite3

MIGRATIONS: tuple[tuple[str, tuple[str, ...]], ...] = (
    (
        "0001_marketdata_meta_tables",
        (
            """
            CREATE TABLE IF NOT EXISTS raw_ingest_meta (
                raw_msg_id TEXT PRIMARY KEY,
                tenant_id TEXT NOT NULL,
                source_type TEXT,
                venue_id TEXT,
                market_id TEXT,
                stream_name TEXT,
                endpoint TEXT,
                event_ts TEXT,
                received_ts TEXT NOT NULL,
                seq TEXT,
                source_msg_key TEXT,
                payload_hash TEXT,
                payload_size INTEGER,
                object_key TEXT,
                quality_json TEXT,
                parser_version TEXT
            )
            """,
            "CREATE INDEX IF NOT EXISTS idx_raw_ingest_meta_received_ts ON raw_ingest_meta(received_ts)",
            "CREATE INDEX IF NOT EXISTS idx_raw_ingest_meta_venue_market_received_ts ON raw_ingest_meta(venue_id, market_id, received_ts)",
            "CREATE INDEX IF NOT EXISTS idx_raw_ingest_meta_payload_hash ON raw_ingest_meta(payload_hash)",
            """
            CREATE TABLE IF NOT EXISTS ws_sessions (
                session_id TEXT PRIMARY KEY,
                venue_id TEXT,
                market_id TEXT,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                close_reason TEXT,
                recv_count INTEGER NOT NULL DEFAULT 0,
                dup_suspect_count INTEGER NOT NULL DEFAULT 0,
                gap_suspect_count INTEGER NOT NULL DEFAULT 0,
                lag_stats_json TEXT
            )
            """,
            """
            CREATE TABLE IF NOT EXISTS ws_subscriptions (
                session_id TEXT NOT NULL,
                stream_name TEXT NOT NULL,
                subscribed_at TEXT NOT NULL,
                unsubscribed_at TEXT,
                meta_json TEXT,
                PRIMARY KEY (session_id, stream_name, subscribed_at),
                FOREIGN KEY (session_id) REFERENCES ws_sessions(session_id)
            )
            """,
            "CREATE INDEX IF NOT EXISTS idx_ws_subscriptions_session_id ON ws_subscriptions(session_id)",
        ),
    ),
    (
        "0002_marketdata_silver_tables",
        (
            """
            CREATE TABLE IF NOT EXISTS md_trades (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                raw_msg_id TEXT NOT NULL,
                venue_id TEXT,
                market_id TEXT,
                instrument_id TEXT,
                source_msg_key TEXT,
                price REAL NOT NULL,
                qty REAL NOT NULL,
                side TEXT NOT NULL,
                occurred_at TEXT NOT NULL,
                received_ts TEXT NOT NULL,
                extra_json TEXT
            )
            """,
            "CREATE UNIQUE INDEX IF NOT EXISTS uq_md_trades_src_msg_key ON md_trades(venue_id, market_id, instrument_id, source_msg_key) WHERE source_msg_key IS NOT NULL",
            """
            CREATE TABLE IF NOT EXISTS md_ohlcv (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                raw_msg_id TEXT NOT NULL,
                venue_id TEXT,
                market_id TEXT,
                instrument_id TEXT,
                timeframe TEXT NOT NULL,
                open_ts TEXT NOT NULL,
                open REAL NOT NULL,
                high REAL NOT NULL,
                low REAL NOT NULL,
                close REAL NOT NULL,
                volume REAL NOT NULL,
                is_final INTEGER NOT NULL,
                extra_json TEXT
            )
            """,
            "CREATE UNIQUE INDEX IF NOT EXISTS uq_md_ohlcv_key ON md_ohlcv(venue_id, market_id, instrument_id, timeframe, open_ts)",
            """
            CREATE TABLE IF NOT EXISTS md_best_bid_ask (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                raw_msg_id TEXT NOT NULL,
                venue_id TEXT,
                market_id TEXT,
                instrument_id TEXT,
                bid_px REAL NOT NULL,
                bid_qty REAL NOT NULL,
                ask_px REAL NOT NULL,
                ask_qty REAL NOT NULL,
                event_ts TEXT NOT NULL,
                received_ts TEXT NOT NULL,
                extra_json TEXT
            )
            """,
            """
            CREATE TABLE IF NOT EXISTS md_events_json (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                raw_msg_id TEXT NOT NULL,
                tenant_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                event_ts TEXT,
                received_ts TEXT NOT NULL,
                payload_jsonb TEXT NOT NULL,
                payload_schema_ref TEXT NOT NULL,
                parser_version TEXT NOT NULL,
                extra_json TEXT
            )
            """,
        ),
    ),
    (
        "0003_marketdata_orderbook_state",
        (
            """
            CREATE TABLE IF NOT EXISTS md_orderbook_state (
                venue_id TEXT NOT NULL,
                market_id TEXT NOT NULL,
                bid_px REAL,
                bid_qty REAL,
                ask_px REAL,
                ask_qty REAL,
                as_of TEXT,
                last_update_ts TEXT NOT NULL,
                last_seq TEXT,
                degraded INTEGER NOT NULL DEFAULT 0,
                reason TEXT,
                PRIMARY KEY (venue_id, market_id)
            )
            """,
        ),
    ),
    (
        "0004_marketdata_gold_tables",
        (
            """
            CREATE TABLE IF NOT EXISTS gold_ticker_latest (
                venue_id TEXT NOT NULL,
                market_id TEXT NOT NULL,
                instrument_id TEXT NOT NULL,
                price REAL NOT NULL,
                bid_px REAL,
                ask_px REAL,
                bid_qty REAL,
                ask_qty REAL,
                ts_event TEXT,
                ts_recv TEXT NOT NULL,
                dt TEXT NOT NULL,
                raw_refs TEXT NOT NULL,
                PRIMARY KEY (venue_id, market_id, instrument_id)
            )
            """,
            """
            CREATE TABLE IF NOT EXISTS gold_best_bid_ask (
                venue_id TEXT NOT NULL,
                market_id TEXT NOT NULL,
                instrument_id TEXT NOT NULL,
                bid_px REAL NOT NULL,
                bid_qty REAL NOT NULL,
                ask_px REAL NOT NULL,
                ask_qty REAL NOT NULL,
                ts_event TEXT,
                ts_recv TEXT NOT NULL,
                dt TEXT NOT NULL,
                raw_refs TEXT NOT NULL,
                PRIMARY KEY (venue_id, market_id, instrument_id)
            )
            """,
            """
            CREATE TABLE IF NOT EXISTS gold_ohlcv_1m (
                venue_id TEXT NOT NULL,
                market_id TEXT NOT NULL,
                instrument_id TEXT NOT NULL,
                ts_bucket TEXT NOT NULL,
                open REAL NOT NULL,
                high REAL NOT NULL,
                low REAL NOT NULL,
                close REAL NOT NULL,
                volume REAL NOT NULL,
                is_final INTEGER NOT NULL,
                dt TEXT NOT NULL,
                raw_refs TEXT NOT NULL,
                PRIMARY KEY (venue_id, market_id, instrument_id, ts_bucket)
            )
            """,
            "CREATE INDEX IF NOT EXISTS idx_gold_ohlcv_1m_lookup ON gold_ohlcv_1m(venue_id, market_id, instrument_id, ts_bucket)",
        ),
    ),
    (
        "0005_marketdata_ops_tables",
        (
            """
            CREATE TABLE IF NOT EXISTS accounts (
                account_id TEXT PRIMARY KEY,
                label TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            """,
            """
            CREATE TABLE IF NOT EXISTS venues (
                venue_id TEXT PRIMARY KEY,
                venue_name TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            """,
            """
            CREATE TABLE IF NOT EXISTS account_venue_binding (
                account_id TEXT NOT NULL,
                venue_id TEXT NOT NULL,
                external_account_ref TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (account_id, venue_id),
                FOREIGN KEY (account_id) REFERENCES accounts(account_id),
                FOREIGN KEY (venue_id) REFERENCES venues(venue_id)
            )
            """,
            """
            CREATE TABLE IF NOT EXISTS balance_snapshots (
                account_id TEXT NOT NULL,
                venue_id TEXT NOT NULL,
                asset TEXT NOT NULL,
                ts_snapshot TEXT NOT NULL,
                free_qty REAL NOT NULL,
                locked_qty REAL NOT NULL,
                PRIMARY KEY (account_id, venue_id, asset, ts_snapshot),
                FOREIGN KEY (account_id, venue_id) REFERENCES account_venue_binding(account_id, venue_id)
            )
            """,
            """
            CREATE TABLE IF NOT EXISTS position_snapshots (
                account_id TEXT NOT NULL,
                venue_id TEXT NOT NULL,
                symbol TEXT NOT NULL,
                ts_snapshot TEXT NOT NULL,
                qty REAL NOT NULL,
                avg_price REAL,
                PRIMARY KEY (account_id, venue_id, symbol, ts_snapshot),
                FOREIGN KEY (account_id, venue_id) REFERENCES account_venue_binding(account_id, venue_id)
            )
            """,
        ),
    ),

    (
        "0006_marketdata_ops_orders_fills",
        (
            """
            CREATE TABLE IF NOT EXISTS orders (
                order_id TEXT PRIMARY KEY,
                account_id TEXT,
                venue_id TEXT,
                symbol TEXT,
                side TEXT,
                order_type TEXT,
                status TEXT,
                ts_event TEXT
            )
            """,
            """
            CREATE TABLE IF NOT EXISTS fills (
                fill_id TEXT PRIMARY KEY,
                order_id TEXT,
                account_id TEXT,
                venue_id TEXT,
                symbol TEXT,
                price REAL,
                qty REAL,
                ts_event TEXT
            )
            """,
        ),
    ),
)


def apply_migrations(conn: sqlite3.Connection) -> None:
    conn.execute("PRAGMA foreign_keys = ON")
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS schema_migrations (
            migration_id TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        """
    )

    for migration_id, statements in MIGRATIONS:
        row = conn.execute(
            "SELECT migration_id FROM schema_migrations WHERE migration_id = ?",
            (migration_id,),
        ).fetchone()
        if row is not None:
            continue

        for ddl in statements:
            conn.execute(ddl)

        conn.execute("INSERT INTO schema_migrations (migration_id) VALUES (?)", (migration_id,))

    conn.commit()
