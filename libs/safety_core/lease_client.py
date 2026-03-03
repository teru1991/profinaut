from __future__ import annotations

import json
import urllib.error
import urllib.parse
import urllib.request
from typing import Any


class SafetyLeaseClient:
    def __init__(self, base_url: str, timeout_seconds: float = 1.5) -> None:
        self._base_url = base_url.rstrip("/")
        self._timeout = timeout_seconds

    def _request(self, method: str, path: str, payload: dict[str, Any] | None = None) -> dict[str, Any]:
        data = None if payload is None else json.dumps(payload).encode("utf-8")
        req = urllib.request.Request(
            f"{self._base_url}{path}",
            data=data,
            method=method,
            headers={"accept": "application/json", "content-type": "application/json"},
        )
        try:
            with urllib.request.urlopen(req, timeout=self._timeout) as response:
                raw = response.read().decode("utf-8")
                return json.loads(raw) if raw else {}
        except (urllib.error.HTTPError, urllib.error.URLError, TimeoutError) as exc:
            raise ConnectionError(f"safety lease api request failed: {method} {path}") from exc

    def issue_lease(self, payload: dict[str, Any]) -> dict[str, Any]:
        return self._request("POST", "/safety/lease/issue", payload)

    def renew_lease(self, payload: dict[str, Any]) -> dict[str, Any]:
        return self._request("POST", "/safety/lease/renew", payload)

    def get_status(self, subject_kind: str, subject_id: str) -> dict[str, Any]:
        query = urllib.parse.urlencode({"subject_kind": subject_kind, "subject_id": subject_id})
        return self._request("GET", f"/safety/lease/status?{query}")
