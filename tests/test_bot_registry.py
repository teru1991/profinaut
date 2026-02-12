"""
Tests for bot registry.
"""
import pytest

from contracts.schemas import BotConfig
from dashboard_api.bot_registry import BotRegistry, DummyBot


@pytest.mark.asyncio
async def test_create_bot():
    """Test bot creation."""
    registry = BotRegistry()
    config = BotConfig(bot_id="test-bot-1", bot_type="dummy")

    bot = await registry.create_bot(config)

    assert bot.config.bot_id == "test-bot-1"
    assert isinstance(bot, DummyBot)
    assert "test-bot-1" in registry.bots


@pytest.mark.asyncio
async def test_start_stop_bot():
    """Test bot start and stop."""
    registry = BotRegistry()
    config = BotConfig(bot_id="test-bot-2", bot_type="dummy")

    await registry.create_bot(config)
    await registry.start_bot("test-bot-2")

    bot = registry.get_bot("test-bot-2")
    assert bot.state.status.value == "running"

    await registry.stop_bot("test-bot-2")
    assert bot.state.status.value == "stopped"


@pytest.mark.asyncio
async def test_list_bots():
    """Test listing bots."""
    registry = BotRegistry()
    config1 = BotConfig(bot_id="test-bot-3", bot_type="dummy")
    config2 = BotConfig(bot_id="test-bot-4", bot_type="dummy")

    await registry.create_bot(config1)
    await registry.create_bot(config2)

    bots = registry.list_bots()
    assert len(bots) == 2
