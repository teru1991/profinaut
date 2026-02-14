import os
from collections.abc import Generator

import pytest
from fastapi.testclient import TestClient

# Set environment variables before importing app
os.environ["ALLOWED_SYMBOLS"] = "BTC/USDT,ETH/USDT"
os.environ["ALLOWED_EXCHANGES"] = "binance,coinbase"

from app.main import app  # noqa: E402
from app.storage import OrderStorage, _storage  # noqa: E402


@pytest.fixture()
def client() -> Generator[TestClient, None, None]:
    # Reset storage before each test
    global _storage
    _storage = OrderStorage()

    with TestClient(app) as c:
        yield c
