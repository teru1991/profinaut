from __future__ import annotations

import logging
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from enum import StrEnum
from typing import Any

from .client import ControlPlaneClient, build_heartbeat
from .deadman import DeadmanSwitch, DeadmanTransition
from .models import Command, CommandType
from .processor import CommandProcessor
from .source import CommandSource


class RuntimeSafetyState(StrEnum):
    NORMAL = "NORMAL"
    SAFE_MODE = "SAFE_MODE"


class SafeModeBlockedError(RuntimeError):
    pass


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
    deadman_stale_seconds: int = 90
    deadman_action: str = "FLATTEN"


class AgentRuntime:
    def __init__(self, config: AgentConfig, source: CommandSource, client: ControlPlaneClient) -> None:
        self.config = config
        self.source = source
        self.client = client
        self.processor = CommandProcessor(mode=config.runtime_mode)
        self.deadman = DeadmanSwitch(
            stale_seconds=config.deadman_stale_seconds,
        )
        self.safety_state = RuntimeSafetyState.NORMAL

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
            self.deadman.register_success(now)
        except Exception as exc:  # noqa: BLE001
            logging.warning("heartbeat failure: %s", exc)
            self._apply_deadman(now, reason_code="HEARTBEAT_FAILURE")

        try:
            command_dicts = self.source.poll_commands()
            self.deadman.register_success(now)
        except Exception as exc:  # noqa: BLE001
            logging.warning("command polling failure: %s", exc)
            self._apply_deadman(now, reason_code="COMMAND_POLL_FAILURE")
            return

        for raw in command_dicts:
            command = Command.from_dict(raw)
            ack = self.processor.process(command, now=datetime.now(timezone.utc))
            try:
                self.client.send_ack(ack)
                self.deadman.register_success(datetime.now(timezone.utc))
            except Exception as exc:  # noqa: BLE001
                logging.warning("ack publish failure: %s", exc)
                self._apply_deadman(datetime.now(timezone.utc), reason_code="ACK_PUBLISH_FAILURE")

    def run_forever(self) -> None:
        while True:
            self.run_once()
            time.sleep(self.config.heartbeat_interval_seconds)

    def place_order(self, order: dict[str, Any]) -> None:
        self._ensure_ordering_allowed()
        self.client.place_order(order)

    def apply_safe_action(self, action: str, now: datetime | None = None) -> None:
        ts = now or datetime.now(timezone.utc)
        command_type = self._resolve_safe_action(action)
        self.processor._apply_command(
            Command(
                command_id=f"safe-action-{action.lower()}-{int(ts.timestamp())}",
                instance_id=self.config.instance_id,
                command_type=command_type,
                issued_at=ts,
                expires_at=ts,
                payload={},
            )
        )

    def _ensure_ordering_allowed(self) -> None:
        if self.safety_state == RuntimeSafetyState.SAFE_MODE:
            raise SafeModeBlockedError("SAFE_MODE active: order placement blocked")

    def _resolve_safe_action(self, action: str) -> CommandType:
        normalized = action.upper()
        if normalized == "FLATTEN":
            return CommandType.FLATTEN
        if normalized == "HALT":
            return CommandType.STOP
        raise ValueError("DEADMAN_ACTION must be FLATTEN or HALT")

    def _apply_deadman(self, now: datetime, reason_code: str) -> None:
        transition = self.deadman.register_failure(now, reason_code=reason_code)
        if transition is None:
            return

        self.safety_state = RuntimeSafetyState.SAFE_MODE
        self.processor._apply_command(
            Command(
                command_id=f"deadman-safe-mode-{int(now.timestamp())}",
                instance_id=self.config.instance_id,
                command_type=CommandType.SAFE_MODE,
                issued_at=now,
                expires_at=now,
                payload={"reason_code": transition.reason_code},
            )
        )

        self.apply_safe_action(self.config.deadman_action, now=now)
        self._log_safe_mode_transition(transition)

    def _log_safe_mode_transition(self, transition: DeadmanTransition) -> None:
        logging.error(
            "entered SAFE_MODE reason_code=%s stale_for_seconds=%s consecutive_failures=%s action=%s",
            transition.reason_code,
            transition.stale_for_seconds,
            transition.consecutive_failures,
            self.config.deadman_action,
        )
