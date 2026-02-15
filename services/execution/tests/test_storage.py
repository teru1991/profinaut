from datetime import datetime, UTC

from app.schemas import OrderIntent
from app.storage import OrderStorage


def test_idempotency_mapping_persists_across_storage_restart(tmp_path):
    db_path = tmp_path / "execution-idempotency.sqlite"

    storage1 = OrderStorage(db_path=str(db_path))
    intent = OrderIntent(
        idempotency_key="persist-key-1",
        exchange="binance",
        symbol="BTC/USDT",
        side="BUY",
        qty=0.01,
        type="MARKET",
        client_ts_utc=datetime.now(UTC),
    )

    created = storage1.create_order(intent)
    assert created is not None

    # Simulate process restart with a fresh storage instance on same sqlite path.
    storage2 = OrderStorage(db_path=str(db_path))
    duplicate = storage2.create_order(intent)
    assert duplicate is None
