from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import requests


class CommandSource:
    def poll_commands(self) -> list[dict[str, Any]]:
        raise NotImplementedError


class HttpCommandSource(CommandSource):
    def __init__(self, pull_url: str, timeout_seconds: float = 5.0) -> None:
        self.pull_url = pull_url
        self.timeout_seconds = timeout_seconds

    def poll_commands(self) -> list[dict[str, Any]]:
        response = requests.get(self.pull_url, timeout=self.timeout_seconds)
        response.raise_for_status()
        payload = response.json()
        if isinstance(payload, list):
            return payload
        return payload.get("items", [])


class FileCommandSource(CommandSource):
    def __init__(self, queue_file: str) -> None:
        self.path = Path(queue_file)

    def poll_commands(self) -> list[dict[str, Any]]:
        if not self.path.exists():
            return []
        raw = json.loads(self.path.read_text(encoding="utf-8") or "[]")
        if not isinstance(raw, list):
            return []
        self.path.write_text("[]", encoding="utf-8")
        return raw
