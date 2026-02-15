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
    assert payload["safe_mode"] in {"NORMAL", "DEGRADED", "SAFE_MODE", "HALTED"}
    assert isinstance(payload["allowed_actions"], list)
    assert isinstance(payload["features"], list)
    assert "paper_execution" in payload["features"]
    assert "generated_at" in payload




def test_post_order_intent_blocked_in_safe_mode_never_calls_upstream(client, monkeypatch):
    monkeypatch.setenv("EXECUTION_SAFE_MODE", "SAFE_MODE")
    monkeypatch.setenv("ALLOWED_EXCHANGES", "binance,coinbase,gmo")
    monkeypatch.setenv("EXECUTION_LIVE_ENABLED", "true")
    monkeypatch.setenv("EXECUTION_LIVE_MODE", "live")

    called = {"place": 0}

    def _never_called(*_args, **_kwargs):
        called["place"] += 1
        raise AssertionError("upstream place_order should not be called in SAFE_MODE")

    monkeypatch.setattr("app.live.GmoLiveExecutor.place_order", _never_called)

    order_intent = {
        "idempotency_key": "safe-mode-blocked",
        "exchange": "gmo",
        "symbol": "BTC/USDT",
        "side": "BUY",
        "qty": 0.01,
        "type": "MARKET",
    }

    response = client.post("/execution/order-intents", json=order_intent)
    assert response.status_code == 403
    assert response.json()["code"] == "SAFE_MODE_BLOCKED"
    assert called["place"] == 0

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
    assert order["status"] == "ACCEPTED"
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
    assert order["status"] == "ACCEPTED"


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
    assert "Duplicate idempotency_key" in response2.json()["message"]


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
    assert "not allowed" in response.json()["message"]


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
    assert "not allowed" in response.json()["message"]


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
    assert "limit_price" in response.json()["message"]


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
    assert response.json()["code"] == "LIVE_DISABLED"




def test_gmo_live_enabled_but_dry_run_rejects_without_upstream_call(client, monkeypatch):
    monkeypatch.setenv("ALLOWED_EXCHANGES", "binance,coinbase,gmo")
    monkeypatch.setenv("EXECUTION_LIVE_ENABLED", "true")

    called = {"place": 0}

    def _never_called(*_args, **_kwargs):
        called["place"] += 1
        raise AssertionError("upstream place_order should not be called in dry_run mode")

    monkeypatch.setattr("app.live.GmoLiveExecutor.place_order", _never_called)

    order_intent = {
        "idempotency_key": "gmo-dry-run-only",
        "exchange": "gmo",
        "symbol": "BTC/USDT",
        "side": "BUY",
        "qty": 0.01,
        "type": "MARKET",
    }

    response = client.post("/execution/order-intents", json=order_intent)
    assert response.status_code == 403
    assert response.json()["code"] == "DRY_RUN_ONLY"
    assert called["place"] == 0


def test_gmo_live_off_never_calls_upstream(client, monkeypatch):
    monkeypatch.setenv("ALLOWED_EXCHANGES", "binance,coinbase,gmo")
    monkeypatch.setenv("EXECUTION_LIVE_ENABLED", "false")

    called = {"place": 0}

    def _never_called(*_args, **_kwargs):
        called["place"] += 1
        raise AssertionError("upstream place_order should never be called when live is disabled")

    monkeypatch.setattr("app.live.GmoLiveExecutor.place_order", _never_called)

    order_intent = {
        "idempotency_key": "gmo-off-never-call",
        "exchange": "gmo",
        "symbol": "BTC/USDT",
        "side": "BUY",
        "qty": 0.01,
        "type": "MARKET",
    }

    response = client.post("/execution/order-intents", json=order_intent)
    assert response.status_code == 403
    assert response.json()["code"] == "LIVE_DISABLED"
    assert called["place"] == 0


def test_gmo_live_place_and_cancel_with_idempotency_mapping(client, monkeypatch, auth_headers):
    from app.live import PlaceOrderResult
    from app.storage import get_storage

    monkeypatch.setenv("EXECUTION_LIVE_ENABLED", "true")
    monkeypatch.setenv("EXECUTION_LIVE_MODE", "live")
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

    cancel_res = client.post("/execution/orders/gmo-order-1/cancel", headers=auth_headers)
    assert cancel_res.status_code == 200
    assert cancel_res.json()["status"] == "CANCELED"


def test_paper_order_lifecycle_fill_and_terminal_guards(client, auth_headers):
    order_intent = {
        "idempotency_key": "paper-lifecycle-fill",
        "exchange": "binance",
        "symbol": "BTC/USDT",
        "side": "BUY",
        "qty": 0.02,
        "type": "MARKET",
    }
    created = client.post("/execution/order-intents", json=order_intent)
    assert created.status_code == 201
    order_id = created.json()["order_id"]

    fill_res = client.post(f"/execution/orders/{order_id}/fill")
    assert fill_res.status_code == 200
    assert fill_res.json()["status"] == "FILLED"
    assert fill_res.json()["filled_qty"] == 0.02

    cancel_after_fill = client.post(f"/execution/orders/{order_id}/cancel", headers=auth_headers)
    assert cancel_after_fill.status_code == 409


def test_paper_order_lifecycle_reject_and_terminal_guards(client):
    order_intent = {
        "idempotency_key": "paper-lifecycle-reject",
        "exchange": "binance",
        "symbol": "ETH/USDT",
        "side": "SELL",
        "qty": 1.5,
        "type": "MARKET",
    }
    created = client.post("/execution/order-intents", json=order_intent)
    assert created.status_code == 201
    order_id = created.json()["order_id"]

    reject_res = client.post(f"/execution/orders/{order_id}/reject")
    assert reject_res.status_code == 200
    assert reject_res.json()["status"] == "REJECTED"

    fill_after_reject = client.post(f"/execution/orders/{order_id}/fill")
    assert fill_after_reject.status_code == 409


def test_gmo_live_429_degrades_and_applies_backoff(client, monkeypatch):
    from app.live import LiveRateLimitError

    monkeypatch.setenv("EXECUTION_LIVE_ENABLED", "true")
    monkeypatch.setenv("EXECUTION_LIVE_MODE", "live")
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
    assert second.json()["code"] == "LIVE_DEGRADED"


def test_gmo_live_duplicate_idempotency_does_not_place_twice(client, monkeypatch):
    from app.live import PlaceOrderResult

    monkeypatch.setenv("EXECUTION_LIVE_ENABLED", "true")
    monkeypatch.setenv("EXECUTION_LIVE_MODE", "live")
    monkeypatch.setenv("GMO_API_BASE_URL", "https://example.invalid")
    monkeypatch.setenv("GMO_API_KEY", "k")
    monkeypatch.setenv("GMO_API_SECRET", "s")
    monkeypatch.setenv("ALLOWED_EXCHANGES", "binance,coinbase,gmo")

    place_calls: list[str] = []

    def _mock_place_order(self, **kwargs):
        place_calls.append(kwargs["client_order_id"])
        return PlaceOrderResult(order_id="gmo-order-dup")

    monkeypatch.setattr("app.live.GmoLiveExecutor.place_order", _mock_place_order)

    order_intent = {
        "idempotency_key": "gmo-live-dup-key",
        "exchange": "gmo",
        "symbol": "BTC/USDT",
        "side": "BUY",
        "qty": 0.01,
        "type": "MARKET",
    }

    first = client.post("/execution/order-intents", json=order_intent)
    assert first.status_code == 201

    second = client.post("/execution/order-intents", json=order_intent)
    assert second.status_code == 409

    assert len(place_calls) == 1


def test_orders_and_fills_history_endpoints(client):
    order_intent = {
        "idempotency_key": "history-order-1",
        "exchange": "binance",
        "symbol": "BTC/USDT",
        "side": "BUY",
        "qty": 0.05,
        "type": "MARKET",
    }
    created = client.post("/execution/order-intents", json=order_intent)
    assert created.status_code == 201
    order_id = created.json()["order_id"]

    filled = client.post(f"/execution/orders/{order_id}/fill")
    assert filled.status_code == 200

    orders_res = client.get("/orders?page=1&page_size=10")
    assert orders_res.status_code == 200
    orders = orders_res.json()
    assert orders["page"] == 1
    assert orders["page_size"] == 10
    assert orders["total"] >= 1
    assert any(item["order_id"] == order_id for item in orders["items"])

    fills_res = client.get("/fills?page=1&page_size=10")
    assert fills_res.status_code == 200
    fills = fills_res.json()
    assert fills["page"] == 1
    assert fills["page_size"] == 10
    assert fills["total"] >= 1
    assert any(item["order_id"] == order_id for item in fills["items"])


def test_history_endpoints_empty_shape(client):
    orders_res = client.get("/orders?page=1&page_size=5")
    fills_res = client.get("/fills?page=1&page_size=5")

    assert orders_res.status_code == 200
    assert orders_res.json()["items"] == []
    assert orders_res.json()["total"] == 0

    assert fills_res.status_code == 200
    assert fills_res.json()["items"] == []
    assert fills_res.json()["total"] == 0


def test_gmo_cancel_blocked_by_policy_gate_never_calls_upstream(client, monkeypatch, auth_headers):
    from app.schemas import OrderIntent
    from app.storage import get_storage

    monkeypatch.setenv("EXECUTION_LIVE_ENABLED", "false")

    called = {"cancel": 0}

    def _never_called(*_args, **_kwargs):
        called["cancel"] += 1
        raise AssertionError("upstream cancel_order should not be called when live is disabled")

    monkeypatch.setattr("app.live.GmoLiveExecutor.cancel_order", _never_called)

    storage = get_storage()
    intent = OrderIntent(
        idempotency_key="gmo-cancel-gated",
        exchange="gmo",
        symbol="BTC/USDT",
        side="BUY",
        qty=0.01,
        type="MARKET",
    )
    created = storage.create_order(intent, order_id="gmo-cancel-1", client_order_id="pfn-cancel-1")
    assert created is not None

    response = client.post("/execution/orders/gmo-cancel-1/cancel", headers=auth_headers)
    assert response.status_code == 403
    assert response.json()["code"] == "LIVE_DISABLED"
    assert called["cancel"] == 0
