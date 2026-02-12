"""
Worker service for background tasks.
"""
import asyncio
import logging

from dashboard_api.bot_registry import bot_registry
from dashboard_api.logging_config import setup_logging
from dashboard_api.market_data import market_data_provider

# Setup logging
setup_logging("worker")
logger = logging.getLogger(__name__)


async def market_data_update_task():
    """Background task to update market data."""
    logger.info("Starting market data update task")

    while True:
        try:
            # Update prices for all symbols
            for symbol in market_data_provider.list_symbols():
                await market_data_provider.get_price(symbol)

            # Sleep for 10 seconds
            await asyncio.sleep(10)
        except Exception as e:
            logger.error(f"Error in market data update task: {e}")
            await asyncio.sleep(5)


async def bot_monitoring_task():
    """Background task to monitor bots."""
    logger.info("Starting bot monitoring task")

    while True:
        try:
            # Check bot status
            bots = bot_registry.list_bots()
            logger.info(f"Monitoring {len(bots)} bots")

            for bot_state in bots:
                logger.info(
                    f"Bot {bot_state.bot_id} status: {bot_state.status}",
                    extra={"bot_id": bot_state.bot_id, "status": bot_state.status.value},
                )

            # Sleep for 30 seconds
            await asyncio.sleep(30)
        except Exception as e:
            logger.error(f"Error in bot monitoring task: {e}")
            await asyncio.sleep(5)


async def main():
    """Main worker function."""
    logger.info("Worker starting")

    # Run background tasks
    await asyncio.gather(market_data_update_task(), bot_monitoring_task())


if __name__ == "__main__":
    asyncio.run(main())
