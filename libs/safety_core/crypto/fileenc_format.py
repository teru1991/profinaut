from __future__ import annotations

import base64
import json
from dataclasses import dataclass
from typing import Any

MAGIC = "UCEL-FILEENC"
VERSION = 1


@dataclass(frozen=True, slots=True)
class FileEncContext:
    path: str
    field: str
    registry_id: str
    scope: str
    version_hint: str | None

    def to_aad_bytes(self) -> bytes:
        obj = {
            "path": self.path,
            "field": self.field,
            "registry_id": self.registry_id,
            "scope": self.scope,
            "version_hint": self.version_hint,
        }
        s = json.dumps(obj, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
        return s.encode("utf-8")


def b64e(b: bytes) -> str:
    return base64.b64encode(b).decode("ascii")


def b64d(s: str) -> bytes:
    return base64.b64decode(s.encode("ascii"))


def dumps_record(obj: dict[str, Any]) -> bytes:
    s = json.dumps(obj, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
    return s.encode("utf-8")


def loads_record(b: bytes) -> dict[str, Any]:
    obj = json.loads(b.decode("utf-8"))
    if not isinstance(obj, dict):
        raise ValueError("record must be object")
    return obj
