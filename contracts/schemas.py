"""
Core schemas shared across all services (SSOT).
"""
from datetime import datetime
from enum import Enum
from typing import Any

from pydantic import BaseModel, Field


class BotStatus(str, Enum):
    """Bot lifecycle status."""

    STOPPED = "stopped"
    STARTING = "starting"
    RUNNING = "running"
    STOPPING = "stopping"
    ERROR = "error"


class MarketDataType(str, Enum):
    """Market data types."""

    PRICE = "price"
    VOLUME = "volume"
    ORDERBOOK = "orderbook"


class BotConfig(BaseModel):
    """Bot configuration schema."""

    bot_id: str = Field(..., description="Unique bot identifier")
    bot_type: str = Field(..., description="Bot type/class name")
    enabled: bool = Field(default=True, description="Whether bot is enabled")
    config: dict[str, Any] = Field(default_factory=dict, description="Bot-specific config")


class BotState(BaseModel):
    """Bot runtime state."""

    bot_id: str
    status: BotStatus
    started_at: datetime | None = None
    stopped_at: datetime | None = None
    error_message: str | None = None


class MarketData(BaseModel):
    """Market data point."""

    symbol: str = Field(..., description="Trading symbol (e.g., BTC/USD)")
    data_type: MarketDataType
    timestamp: datetime
    value: float | dict[str, Any]
    source: str = Field(default="dummy", description="Data source")


class HealthStatus(BaseModel):
    """Health check response."""

    status: str
    timestamp: datetime
    services: dict[str, str] = Field(default_factory=dict)


class KillSwitch(BaseModel):
    """Kill switch state (no real trading allowed)."""

    enabled: bool = Field(default=True, description="Kill switch enabled")
    message: str = Field(
        default="Real trading disabled - demo mode only", description="Kill switch message"
    )
