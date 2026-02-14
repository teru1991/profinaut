def test_healthz(client):
    """Test GET /healthz endpoint"""
    response = client.get("/healthz")
    assert response.status_code == 200
    payload = response.json()
    assert payload["status"] == "ok"
    assert "timestamp" in payload


def test_capabilities(client):
    """Test GET /capabilities endpoint indicates paper_execution=true"""
    response = client.get("/capabilities")
    assert response.status_code == 200
    payload = response.json()
    assert payload["service"] == "execution"
    assert payload["version"]
    assert payload["status"] in {"ok", "degraded"}
    assert isinstance(payload["features"], list)
    assert "paper_execution" in payload["features"]
    assert "generated_at" in payload


def test_post_order_intent_success(client):
    """Test POST /execution/order-intents returns 201 with Order JSON"""
    order_intent = {
        "idempotency_key": "test-order-1",
        "exchange": "binance",
        "symbol": "BTC/USDT",
        "side": "BUY",
        "qty": 0.01,
        "type": "MARKET",
    }

    response = client.post("/execution/order-intents", json=order_intent)
    assert response.status_code == 201

    order = response.json()
    assert "order_id" in order
    assert order["order_id"].startswith("paper-")
    assert order["status"] == "NEW"
    assert order["exchange"] == "binance"
    assert order["symbol"] == "BTC/USDT"
    assert order["side"] == "BUY"
    assert order["qty"] == 0.01
    assert order["filled_qty"] == 0.0
    assert "accepted_ts_utc" in order


def test_post_order_intent_limit_order(client):
    """Test POST /execution/order-intents with LIMIT order"""
    order_intent = {
        "idempotency_key": "test-limit-1",
        "exchange": "binance",
        "symbol": "ETH/USDT",
        "side": "SELL",
        "qty": 1.5,
        "type": "LIMIT",
        "limit_price": 2000.50,
    }

    response = client.post("/execution/order-intents", json=order_intent)
    assert response.status_code == 201

    order = response.json()
    assert order["order_id"].startswith("paper-")
    assert order["status"] == "NEW"


def test_post_order_intent_duplicate_idempotency_key(client):
    """Test duplicate idempotency_key returns 409"""
    order_intent = {
        "idempotency_key": "duplicate-test",
        "exchange": "binance",
        "symbol": "BTC/USDT",
        "side": "BUY",
        "qty": 0.01,
        "type": "MARKET",
    }

    # First request should succeed
    response1 = client.post("/execution/order-intents", json=order_intent)
    assert response1.status_code == 201

    # Second request with same idempotency_key should return 409
    response2 = client.post("/execution/order-intents", json=order_intent)
    assert response2.status_code == 409
    assert "Duplicate idempotency_key" in response2.json()["detail"]


def test_post_order_intent_unknown_symbol_rejected(client):
    """Test unknown symbol is rejected by default (allowlist)"""
    order_intent = {
        "idempotency_key": "test-unknown-symbol",
        "exchange": "binance",
        "symbol": "UNKNOWN/PAIR",
        "side": "BUY",
        "qty": 1.0,
        "type": "MARKET",
    }

    response = client.post("/execution/order-intents", json=order_intent)
    assert response.status_code == 400
    assert "not allowed" in response.json()["detail"]


def test_post_order_intent_unknown_exchange_rejected(client):
    """Test unknown exchange is rejected by default"""
    order_intent = {
        "idempotency_key": "test-unknown-exchange",
        "exchange": "unknown_exchange",
        "symbol": "BTC/USDT",
        "side": "BUY",
        "qty": 1.0,
        "type": "MARKET",
    }

    response = client.post("/execution/order-intents", json=order_intent)
    assert response.status_code == 400
    assert "not allowed" in response.json()["detail"]


def test_post_order_intent_limit_without_price(client):
    """Test LIMIT order without limit_price is rejected"""
    order_intent = {
        "idempotency_key": "test-limit-no-price",
        "exchange": "binance",
        "symbol": "BTC/USDT",
        "side": "BUY",
        "qty": 0.01,
        "type": "LIMIT",
        # Missing limit_price
    }

    response = client.post("/execution/order-intents", json=order_intent)
    assert response.status_code == 400
    assert "limit_price" in response.json()["detail"]


def test_post_order_intent_invalid_side(client):
    """Test invalid side is rejected"""
    order_intent = {
        "idempotency_key": "test-invalid-side",
        "exchange": "binance",
        "symbol": "BTC/USDT",
        "side": "INVALID",
        "qty": 0.01,
        "type": "MARKET",
    }

    response = client.post("/execution/order-intents", json=order_intent)
    assert response.status_code == 422  # Pydantic validation error


def test_post_order_intent_negative_qty(client):
    """Test negative quantity is rejected"""
    order_intent = {
        "idempotency_key": "test-negative-qty",
        "exchange": "binance",
        "symbol": "BTC/USDT",
        "side": "BUY",
        "qty": -1.0,
        "type": "MARKET",
    }

    response = client.post("/execution/order-intents", json=order_intent)
    assert response.status_code == 422  # Pydantic validation error


def test_post_order_intent_missing_required_field(client):
    """Test missing required field is rejected"""
    order_intent = {
        "idempotency_key": "test-missing-field",
        "exchange": "binance",
        # Missing symbol
        "side": "BUY",
        "qty": 0.01,
        "type": "MARKET",
    }

    response = client.post("/execution/order-intents", json=order_intent)
    assert response.status_code == 422  # Pydantic validation error
