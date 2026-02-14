import os
from collections.abc import Generator

import pytest
from fastapi.testclient import TestClient
from sqlalchemy import create_engine
from sqlalchemy.orm import Session, sessionmaker
from sqlalchemy.pool import StaticPool

os.environ["ADMIN_TOKEN"] = "test-admin-token"
os.environ["DATABASE_URL"] = "sqlite+pysqlite:///:memory:"

from app.database import Base, get_db  # noqa: E402
from app.main import app  # noqa: E402

# Global engine and session for tests
_test_engine = None
_TestingSessionLocal = None


@pytest.fixture()
def client() -> Generator[TestClient, None, None]:
    global _test_engine, _TestingSessionLocal

    _test_engine = create_engine(
        "sqlite+pysqlite:///:memory:",
        future=True,
        connect_args={"check_same_thread": False},
        poolclass=StaticPool,
    )
    _TestingSessionLocal = sessionmaker(
        bind=_test_engine, autoflush=False, autocommit=False, future=True
    )
    Base.metadata.create_all(bind=_test_engine)

    def override_get_db() -> Generator[Session, None, None]:
        db = _TestingSessionLocal()
        try:
            yield db
        finally:
            db.close()

    app.dependency_overrides[get_db] = override_get_db
    with TestClient(app) as c:
        yield c
    app.dependency_overrides.clear()


@pytest.fixture()
def db_session() -> Generator[Session, None, None]:
    """Provides a database session for direct database manipulation in tests."""
    if _TestingSessionLocal is None:
        raise RuntimeError("db_session fixture requires client fixture to be called first")

    db = _TestingSessionLocal()
    try:
        yield db
    finally:
        db.close()
