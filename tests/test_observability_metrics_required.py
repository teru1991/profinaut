import importlib.util
import sys
import types
from pathlib import Path

from fastapi.testclient import TestClient


def _has(text: str, name: str) -> bool:
    return name in text


def _load_dashboard_app():
    path = Path("services/dashboard-api/app/main.py")
    app_dir = str(path.parent)

    services_pkg = sys.modules.setdefault("services", types.ModuleType("services"))
    if not hasattr(services_pkg, "__path__"):
        services_pkg.__path__ = []

    dash_pkg = sys.modules.setdefault("services.dashboard_api", types.ModuleType("services.dashboard_api"))
    dash_pkg.__path__ = [str(path.parent.parent)]

    app_pkg = sys.modules.setdefault("services.dashboard_api.app", types.ModuleType("services.dashboard_api.app"))
    app_pkg.__path__ = [app_dir]

    module_name = "services.dashboard_api.app.main"
    spec = importlib.util.spec_from_file_location(module_name, path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module.app


def test_execution_has_metrics():
    from services.execution.app.main import app

    client = TestClient(app)
    response = client.get("/metrics")
    assert response.status_code == 200
    text = response.text
    assert _has(text, "profinaut_build_info")
    assert _has(text, "profinaut_uptime_seconds")
    assert _has(text, "profinaut_http_requests_total")


def test_marketdata_has_required_metrics():
    from services.marketdata.app.main import app

    client = TestClient(app)
    response = client.get("/metrics")
    assert response.status_code == 200
    text = response.text
    assert _has(text, "profinaut_build_info")
    assert _has(text, "profinaut_uptime_seconds")
    assert _has(text, "profinaut_http_requests_total")


def test_dashboard_api_has_required_metrics():
    app = _load_dashboard_app()

    client = TestClient(app)
    response = client.get("/metrics")
    assert response.status_code == 200
    text = response.text
    assert _has(text, "profinaut_build_info")
    assert _has(text, "profinaut_uptime_seconds")
    assert _has(text, "profinaut_http_requests_total")
