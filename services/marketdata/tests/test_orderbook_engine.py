from services.marketdata.app.silver.orderbook import OrderbookEngine


def test_snapshot_then_delta_updates_bbo() -> None:
    engine = OrderbookEngine()
    engine.apply_snapshot(
        {
            "bids": [{"price": "100", "size": "2"}, {"price": "99", "size": "3"}],
            "asks": [{"price": "101", "size": "4"}, {"price": "102", "size": "1"}],
        }
    )
    engine.apply_delta(
        {
            "bids": [{"price": "101", "size": "1"}],
            "asks": [{"price": "101", "size": "0.5"}],
        }
    )

    bid, ask = engine.derive_bbo()
    assert bid is not None and bid.price == 101 and bid.size == 1
    assert ask is not None and ask.price == 101 and ask.size == 0.5


def test_removing_levels_updates_bbo() -> None:
    engine = OrderbookEngine()
    engine.apply_snapshot(
        {
            "bids": [{"price": "100", "size": "1"}, {"price": "99", "size": "2"}],
            "asks": [{"price": "101", "size": "1"}, {"price": "102", "size": "2"}],
        }
    )
    engine.apply_delta({"bids": [{"price": "100", "size": "0"}], "asks": [{"price": "101", "size": "0"}]})

    bid, ask = engine.derive_bbo()
    assert bid is not None and bid.price == 99
    assert ask is not None and ask.price == 102
