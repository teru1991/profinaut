import os
from collections.abc import Generator

import pytest
from fastapi.testclient import TestClient

# Set environment variables before importing app
os.environ["ALLOWED_SYMBOLS"] = "BTC/USDT,ETH/USDT"
os.environ["ALLOWED_EXCHANGES"] = "binance,coinbase"
os.environ["EXECUTION_API_TOKEN"] = "test-token-12345"

import app.config as app_config  # noqa: E402
import app.main as app_main  # noqa: E402
import app.storage as app_storage  # noqa: E402
from app.main import app  # noqa: E402
from app.storage import OrderStorage  # noqa: E402


@pytest.fixture()
def client(tmp_path) -> Generator[TestClient, None, None]:
    # Reset storage before each test
    os.environ["EXECUTION_STORAGE_DB_PATH"] = str(tmp_path / "execution-test.sqlite")
    app_storage._storage = OrderStorage(db_path=os.environ["EXECUTION_STORAGE_DB_PATH"])
    app_config._settings = None
    app_main._degraded_reason = None
    app_main._live_backoff_until_utc = None

    with TestClient(app) as c:
        yield c


@pytest.fixture()
def auth_headers() -> dict[str, str]:
    """Return authentication headers for protected endpoints."""
    return {"X-Execution-Token": "test-token-12345"}
