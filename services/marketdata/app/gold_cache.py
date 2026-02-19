from __future__ import annotations

import random
import threading
import time
from collections.abc import Callable
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class CacheStats:
    hit: int = 0
    miss: int = 0


class HotCache:
    def __init__(self, *, default_ttl_seconds: float = 2.0, jitter_seconds: float = 0.2):
        self._default_ttl_seconds = default_ttl_seconds
        self._jitter_seconds = jitter_seconds
        self._store: dict[str, tuple[float, Any]] = {}
        self._lock = threading.Lock()
        self._key_locks: dict[str, threading.Lock] = {}
        self._stats = CacheStats()

    def get(self, key: str) -> Any | None:
        now = time.time()
        with self._lock:
            item = self._store.get(key)
            if item is None:
                self._stats = CacheStats(hit=self._stats.hit, miss=self._stats.miss + 1)
                return None
            expires_at, value = item
            if expires_at < now:
                self._store.pop(key, None)
                self._stats = CacheStats(hit=self._stats.hit, miss=self._stats.miss + 1)
                return None
            self._stats = CacheStats(hit=self._stats.hit + 1, miss=self._stats.miss)
            return value

    def set(self, key: str, value: Any, *, ttl_seconds: float | None = None) -> None:
        ttl = ttl_seconds if ttl_seconds is not None else self._default_ttl_seconds
        ttl = max(0.05, ttl + random.uniform(0, self._jitter_seconds))
        with self._lock:
            self._store[key] = (time.time() + ttl, value)

    def invalidate(self, key: str) -> None:
        with self._lock:
            self._store.pop(key, None)

    def get_or_load(self, key: str, loader: Callable[[], Any], *, ttl_seconds: float | None = None) -> Any:
        cached = self.get(key)
        if cached is not None:
            return cached
        with self._lock:
            key_lock = self._key_locks.setdefault(key, threading.Lock())
        with key_lock:
            second = self.get(key)
            if second is not None:
                return second
            value = loader()
            if value is not None:
                self.set(key, value, ttl_seconds=ttl_seconds)
            return value

    def stats(self) -> CacheStats:
        with self._lock:
            return self._stats
