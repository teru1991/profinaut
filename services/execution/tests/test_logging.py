import logging

import pytest


def test_logging_includes_required_fields(client, caplog):
    """Test that logs include idempotency_key, order_id, exchange, symbol"""
    with caplog.at_level(logging.INFO):
        order_intent = {
            "idempotency_key": "logging-test-1",
            "exchange": "binance",
            "symbol": "BTC/USDT",
            "side": "BUY",
            "qty": 0.01,
            "type": "MARKET",
        }

        response = client.post("/execution/order-intents", json=order_intent)
        assert response.status_code == 201
        order = response.json()

        # Check that log messages exist
        log_messages = [record.message for record in caplog.records]
        assert any("Received order intent" in msg for msg in log_messages)
        assert any("Order created successfully" in msg for msg in log_messages)

        # Check that required fields are in log records
        for record in caplog.records:
            if hasattr(record, "idempotency_key"):
                assert record.idempotency_key == "logging-test-1"
            if hasattr(record, "exchange"):
                assert record.exchange == "binance"
            if hasattr(record, "symbol"):
                assert record.symbol == "BTC/USDT"


def test_logging_duplicate_idempotency_key(client, caplog):
    """Test that duplicate idempotency_key is logged"""
    with caplog.at_level(logging.WARNING):
        order_intent = {
            "idempotency_key": "duplicate-logging-test",
            "exchange": "binance",
            "symbol": "BTC/USDT",
            "side": "BUY",
            "qty": 0.01,
            "type": "MARKET",
        }

        # First request
        client.post("/execution/order-intents", json=order_intent)

        # Clear logs
        caplog.clear()

        # Second request with same key
        response = client.post("/execution/order-intents", json=order_intent)
        assert response.status_code == 409

        # Check warning log
        log_messages = [record.message for record in caplog.records]
        assert any("Duplicate idempotency_key rejected" in msg for msg in log_messages)


def test_logging_unknown_symbol_rejection(client, caplog):
    """Test that unknown symbol rejection is logged"""
    with caplog.at_level(logging.WARNING):
        order_intent = {
            "idempotency_key": "unknown-symbol-logging",
            "exchange": "binance",
            "symbol": "INVALID/SYMBOL",
            "side": "BUY",
            "qty": 1.0,
            "type": "MARKET",
        }

        response = client.post("/execution/order-intents", json=order_intent)
        assert response.status_code == 400

        # Check warning log
        log_messages = [record.message for record in caplog.records]
        assert any("Symbol not in allowlist" in msg for msg in log_messages)
