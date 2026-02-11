"""
Tests for dashboard API.
"""
import pytest
from fastapi.testclient import TestClient

from dashboard_api.main import app

client = TestClient(app)


def test_health_check():
    """Test health check endpoint."""
    response = client.get("/health")
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "healthy"
    assert "services" in data


def test_metrics_endpoint():
    """Test metrics endpoint."""
    response = client.get("/metrics")
    assert response.status_code == 200
    assert response.headers["content-type"] == "text/plain; charset=utf-8"


def test_kill_switch():
    """Test kill switch endpoint."""
    response = client.get("/kill-switch")
    assert response.status_code == 200
    data = response.json()
    assert data["enabled"] is True
    assert "demo mode" in data["message"]


def test_list_symbols():
    """Test listing market symbols."""
    response = client.get("/market-data/symbols")
    assert response.status_code == 200
    symbols = response.json()
    assert isinstance(symbols, list)
    assert len(symbols) > 0


def test_get_price():
    """Test getting price data."""
    response = client.get("/market-data/BTC%2FUSD/price")
    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "BTC/USD"
    assert data["data_type"] == "price"
