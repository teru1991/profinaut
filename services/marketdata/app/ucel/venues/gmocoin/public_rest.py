from __future__ import annotations

import json
import urllib.parse
from typing import Any, Callable

from services.marketdata.app.transport import HttpTransportClient
from services.marketdata.app.ucel_core import CoreError, ErrorCode, OpName


class GmoPublicRestClient:
    def __init__(
        self,
        *,
        timeout_seconds: float,
        catalog_scope: dict[OpName, dict[str, Any]],
        http_client: HttpTransportClient | None = None,
        request_fn: Callable[[str, str], dict[str, Any]] | None = None,
    ) -> None:
        self._timeout_seconds = timeout_seconds
        self._scope = catalog_scope
        self._http = http_client or HttpTransportClient()
        self._request_fn = request_fn

    @staticmethod
    def assert_public_only(ctx_has_auth: bool, secret_ref: str | None) -> None:
        if ctx_has_auth or secret_ref:
            raise CoreError(ErrorCode.INVALID_PARAMS, "public marketdata flow must not use auth/secrets")

    def fetch(self, op: OpName, query: dict[str, str]) -> dict[str, Any]:
        endpoint = self._scope.get(op)
        if endpoint is None:
            raise CoreError(ErrorCode.NOT_SUPPORTED, f"catalog does not define op={op.value}")
        if self._request_fn is not None:
            return self._request_fn(endpoint["id"], urllib.parse.urlencode(query))

        url = f"{str(endpoint['base_url']).rstrip('/')}{endpoint['path']}?{urllib.parse.urlencode(query)}"
        raw = self._http.request(op_name=op.value, method=str(endpoint["method"]), url=url, timeout_seconds=self._timeout_seconds)
        return json.loads(raw.decode("utf-8"))
