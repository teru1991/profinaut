import logging

def test_logging_includes_required_fields(client, caplog):
    """Test logs include idempotency_key, order_id, exchange, symbol, status"""
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

        received_record = next(
            (record for record in caplog.records if record.message == "Received order intent"),
            None,
        )
        created_record = next(
            (record for record in caplog.records if record.message == "Order created successfully"),
            None,
        )
        assert received_record is not None
        assert created_record is not None

        assert received_record.idempotency_key == "logging-test-1"
        assert received_record.exchange == "binance"
        assert received_record.symbol == "BTC/USDT"
        assert received_record.order_id is None
        assert received_record.status == "RECEIVED"

        assert created_record.idempotency_key == "logging-test-1"
        assert created_record.exchange == "binance"
        assert created_record.symbol == "BTC/USDT"
        assert created_record.order_id == order["order_id"]
        assert created_record.status == "ACCEPTED"


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

        duplicate_record = next(
            (
                record
                for record in caplog.records
                if record.message == "Duplicate idempotency_key rejected"
            ),
            None,
        )
        assert duplicate_record is not None
        assert duplicate_record.idempotency_key == "duplicate-logging-test"
        assert duplicate_record.exchange == "binance"
        assert duplicate_record.symbol == "BTC/USDT"
        assert duplicate_record.order_id is not None
        assert duplicate_record.status == "REJECTED"


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
