from __future__ import annotations

import json
import logging
import urllib.parse
from abc import ABC, abstractmethod
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any, Callable

from services.marketdata.app.logging import scrub_sensitive_fields
from services.marketdata.app.registry import load_venue_registry
from services.marketdata.app.transport import HttpTransportClient
from services.marketdata.app.ucel_core import Capabilities, CoreError, ErrorCode, Exchange, ExecuteContext, OpName

logger = logging.getLogger("marketdata.gmo_private")


@dataclass(frozen=True)
class ResolvedSecret:
    api_key: str
    api_secret: str
    passphrase: str | None = None


class SecretRefResolver(ABC):
    @abstractmethod
    def resolve(self, secret_ref: str) -> ResolvedSecret:
        raise NotImplementedError


class DictSecretRefResolver(SecretRefResolver):
    def __init__(self, secrets: dict[str, ResolvedSecret]) -> None:
        self._secrets = dict(secrets)

    def resolve(self, secret_ref: str) -> ResolvedSecret:
        value = self._secrets.get(secret_ref)
        if value is None:
            raise CoreError(ErrorCode.MISSING_AUTH, f"secret_ref not found: {secret_ref}")
        return value


@dataclass(frozen=True)
class KeyRef:
    key_id: str
    secret_ref: str
    scope: str
    account_id: str | None = None


@dataclass
class KeyHealth:
    last_success_ms: int | None = None
    last_error_code: str | None = None
    cooldown_until_ms: int = 0
    rate_limited_until_ms: int = 0
    consecutive_failures: int = 0
    exhausted: bool = False


@dataclass(frozen=True)
class FailoverPolicy:
    max_attempts: int = 2
    cooldown_ms: int = 30_000
    respect_retry_after: bool = True
    ban_risk_guard: bool = True
    policy_id: str = "gmo-v114"


class KeyPool:
    def __init__(self, *, keys: list[KeyRef], failover_policy: FailoverPolicy, now_ms_fn: Callable[[], int] | None = None) -> None:
        self._keys = list(keys)
        self._policy = failover_policy
        self._health = {key.key_id: KeyHealth() for key in keys}
        self._now_ms_fn = now_ms_fn or (lambda: int(datetime.now(UTC).timestamp() * 1000))

    @property
    def health(self) -> dict[str, KeyHealth]:
        return self._health

    def select(self, *, required_scope: str) -> KeyRef:
        now_ms = self._now_ms_fn()
        candidates = [k for k in self._keys if k.scope == required_scope]
        for key in candidates:
            state = self._health[key.key_id]
            if state.exhausted:
                continue
            if state.cooldown_until_ms > now_ms or state.rate_limited_until_ms > now_ms:
                continue
            return key
        raise CoreError(ErrorCode.AUTH_FAILED, "all keys exhausted or cooling down", details={"scope": required_scope})

    def mark_success(self, *, key_id: str) -> None:
        state = self._health[key_id]
        state.last_success_ms = self._now_ms_fn()
        state.last_error_code = None
        state.consecutive_failures = 0
        state.cooldown_until_ms = 0

    def mark_failure(self, *, key_id: str, error_code: ErrorCode, retry_after_ms: int | None = None) -> None:
        state = self._health[key_id]
        now_ms = self._now_ms_fn()
        state.last_error_code = error_code.value
        state.consecutive_failures += 1

        if error_code in {ErrorCode.NOT_SUPPORTED, ErrorCode.INVALID_PARAMS}:
            return

        if error_code == ErrorCode.RATE_LIMITED and self._policy.respect_retry_after and retry_after_ms:
            state.rate_limited_until_ms = max(state.rate_limited_until_ms, now_ms + retry_after_ms)

        state.cooldown_until_ms = max(state.cooldown_until_ms, now_ms + self._policy.cooldown_ms)
        if self._policy.ban_risk_guard and state.consecutive_failures >= self._policy.max_attempts:
            state.exhausted = True


class GmoPrivateExecutionAdapter(Exchange):
    def __init__(
        self,
        *,
        http_client: HttpTransportClient | None = None,
        secret_resolver: SecretRefResolver,
        key_pool: KeyPool,
        dry_run_default: bool = True,
        request_fn: Callable[[str, dict[str, Any], str], Any] | None = None,
    ) -> None:
        self._registry = load_venue_registry("gmocoin")
        self._http = http_client or HttpTransportClient()
        self._secret_resolver = secret_resolver
        self._key_pool = key_pool
        self._dry_run_default = dry_run_default
        self._request_fn = request_fn
        self.metrics: dict[str, int] = {
            "ucel_key_selected_total": 0,
            "ucel_key_failover_total": 0,
            "ucel_auth_failed_total": 0,
            "ucel_rate_limited_total": 0,
        }

    def capabilities(self) -> Capabilities:
        return Capabilities(
            venue="gmocoin",
            supported_ops=frozenset(
                {OpName.FETCH_BALANCES, OpName.PLACE_ORDER, OpName.CANCEL_ORDER, OpName.FETCH_ORDERS, OpName.FETCH_FILLS}
            ),
        )

    def _execute_impl(self, op: OpName, params: dict[str, Any], ctx: ExecuteContext) -> Any:
        if "execution" not in ctx.features:
            raise CoreError(ErrorCode.FEATURE_DISABLED, "execution feature disabled")

        if op in {OpName.PLACE_ORDER, OpName.CANCEL_ORDER} and self._dry_run_default and not ctx.live_trading:
            self._audit("gmo_preflight_reject", op, ctx, ErrorCode.DRY_RUN_ONLY, key_id=None, scope="trade")
            raise CoreError(ErrorCode.DRY_RUN_ONLY, "live-trading override required")

        scope = "trade"
        key = self._key_pool.select(required_scope=scope)
        self.metrics["ucel_key_selected_total"] += 1

        try:
            secret = self._secret_resolver.resolve(key.secret_ref)
        except CoreError:
            self.metrics["ucel_auth_failed_total"] += 1
            self._audit("gmo_preflight_reject", op, ctx, ErrorCode.MISSING_AUTH, key_id=key.key_id, scope=scope)
            raise

        path_map = self._paths_by_op()
        endpoint = path_map.get(op)
        if endpoint is None:
            raise CoreError(ErrorCode.NOT_SUPPORTED, f"op unsupported: {op.value}")

        if self._request_fn is not None:
            response = self._request_fn(endpoint["id"], params, secret.api_key)
            self._key_pool.mark_success(key_id=key.key_id)
            self._audit("gmo_private_ok", op, ctx, None, key_id=key.key_id, scope=scope)
            return response

        base_url = str(endpoint["base_url"]).rstrip("/")
        url = f"{base_url}{endpoint['path']}"
        method = str(endpoint["method"])
        body = json.dumps(params).encode("utf-8") if method.upper() == "POST" else None
        if method.upper() == "GET" and params:
            url += f"?{urllib.parse.urlencode(params)}"

        try:
            payload = self._http.request(
                op_name=op.value,
                method=method,
                url=url,
                timeout_seconds=5,
                body=body,
                headers={"X-API-KEY": secret.api_key},
                is_private=True,
                auth_header="signed",
            )
        except Exception as exc:  # noqa: BLE001
            code = ErrorCode.RATE_LIMITED if "429" in str(exc) else ErrorCode.AUTH_FAILED
            if code == ErrorCode.RATE_LIMITED:
                self.metrics["ucel_rate_limited_total"] += 1
            self.metrics["ucel_key_failover_total"] += 1
            self._key_pool.mark_failure(key_id=key.key_id, error_code=code)
            self._audit("gmo_private_error", op, ctx, code, key_id=key.key_id, scope=scope)
            raise

        self._key_pool.mark_success(key_id=key.key_id)
        self._audit("gmo_private_ok", op, ctx, None, key_id=key.key_id, scope=scope)
        return json.loads(payload.decode("utf-8"))

    def _paths_by_op(self) -> dict[OpName, dict[str, Any]]:
        catalog = json.loads(Path(self._registry.catalog_path).read_text(encoding="utf-8"))
        by_id = {row["id"]: row for row in catalog["rest_endpoints"]}
        return {
            OpName.FETCH_BALANCES: by_id["crypto.private.rest.assets.get"],
            OpName.PLACE_ORDER: by_id["crypto.private.rest.order.post"],
            OpName.CANCEL_ORDER: by_id["crypto.private.rest.cancelorder.post"],
            OpName.FETCH_ORDERS: by_id["crypto.private.rest.activeorders.get"],
            OpName.FETCH_FILLS: by_id["crypto.private.rest.executions.get"],
        }

    def _audit(
        self,
        event: str,
        op: OpName,
        ctx: ExecuteContext,
        error_code: ErrorCode | None,
        *,
        key_id: str | None,
        scope: str,
    ) -> None:
        fields = scrub_sensitive_fields(
            {
                "venue": "gmocoin",
                "event": event,
                "op": op.value,
                "error_code": error_code.value if error_code else None,
                "request_id": ctx.request_id,
                "trace_id": ctx.trace_id,
                "run_id": ctx.run_id,
                "key_id": key_id,
                "key_scope": scope,
                "policy_id": ctx.policy.policy_id if ctx.policy else None,
                "actor_id": ctx.actor_id,
            }
        )
        logger.info("gmo_private_audit %s", fields)
