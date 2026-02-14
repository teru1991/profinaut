from functools import lru_cache

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", extra="ignore")

    database_url: str = "postgresql://profinaut:profinaut@postgres:5432/profinaut"
    admin_token: str = "change-me-local-admin-token"
    discord_webhook_url: str | None = None
    marketdata_base_url: str = "http://127.0.0.1:8081"
    marketdata_timeout_seconds: float = 2.0


@lru_cache
def get_settings() -> Settings:
    return Settings()
