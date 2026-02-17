"""
Tests for contracts schemas.
"""
from datetime import UTC, datetime

from contracts.schemas import (
    BotConfig,
    BotState,
    BotStatus,
    HealthStatus,
    KillSwitch,
    MarketData,
    MarketDataType,
)


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
