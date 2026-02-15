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

    execution_live_enabled: bool = Field(
        default=False,
        description="Enable live execution paths (safe default: disabled)",
    )
    execution_live_mode: str = Field(
        default="dry_run",
        description="Live mode: 'dry_run' (default) or 'live'",
    )

    execution_safe_mode: str = Field(
        default="SAFE_MODE",
        description="Execution safety mode: NORMAL|DEGRADED|SAFE_MODE|HALTED (safe-by-default)",
    )
    execution_degraded_reason: str = Field(
        default="",
        description="Optional operator-provided reason used when mode is DEGRADED/SAFE_MODE/HALTED",
    )

    gmo_api_base_url: str = Field(default="", description="GMO API base URL for live execution")
    gmo_request_timeout_seconds: float = Field(default=5.0, description="GMO request timeout")
    live_backoff_seconds: int = Field(default=30, description="Backoff duration after 429/timeout")

    execution_storage_db_path: str = Field(
        default=os.getenv("EXECUTION_STORAGE_DB_PATH", "/tmp/profinaut_execution.sqlite"),
        description="SQLite path for idempotency persistence",
    )

    def get_allowed_symbols(self) -> set[str]:
        if not self.allowed_symbols:
            return set()
        return {s.strip() for s in self.allowed_symbols.split(",") if s.strip()}

    def get_allowed_exchanges(self) -> set[str]:
        if not self.allowed_exchanges:
            return set()
        return {e.strip() for e in self.allowed_exchanges.split(",") if e.strip()}

    def is_live_mode(self) -> bool:
        return self.execution_live_mode.strip().lower() == "live"

    def get_safe_mode(self) -> Literal["NORMAL", "DEGRADED", "SAFE_MODE", "HALTED"]:
        normalized = self.execution_safe_mode.strip().upper()
        if normalized in {"NORMAL", "DEGRADED", "SAFE_MODE", "HALTED"}:
            return normalized
        return "SAFE_MODE"


_settings: Settings | None = None


def get_settings() -> Settings:
    global _settings
    if _settings is None:
        _settings = Settings()
    return _settings
