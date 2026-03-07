"""
Tests for contracts schemas.
"""
from __future__ import annotations

import json
from datetime import UTC, datetime
from pathlib import Path

import pytest

from contracts.schemas import (
    BotConfig,
    BotState,
    BotStatus,
    HealthStatus,
    KillSwitch,
    MarketData,
    MarketDataType,
)


SCHEMA_PATH = Path("contracts/schemas/common/error_envelope.schema.json")
SCHEMA = json.loads(SCHEMA_PATH.read_text())
KIND_ENUM = set(SCHEMA["properties"]["error"]["properties"]["kind"]["enum"])
SEVERITY_ENUM = set(SCHEMA["properties"]["error"]["properties"]["severity"]["enum"])
CORRELATION_SCHEMA = json.loads(Path("docs/contracts/observability/correlation.schema.json").read_text())
LOG_EVENT_SCHEMA = json.loads(Path("docs/contracts/observability/log_event.schema.json").read_text())


def _validate_error_envelope(payload: dict) -> None:
    assert "error" in payload
    error = payload["error"]
    for required in SCHEMA["properties"]["error"]["required"]:
        assert required in error
    context = error["context"]
    assert "component" in context
    assert isinstance(error["retryable"], bool)
    assert error["kind"] in KIND_ENUM
    assert error["severity"] in SEVERITY_ENUM


@pytest.fixture
def valid_error_envelope() -> dict:
    return {
        "error": {
            "code": "PLATFORM_INTERNAL_ERROR",
            "reason_code": "UNHANDLED_EXCEPTION",
            "kind": "internal_error",
            "severity": "critical",
            "retryable": False,
            "source": "services.execution",
            "context": {"component": "execution"},
        }
    }


def test_bot_config_creation():
    """Test BotConfig creation."""
    config = BotConfig(bot_id="test-bot", bot_type="dummy", enabled=True, config={})
    assert config.bot_id == "test-bot"
    assert config.bot_type == "dummy"
    assert config.enabled is True


def test_bot_state_creation():
    """Test BotState creation."""
    state = BotState(bot_id="test-bot", status=BotStatus.STOPPED)
    assert state.bot_id == "test-bot"
    assert state.status == BotStatus.STOPPED
    assert state.started_at is None


def test_market_data_creation():
    """Test MarketData creation."""
    data = MarketData(
        symbol="BTC/USD",
        data_type=MarketDataType.PRICE,
        timestamp=datetime.now(UTC),
        value=50000.0,
    )
    assert data.symbol == "BTC/USD"
    assert data.data_type == MarketDataType.PRICE
    assert data.value == 50000.0


def test_health_status_creation():
    """Test HealthStatus creation."""
    health = HealthStatus(status="healthy", timestamp=datetime.now(UTC))
    assert health.status == "healthy"


def test_kill_switch_creation():
    """Test KillSwitch creation."""
    kill_switch = KillSwitch(enabled=True)
    assert kill_switch.enabled is True
    assert "demo mode" in kill_switch.message


def test_error_envelope_contract_valid(valid_error_envelope: dict) -> None:
    _validate_error_envelope(valid_error_envelope)


def test_error_envelope_missing_code_fails(valid_error_envelope: dict) -> None:
    del valid_error_envelope["error"]["code"]
    with pytest.raises(AssertionError):
        _validate_error_envelope(valid_error_envelope)


def test_error_envelope_missing_reason_code_fails(valid_error_envelope: dict) -> None:
    del valid_error_envelope["error"]["reason_code"]
    with pytest.raises(AssertionError):
        _validate_error_envelope(valid_error_envelope)


def test_error_envelope_missing_kind_fails(valid_error_envelope: dict) -> None:
    del valid_error_envelope["error"]["kind"]
    with pytest.raises(AssertionError):
        _validate_error_envelope(valid_error_envelope)


def test_error_envelope_retryable_must_be_bool(valid_error_envelope: dict) -> None:
    valid_error_envelope["error"]["retryable"] = "yes"
    with pytest.raises(AssertionError):
        _validate_error_envelope(valid_error_envelope)


def test_error_envelope_context_component_required(valid_error_envelope: dict) -> None:
    del valid_error_envelope["error"]["context"]["component"]
    with pytest.raises(AssertionError):
        _validate_error_envelope(valid_error_envelope)


def test_error_envelope_kind_enum_rejects_unknown(valid_error_envelope: dict) -> None:
    valid_error_envelope["error"]["kind"] = "made_up_kind"
    with pytest.raises(AssertionError):
        _validate_error_envelope(valid_error_envelope)


def test_observability_correlation_schema_required_fields() -> None:
    required = set(CORRELATION_SCHEMA["required"])
    assert {"trace_id", "run_id", "component", "source", "schema_version"}.issubset(required)


def test_observability_log_event_schema_required_fields() -> None:
    required = set(LOG_EVENT_SCHEMA["required"])
    assert {"timestamp", "level", "message", "component", "trace_id", "run_id", "schema_version"}.issubset(required)
