"""
Database models using SQLAlchemy.
"""
from datetime import datetime

from sqlalchemy import JSON, Boolean, Column, DateTime, Integer, String, create_engine
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker

Base = declarative_base()


class BotConfigDB(Base):
    """Bot configuration stored in database."""

    __tablename__ = "bot_configs"

    id = Column(Integer, primary_key=True, index=True)
    bot_id = Column(String, unique=True, index=True, nullable=False)
    bot_type = Column(String, nullable=False)
    enabled = Column(Boolean, default=True)
    config = Column(JSON, nullable=False)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)


class MarketDataDB(Base):
    """Market data stored in database."""

    __tablename__ = "market_data"

    id = Column(Integer, primary_key=True, index=True)
    symbol = Column(String, index=True, nullable=False)
    data_type = Column(String, nullable=False)
    timestamp = Column(DateTime, index=True, nullable=False)
    value = Column(JSON, nullable=False)
    source = Column(String, default="dummy")
    created_at = Column(DateTime, default=datetime.utcnow)


def get_database_url() -> str:
    """Get database URL from environment or default."""
    import os

    return os.getenv(
        "DATABASE_URL", "postgresql://profinaut:profinaut@localhost:5432/profinaut"
    )


def get_engine():
    """Create database engine."""
    return create_engine(get_database_url())


def get_session_local():
    """Create session factory."""
    engine = get_engine()
    return sessionmaker(autocommit=False, autoflush=False, bind=engine)
