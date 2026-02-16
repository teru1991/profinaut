from __future__ import annotations

import hashlib
import json
from typing import Any


def canonical_json_bytes(payload_json: Any) -> bytes:
    """Return deterministic JSON bytes (sorted keys + minimal separators + utf-8)."""
    canonical = json.dumps(payload_json, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    return canonical.encode("utf-8")


def compute_payload_hash(payload_json: Any) -> str:
    """Compute deterministic SHA256 hash for JSON-like payloads."""
    return hashlib.sha256(canonical_json_bytes(payload_json)).hexdigest()
