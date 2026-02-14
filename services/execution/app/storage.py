import threading
import uuid
from datetime import datetime, timezone

from .schemas import Order, OrderIntent


class OrderStorage:
    """Thread-safe in-memory storage for orders with idempotency tracking"""

    def __init__(self):
        self._lock = threading.Lock()
        self._orders: dict[str, Order] = {}
        self._idempotency_map: dict[str, str] = {}  # idempotency_key -> order_id

    def create_order(self, intent: OrderIntent) -> Order | None:
        """
        Create an order from an intent.
        Returns Order if successful, None if idempotency_key already exists.
        """
        with self._lock:
            # Check for duplicate idempotency_key
            if intent.idempotency_key in self._idempotency_map:
                return None

            # Generate order_id and create order
            order_id = f"paper-{uuid.uuid4()}"
            order = Order(
                order_id=order_id,
                status="NEW",
                accepted_ts_utc=datetime.now(timezone.utc),
                exchange=intent.exchange,
                symbol=intent.symbol,
                side=intent.side,
                qty=intent.qty,
                filled_qty=0.0,
            )

            # Store order and idempotency mapping
            self._orders[order_id] = order
            self._idempotency_map[intent.idempotency_key] = order_id

            return order

    def get_order(self, order_id: str) -> Order | None:
        """Get an order by order_id"""
        with self._lock:
            return self._orders.get(order_id)

    def get_order_by_idempotency_key(self, idempotency_key: str) -> Order | None:
        """Get an order by idempotency_key"""
        with self._lock:
            order_id = self._idempotency_map.get(idempotency_key)
            if order_id:
                return self._orders.get(order_id)
            return None


# Global storage instance
_storage: OrderStorage | None = None


def get_storage() -> OrderStorage:
    global _storage
    if _storage is None:
        _storage = OrderStorage()
    return _storage
