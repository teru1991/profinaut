from __future__ import annotations

from services.marketdata.app.hash_util import compute_payload_hash


def test_compute_payload_hash_is_stable_across_runs() -> None:
    payload = {"symbol": "BTC_JPY", "price": 100.1, "meta": {"a": 1, "b": [2, 3]}}

    h1 = compute_payload_hash(payload)
    h2 = compute_payload_hash(payload)

    assert h1 == h2


def test_compute_payload_hash_is_key_order_invariant() -> None:
    payload_a = {"a": 1, "b": 2, "nested": {"x": 10, "y": 20}}
    payload_b = {"nested": {"y": 20, "x": 10}, "b": 2, "a": 1}

    assert compute_payload_hash(payload_a) == compute_payload_hash(payload_b)
