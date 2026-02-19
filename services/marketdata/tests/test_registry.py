from __future__ import annotations

import json

import pytest

from services.marketdata.app.registry import CatalogValidationError, load_venue_registry


def test_load_venue_registry_gmocoin_success() -> None:
    registry = load_venue_registry("gmocoin")

    assert registry.venue == "gmocoin"
    assert len(registry.connections) > 0
    ticker_rest = next(c for c in registry.connections if c.connection_id == "crypto.public.rest.ticker.get")
    assert ticker_rest.op is not None
    assert ticker_rest.op.value == "fetch_ticker"
    assert ticker_rest.requires_auth is False
    assert ticker_rest.supported is True

    private_ws = next(c for c in registry.connections if c.connection_id == "crypto.private.ws.executionevents.update")
    assert private_ws.requires_auth is True

    assert registry.capabilities["fetch_ticker"]["supported"] is True
    assert registry.capabilities["fetch_balance"]["supported"] is False


def test_load_venue_registry_rejects_duplicate_id(tmp_path, monkeypatch: pytest.MonkeyPatch) -> None:
    venue = "invalidx"
    exch_dir = tmp_path / "docs" / "exchanges" / venue
    exch_dir.mkdir(parents=True)
    payload = {
        "exchange": venue,
        "rest_endpoints": [
            {"id": "same", "visibility": "public", "method": "GET", "path": "/a", "operation": "Get ticker"}
        ],
        "ws_channels": [
            {"id": "same", "visibility": "public", "channel": "ticker", "ws_url": "wss://example"}
        ],
        "data_feeds": [],
    }
    (exch_dir / "catalog.json").write_text(json.dumps(payload), encoding="utf-8")
    monkeypatch.setattr("services.marketdata.app.registry._DOCS_EXCHANGES_ROOT", tmp_path / "docs" / "exchanges")

    with pytest.raises(CatalogValidationError, match="duplicate id"):
        load_venue_registry(venue)
