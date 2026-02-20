from __future__ import annotations

import sqlite3

from services.marketdata.app.db.schema import apply_migrations
from services.marketdata.app.gold_materializer import materialize_gold


def test_ohlcv_1m_materialization_keeps_values_and_lineage() -> None:
    conn = sqlite3.connect(":memory:")
    apply_migrations(conn)
    conn.execute(
        """
        INSERT INTO md_ohlcv(raw_msg_id, venue_id, market_id, instrument_id, timeframe, open_ts, open, high, low, close, volume, is_final, extra_json)
        VALUES ('raw-1','gmo','spot','btc_jpy','1m','2026-02-16T00:00:42Z',10,12,9,11,3.5,1,'{}')
        """
    )
    conn.commit()

    result = materialize_gold(conn)
    row = conn.execute("SELECT ts_bucket, open, high, low, close, volume, raw_refs FROM gold_ohlcv_1m").fetchone()

    assert result.ohlcv_rows == 1
    assert row is not None
    assert row[0] == "2026-02-16T00:00:00Z"
    assert row[1:6] == (10.0, 12.0, 9.0, 11.0, 3.5)
    assert row[6] == '["raw-1"]'
