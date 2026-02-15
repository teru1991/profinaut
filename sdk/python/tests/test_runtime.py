from datetime import datetime, timedelta, timezone

import pytest

from profinaut_agent.agent import AgentConfig, AgentRuntime, RuntimeSafetyState, SafeModeBlockedError
from profinaut_agent.models import CommandType
from profinaut_agent.source import CommandSource


class FakeSource(CommandSource):
    def __init__(self, items):
        self.items = items

    def poll_commands(self):
        items = self.items
        self.items = []
        return items


class FakeClient:
    def __init__(self):
        self.heartbeats = []
        self.acks = []
        self.orders = []

    def send_heartbeat(self, heartbeat):
        self.heartbeats.append(heartbeat)

    def send_ack(self, ack):
        self.acks.append(ack)

    def place_order(self, order):
        self.orders.append(order)


class FailingClient(FakeClient):
    def send_heartbeat(self, heartbeat):
        raise TimeoutError("control plane timeout")


def test_runtime_processes_valid_command_and_acks() -> None:
    now = datetime.now(timezone.utc)
    source = FakeSource(
        [
            {
                "command_id": "11111111-1111-1111-1111-111111111111",
                "instance_id": "inst-1",
                "command_type": "SAFE_MODE",
                "issued_at": now.isoformat(),
                "expires_at": (now + timedelta(seconds=30)).isoformat(),
                "payload": {},
            }
        ]
    )
    client = FakeClient()
    config = AgentConfig(
        control_plane_url="http://localhost:8000",
        command_pull_url=None,
        command_file=None,
        instance_id="inst-1",
        bot_id="bot-1",
        runtime_mode="PAPER",
        exchange="BINANCE",
        symbol="BTCUSDT",
        version="0.1.0",
        heartbeat_interval_seconds=30,
        deadman_stale_seconds=5,
        deadman_action="FLATTEN",
    )

    runtime = AgentRuntime(config=config, source=source, client=client)
    runtime.run_once()

    assert len(client.heartbeats) == 1
    assert len(client.acks) == 1
    assert client.acks[0].status.value == "COMPLETED"
    assert runtime.processor.state == CommandType.SAFE_MODE


def test_runtime_enters_safe_mode_on_stale_control_plane_failure() -> None:
    source = FakeSource([])
    client = FailingClient()
    config = AgentConfig(
        control_plane_url="http://localhost:8000",
        command_pull_url=None,
        command_file=None,
        instance_id="inst-1",
        bot_id="bot-1",
        runtime_mode="PAPER",
        exchange="BINANCE",
        symbol="BTCUSDT",
        version="0.1.0",
        heartbeat_interval_seconds=30,
        deadman_stale_seconds=0,
        deadman_action="HALT",
    )

    runtime = AgentRuntime(config=config, source=source, client=client)
    runtime.run_once()

    assert runtime.safety_state == RuntimeSafetyState.SAFE_MODE
    assert runtime.processor.state == CommandType.STOP


def test_place_order_blocked_and_not_sent_when_safe_mode() -> None:
    source = FakeSource([])
    client = FakeClient()
    config = AgentConfig(
        control_plane_url="http://localhost:8000",
        command_pull_url=None,
        command_file=None,
        instance_id="inst-1",
        bot_id="bot-1",
        runtime_mode="PAPER",
        exchange="BINANCE",
        symbol="BTCUSDT",
        version="0.1.0",
        heartbeat_interval_seconds=30,
        deadman_stale_seconds=5,
        deadman_action="FLATTEN",
    )

    runtime = AgentRuntime(config=config, source=source, client=client)
    runtime.safety_state = RuntimeSafetyState.SAFE_MODE

    with pytest.raises(SafeModeBlockedError):
        runtime.place_order({"symbol": "BTCUSDT", "qty": 1})

    assert client.orders == []
