"""
Market data provider with caching.
"""
import logging
import random
from datetime import UTC, datetime

from contracts.schemas import MarketData, MarketDataType

logger = logging.getLogger(__name__)


class MarketDataProvider:
    """Market data provider with dummy data generation and caching."""

    def __init__(self):
        self.cache: dict[str, MarketData] = {}
        self.symbols = ["BTC/USD", "ETH/USD", "SOL/USD", "MATIC/USD"]

    async def get_price(self, symbol: str) -> MarketData:
        """Get current price for a symbol (dummy data)."""
        # Generate dummy price
        base_prices = {
            "BTC/USD": 50000.0,
            "ETH/USD": 3000.0,
            "SOL/USD": 100.0,
            "MATIC/USD": 1.0,
        }
        base_price = base_prices.get(symbol, 1000.0)
        # Add random variation +/- 5%
        price = base_price * (1 + random.uniform(-0.05, 0.05))

        market_data = MarketData(
            symbol=symbol,
            data_type=MarketDataType.PRICE,
            timestamp=datetime.now(UTC),
            value=price,
            source="dummy",
        )

        # Cache the data
        self.cache[f"price:{symbol}"] = market_data
        logger.info(f"Generated dummy price for {symbol}: {price:.2f}")

        return market_data

    async def get_volume(self, symbol: str) -> MarketData:
        """Get current volume for a symbol (dummy data)."""
        volume = random.uniform(1000000, 10000000)

        market_data = MarketData(
            symbol=symbol,
            data_type=MarketDataType.VOLUME,
            timestamp=datetime.now(UTC),
            value=volume,
            source="dummy",
        )

        self.cache[f"volume:{symbol}"] = market_data
        logger.info(f"Generated dummy volume for {symbol}: {volume:.2f}")

        return market_data

    async def get_orderbook(self, symbol: str) -> MarketData:
        """Get orderbook for a symbol (dummy data)."""
        bids = [[random.uniform(1000, 5000), random.uniform(0.1, 10)] for _ in range(5)]
        asks = [[random.uniform(1000, 5000), random.uniform(0.1, 10)] for _ in range(5)]

        orderbook_data = {"bids": bids, "asks": asks}

        market_data = MarketData(
            symbol=symbol,
            data_type=MarketDataType.ORDERBOOK,
            timestamp=datetime.now(UTC),
            value=orderbook_data,
            source="dummy",
        )

        self.cache[f"orderbook:{symbol}"] = market_data
        logger.info(f"Generated dummy orderbook for {symbol}")

        return market_data

    def get_cached(self, key: str) -> MarketData | None:
        """Get cached market data."""
        return self.cache.get(key)

    def list_symbols(self) -> list[str]:
        """List available symbols."""
        return self.symbols


# Global market data provider instance
market_data_provider = MarketDataProvider()
