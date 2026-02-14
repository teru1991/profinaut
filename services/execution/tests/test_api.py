from app.storage import get_storage


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


def test_post_order_intent_rejects_unknown_field(client):
    """Test unknown fields are rejected to match OpenAPI schema"""
    order_intent = {
        "idempotency_key": "test-unknown-field",
        "exchange": "binance",
        "symbol": "BTC/USDT",
        "side": "BUY",
        "qty": 0.01,
        "type": "MARKET",
        "unexpected": "value",
    }

    response = client.post("/execution/order-intents", json=order_intent)
    assert response.status_code == 422


def test_gmo_live_requires_explicit_feature_flag(client, monkeypatch):
    monkeypatch.setenv("ALLOWED_EXCHANGES", "binance,coinbase,gmo")
    order_intent = {
        "idempotency_key": "gmo-live-disabled",
        "exchange": "gmo",
        "symbol": "BTC/USDT",
        "side": "BUY",
        "qty": 0.01,
        "type": "MARKET",
    }
    response = client.post("/execution/order-intents", json=order_intent)
    assert response.status_code == 403


def test_gmo_live_place_and_cancel_with_idempotency_mapping(client, monkeypatch):
    from app.live import PlaceOrderResult

    monkeypatch.setenv("EXECUTION_LIVE_ENABLED", "true")
    monkeypatch.setenv("GMO_API_BASE_URL", "https://example.invalid")
    monkeypatch.setenv("GMO_API_KEY", "k")
    monkeypatch.setenv("GMO_API_SECRET", "s")
    monkeypatch.setenv("ALLOWED_EXCHANGES", "binance,coinbase,gmo")

    def _mock_place_order(self, **kwargs):
        assert kwargs["client_order_id"].startswith("pfn-")
        return PlaceOrderResult(order_id="gmo-order-1")

    def _mock_cancel_order(self, **kwargs):
        assert kwargs["order_id"] == "gmo-order-1"
        assert kwargs["client_order_id"].startswith("pfn-")

    monkeypatch.setattr("app.live.GmoLiveExecutor.place_order", _mock_place_order)
    monkeypatch.setattr("app.live.GmoLiveExecutor.cancel_order", _mock_cancel_order)

    order_intent = {
        "idempotency_key": "gmo-live-enabled",
        "exchange": "gmo",
        "symbol": "BTC/USDT",
        "side": "BUY",
        "qty": 0.01,
        "type": "MARKET",
    }

    create_res = client.post("/execution/order-intents", json=order_intent)
    assert create_res.status_code == 201
    assert create_res.json()["order_id"] == "gmo-order-1"

    storage = get_storage()
    assert storage.get_client_order_id_by_idempotency_key("gmo-live-enabled") is not None

    cancel_res = client.post("/execution/orders/gmo-order-1/cancel")
    assert cancel_res.status_code == 200
    assert cancel_res.json()["status"] == "CANCELED"


def test_gmo_live_429_degrades_and_applies_backoff(client, monkeypatch):
    from app.live import LiveRateLimitError

    monkeypatch.setenv("EXECUTION_LIVE_ENABLED", "true")
    monkeypatch.setenv("GMO_API_BASE_URL", "https://example.invalid")
    monkeypatch.setenv("GMO_API_KEY", "k")
    monkeypatch.setenv("GMO_API_SECRET", "s")
    monkeypatch.setenv("LIVE_BACKOFF_SECONDS", "60")
    monkeypatch.setenv("ALLOWED_EXCHANGES", "binance,coinbase,gmo")

    def _mock_place_order(self, **kwargs):
        raise LiveRateLimitError("GMO live order rate limited")

    monkeypatch.setattr("app.live.GmoLiveExecutor.place_order", _mock_place_order)

    order_intent = {
        "idempotency_key": "gmo-rate-limit",
        "exchange": "gmo",
        "symbol": "BTC/USDT",
        "side": "BUY",
        "qty": 0.01,
        "type": "MARKET",
    }
    first = client.post("/execution/order-intents", json=order_intent)
    assert first.status_code == 503

    second = client.post(
        "/execution/order-intents",
        json={**order_intent, "idempotency_key": "gmo-rate-limit-2"},
    )
    assert second.status_code == 503
    assert "degraded" in second.json()["detail"]
