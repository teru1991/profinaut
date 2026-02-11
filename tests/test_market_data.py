"""
Tests for market data provider.
"""
import pytest

from contracts.schemas import MarketDataType
from dashboard_api.market_data import MarketDataProvider


@pytest.mark.asyncio
async def test_get_price():
    """Test getting price data."""
    provider = MarketDataProvider()
    data = await provider.get_price("BTC/USD")

    assert data.symbol == "BTC/USD"
    assert data.data_type == MarketDataType.PRICE
    assert isinstance(data.value, float)
    assert data.source == "dummy"


@pytest.mark.asyncio
async def test_get_volume():
    """Test getting volume data."""
    provider = MarketDataProvider()
    data = await provider.get_volume("ETH/USD")

    assert data.symbol == "ETH/USD"
    assert data.data_type == MarketDataType.VOLUME
    assert isinstance(data.value, float)


@pytest.mark.asyncio
async def test_get_orderbook():
    """Test getting orderbook data."""
    provider = MarketDataProvider()
    data = await provider.get_orderbook("SOL/USD")

    assert data.symbol == "SOL/USD"
    assert data.data_type == MarketDataType.ORDERBOOK
    assert isinstance(data.value, dict)
    assert "bids" in data.value
    assert "asks" in data.value


def test_list_symbols():
    """Test listing symbols."""
    provider = MarketDataProvider()
    symbols = provider.list_symbols()

    assert len(symbols) > 0
    assert "BTC/USD" in symbols
