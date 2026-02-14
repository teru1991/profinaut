import os
from typing import Literal

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", env_file_encoding="utf-8", extra="ignore")

    service_name: str = "execution"
    service_version: str = "0.1.0"
    
    # Symbol allowlist for paper execution (comma-separated)
    # Empty means reject all by default (safe default)
    allowed_symbols: str = Field(default="", description="Comma-separated list of allowed symbols")
    
    # Exchange allowlist (comma-separated)
    allowed_exchanges: str = Field(default="", description="Comma-separated list of allowed exchanges")

    def get_allowed_symbols(self) -> set[str]:
        """Return set of allowed symbols, empty set means reject all"""
        if not self.allowed_symbols:
            return set()
        return {s.strip() for s in self.allowed_symbols.split(",") if s.strip()}

    def get_allowed_exchanges(self) -> set[str]:
        """Return set of allowed exchanges, empty set means reject all"""
        if not self.allowed_exchanges:
            return set()
        return {e.strip() for e in self.allowed_exchanges.split(",") if e.strip()}


_settings: Settings | None = None


def get_settings() -> Settings:
    global _settings
    if _settings is None:
        _settings = Settings()
    return _settings
