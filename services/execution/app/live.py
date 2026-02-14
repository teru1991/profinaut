import hashlib
from dataclasses import dataclass

import httpx


class LiveRateLimitError(Exception):
    pass


class LiveTimeoutError(Exception):
    pass


@dataclass(frozen=True)
class PlaceOrderResult:
    order_id: str


class GmoLiveExecutor:
    def __init__(self, *, base_url: str, timeout_seconds: float, api_key: str, api_secret: str):
        self._base_url = base_url.rstrip("/")
        self._timeout = timeout_seconds
        self._api_key = api_key
        self._api_secret = api_secret

    @staticmethod
    def build_client_order_id(idempotency_key: str) -> str:
        digest = hashlib.sha256(idempotency_key.encode("utf-8")).hexdigest()[:24]
        return f"pfn-{digest}"

    def place_order(self, *, symbol: str, side: str, qty: float, order_type: str, limit_price: float | None, client_order_id: str) -> PlaceOrderResult:
        try:
            with httpx.Client(timeout=self._timeout) as client:
                response = client.post(
                    f"{self._base_url}/private/v1/order",
                    json={
                        "symbol": symbol,
                        "side": side,
                        "size": qty,
                        "executionType": order_type,
                        "price": limit_price,
                        "clientOrderId": client_order_id,
                    },
                    headers={"X-API-KEY": self._api_key, "X-API-SECRET": self._api_secret},
                )
        except httpx.TimeoutException as e:
            raise LiveTimeoutError("GMO live order timeout") from e

        if response.status_code == 429:
            raise LiveRateLimitError("GMO live order rate limited")
        response.raise_for_status()
        payload = response.json()
        return PlaceOrderResult(order_id=str(payload.get("orderId", "")))

    def cancel_order(self, *, order_id: str, client_order_id: str) -> None:
        try:
            with httpx.Client(timeout=self._timeout) as client:
                response = client.post(
                    f"{self._base_url}/private/v1/cancel-order",
                    json={"orderId": order_id, "clientOrderId": client_order_id},
                    headers={"X-API-KEY": self._api_key, "X-API-SECRET": self._api_secret},
                )
        except httpx.TimeoutException as e:
            raise LiveTimeoutError("GMO live cancel timeout") from e

        if response.status_code == 429:
            raise LiveRateLimitError("GMO live cancel rate limited")
        response.raise_for_status()
