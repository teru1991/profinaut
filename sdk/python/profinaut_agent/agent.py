from __future__ import annotations

import logging
import time
from dataclasses import dataclass
from datetime import datetime, timezone

from .client import ControlPlaneClient, build_heartbeat
from .deadman import DeadmanSwitch
from .models import Command, CommandType
from .processor import CommandProcessor
from .source import CommandSource


@dataclass(slots=True)
class AgentConfig:
    control_plane_url: str
    command_pull_url: str | None
    command_file: str | None
    instance_id: str
    bot_id: str
    runtime_mode: str
    exchange: str
    symbol: str
    version: str
    heartbeat_interval_seconds: int = 30
    deadman_timeout_seconds: int = 90
    deadman_action: str = "SAFE_MODE"


class AgentRuntime:
    def __init__(self, config: AgentConfig, source: CommandSource, client: ControlPlaneClient) -> None:
        self.config = config
        self.source = source
        self.client = client
        self.processor = CommandProcessor(mode=config.runtime_mode)
        self.deadman = DeadmanSwitch(
            timeout_seconds=config.deadman_timeout_seconds,
            fallback_action=config.deadman_action,
        )

    def run_once(self) -> None:
        now = datetime.now(timezone.utc)
        try:
            hb = build_heartbeat(
                instance_id=self.config.instance_id,
                bot_id=self.config.bot_id,
                runtime_mode=self.processor.mode,
                exchange=self.config.exchange,
                symbol=self.config.symbol,
                version=self.config.version,
            )
            self.client.send_heartbeat(hb)
            self.deadman.register_success()
        except Exception as exc:  # noqa: BLE001
            logging.warning("heartbeat failure: %s", exc)
            self._apply_deadman(now)

        try:
            command_dicts = self.source.poll_commands()
            self.deadman.register_success()
        except Exception as exc:  # noqa: BLE001
            logging.warning("command polling failure: %s", exc)
            self._apply_deadman(now)
            return

        for raw in command_dicts:
            command = Command.from_dict(raw)
            ack = self.processor.process(command, now=datetime.now(timezone.utc))
            try:
                self.client.send_ack(ack)
                self.deadman.register_success()
            except Exception as exc:  # noqa: BLE001
                logging.warning("ack publish failure: %s", exc)
                self._apply_deadman(datetime.now(timezone.utc))

    def run_forever(self) -> None:
        while True:
            self.run_once()
            time.sleep(self.config.heartbeat_interval_seconds)

    def _apply_deadman(self, now: datetime) -> None:
        action = self.deadman.register_failure(now)
        if action is None:
            return
        fallback = Command(
            command_id=f"deadman-{int(now.timestamp())}",
            instance_id=self.config.instance_id,
            command_type=CommandType(action),
            issued_at=now,
            expires_at=now,
            payload={},
        )
        self.processor._apply_command(fallback)
        logging.error("dead-man switch triggered: %s", action)
