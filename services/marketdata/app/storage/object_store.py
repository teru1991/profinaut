from __future__ import annotations

from abc import ABC, abstractmethod


class ObjectStore(ABC):
    @abstractmethod
    def put_bytes(self, key: str, data: bytes, content_type: str = "application/octet-stream") -> None:
        """Store bytes at key."""

    @abstractmethod
    def get_bytes(self, key: str) -> bytes:
        """Load bytes at key."""

    @abstractmethod
    def list(self, prefix: str) -> list[str]:
        """List keys for prefix."""
