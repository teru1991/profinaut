"""
FastAPI Dashboard API application.
"""
import logging
from datetime import datetime

from fastapi import FastAPI, HTTPException
from prometheus_client import Counter, generate_latest
from starlette.responses import Response

from contracts.schemas import (
    BotConfig,
    BotState,
    HealthStatus,
    KillSwitch,
    MarketData,
)
from dashboard_api.bot_registry import bot_registry
from dashboard_api.logging_config import setup_logging
from dashboard_api.market_data import market_data_provider

# Setup logging
setup_logging("dashboard-api")
logger = logging.getLogger(__name__)

# Prometheus metrics
request_counter = Counter("api_requests_total", "Total API requests", ["endpoint"])

# FastAPI app
app = FastAPI(
    title="Profinaut Dashboard API",
    description="Investment platform dashboard API",
    version="0.1.0",
)

# Kill switch (no real trading)
KILL_SWITCH = KillSwitch(enabled=True, message="Real trading disabled - demo mode only")


@app.get("/health", response_model=HealthStatus)
async def health_check():
    """Health check endpoint."""
    request_counter.labels(endpoint="/health").inc()
    logger.info("Health check requested")

    return HealthStatus(
        status="healthy",
        timestamp=datetime.utcnow(),
        services={
            "api": "healthy",
            "bots": f"{len(bot_registry.bots)} bots registered",
            "market_data": "healthy",
        },
    )


@app.get("/metrics")
async def metrics():
    """Prometheus metrics endpoint."""
    logger.info("Metrics requested")
    return Response(content=generate_latest(), media_type="text/plain")


@app.get("/kill-switch", response_model=KillSwitch)
async def get_kill_switch():
    """Get kill switch status."""
    request_counter.labels(endpoint="/kill-switch").inc()
    return KILL_SWITCH


@app.get("/bots", response_model=list[BotState])
async def list_bots():
    """List all bots."""
    request_counter.labels(endpoint="/bots").inc()
    logger.info("Listing bots")
    return bot_registry.list_bots()


@app.post("/bots", response_model=BotState)
async def create_bot(config: BotConfig):
    """Create a new bot."""
    request_counter.labels(endpoint="/bots").inc()
    logger.info(f"Creating bot {config.bot_id}")

    if KILL_SWITCH.enabled:
        logger.warning("Bot creation attempted but kill switch is enabled")

    try:
        bot = await bot_registry.create_bot(config)
        return bot.state
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))


@app.post("/bots/{bot_id}/start", response_model=BotState)
async def start_bot(bot_id: str):
    """Start a bot."""
    request_counter.labels(endpoint="/bots/start").inc()
    logger.info(f"Starting bot {bot_id}")

    if KILL_SWITCH.enabled:
        logger.warning(f"Bot {bot_id} start attempted but kill switch is enabled")
        # Still allow starting in demo mode
        pass

    try:
        await bot_registry.start_bot(bot_id)
        bot = bot_registry.get_bot(bot_id)
        return bot.state if bot else None
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))


@app.post("/bots/{bot_id}/stop", response_model=BotState)
async def stop_bot(bot_id: str):
    """Stop a bot."""
    request_counter.labels(endpoint="/bots/stop").inc()
    logger.info(f"Stopping bot {bot_id}")

    try:
        await bot_registry.stop_bot(bot_id)
        bot = bot_registry.get_bot(bot_id)
        return bot.state if bot else None
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))


@app.get("/market-data/symbols", response_model=list[str])
async def list_symbols():
    """List available market symbols."""
    request_counter.labels(endpoint="/market-data/symbols").inc()
    return market_data_provider.list_symbols()


@app.get("/market-data/{symbol}/price", response_model=MarketData)
async def get_price(symbol: str):
    """Get current price for a symbol."""
    request_counter.labels(endpoint="/market-data/price").inc()
    logger.info(f"Fetching price for {symbol}")
    return await market_data_provider.get_price(symbol)


@app.get("/market-data/{symbol}/volume", response_model=MarketData)
async def get_volume(symbol: str):
    """Get current volume for a symbol."""
    request_counter.labels(endpoint="/market-data/volume").inc()
    logger.info(f"Fetching volume for {symbol}")
    return await market_data_provider.get_volume(symbol)


@app.get("/market-data/{symbol}/orderbook", response_model=MarketData)
async def get_orderbook(symbol: str):
    """Get orderbook for a symbol."""
    request_counter.labels(endpoint="/market-data/orderbook").inc()
    logger.info(f"Fetching orderbook for {symbol}")
    return await market_data_provider.get_orderbook(symbol)


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8000)
