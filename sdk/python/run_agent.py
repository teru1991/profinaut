from __future__ import annotations

import os

from profinaut_agent.agent import AgentConfig, AgentRuntime
from profinaut_agent.client import ControlPlaneClient
from profinaut_agent.source import FileCommandSource, HttpCommandSource


def get_env(name: str, default: str) -> str:
    return os.getenv(name, default)


def main() -> None:
    config = AgentConfig(
        control_plane_url=get_env("CONTROL_PLANE_URL", "http://localhost:8000"),
        command_pull_url=os.getenv("COMMAND_PULL_URL"),
        command_file=os.getenv("COMMAND_FILE", "sdk/python/commands.json"),
        instance_id=get_env("INSTANCE_ID", "inst-local-1"),
        bot_id=get_env("BOT_ID", "bot-local-1"),
        runtime_mode=get_env("RUNTIME_MODE", "PAPER"),
        exchange=get_env("EXCHANGE", "BINANCE"),
        symbol=get_env("SYMBOL", "BTCUSDT"),
        version=get_env("AGENT_VERSION", "0.1.0"),
        heartbeat_interval_seconds=int(get_env("HEARTBEAT_INTERVAL_SECONDS", "30")),
        deadman_stale_seconds=int(get_env("DEADMAN_STALE_SECONDS", "90")),
        deadman_action=get_env("DEADMAN_ACTION", "FLATTEN"),
    )

    pull_url = config.command_pull_url or f"{config.control_plane_url.rstrip("/")}/instances/{config.instance_id}/commands/pending"
    source = HttpCommandSource(pull_url) if pull_url else FileCommandSource(config.command_file or "sdk/python/commands.json")
    timeout_seconds = float(get_env("CONTROL_PLANE_TIMEOUT_SECONDS", "5"))
    client = ControlPlaneClient(config.control_plane_url, timeout_seconds=timeout_seconds)

    runtime = AgentRuntime(config=config, source=source, client=client)
    runtime.run_forever()


if __name__ == "__main__":
    main()
