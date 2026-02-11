"""
Bot registry and lifecycle management.
"""
import logging
from datetime import datetime
from typing import Any

from contracts.schemas import BotConfig, BotState, BotStatus

logger = logging.getLogger(__name__)


class Bot:
    """Base bot class."""

    def __init__(self, config: BotConfig):
        self.config = config
        self.state = BotState(bot_id=config.bot_id, status=BotStatus.STOPPED)

    async def start(self) -> None:
        """Start the bot."""
        logger.info(f"Starting bot {self.config.bot_id}")
        self.state.status = BotStatus.STARTING
        self.state.started_at = datetime.utcnow()
        await self._start()
        self.state.status = BotStatus.RUNNING

    async def stop(self) -> None:
        """Stop the bot."""
        logger.info(f"Stopping bot {self.config.bot_id}")
        self.state.status = BotStatus.STOPPING
        await self._stop()
        self.state.status = BotStatus.STOPPED
        self.state.stopped_at = datetime.utcnow()

    async def _start(self) -> None:
        """Bot-specific start logic."""
        pass

    async def _stop(self) -> None:
        """Bot-specific stop logic."""
        pass


class DummyBot(Bot):
    """Dummy bot for testing (no real trading)."""

    async def _start(self) -> None:
        """Start dummy bot."""
        logger.info(f"DummyBot {self.config.bot_id} started (demo mode)")

    async def _stop(self) -> None:
        """Stop dummy bot."""
        logger.info(f"DummyBot {self.config.bot_id} stopped")


class BotRegistry:
    """Bot registry for managing bot lifecycle."""

    def __init__(self):
        self.bots: dict[str, Bot] = {}
        self._bot_types: dict[str, type[Bot]] = {"dummy": DummyBot}

    def register_bot_type(self, bot_type: str, bot_class: type[Bot]) -> None:
        """Register a new bot type."""
        self._bot_types[bot_type] = bot_class
        logger.info(f"Registered bot type: {bot_type}")

    async def create_bot(self, config: BotConfig) -> Bot:
        """Create and register a bot."""
        if config.bot_id in self.bots:
            raise ValueError(f"Bot {config.bot_id} already exists")

        bot_class = self._bot_types.get(config.bot_type)
        if not bot_class:
            raise ValueError(f"Unknown bot type: {config.bot_type}")

        bot = bot_class(config)
        self.bots[config.bot_id] = bot
        logger.info(f"Created bot {config.bot_id} of type {config.bot_type}")
        return bot

    async def start_bot(self, bot_id: str) -> None:
        """Start a bot."""
        bot = self.bots.get(bot_id)
        if not bot:
            raise ValueError(f"Bot {bot_id} not found")
        await bot.start()

    async def stop_bot(self, bot_id: str) -> None:
        """Stop a bot."""
        bot = self.bots.get(bot_id)
        if not bot:
            raise ValueError(f"Bot {bot_id} not found")
        await bot.stop()

    def get_bot(self, bot_id: str) -> Bot | None:
        """Get a bot by ID."""
        return self.bots.get(bot_id)

    def list_bots(self) -> list[BotState]:
        """List all bot states."""
        return [bot.state for bot in self.bots.values()]


# Global bot registry instance
bot_registry = BotRegistry()
