import sqlite3
import threading
import uuid
from datetime import datetime, timezone

from .schemas import Fill, Order, OrderIntent


class OrderStorage:
    """Thread-safe storage with SQLite-persisted idempotency mapping."""

    def __init__(self, db_path: str):
        self._lock = threading.Lock()
        self._orders: dict[str, Order] = {}
        self._fills: list[Fill] = []
        self._idempotency_map: dict[str, str] = {}  # idempotency_key -> order_id
        self._client_order_id_map: dict[str, str] = {}  # idempotency_key -> client_order_id
        self._order_client_map: dict[str, str] = {}  # order_id -> client_order_id

        self._conn = sqlite3.connect(db_path, check_same_thread=False)
        self._conn.execute("PRAGMA journal_mode=WAL")
        self._conn.execute(
            """
            CREATE TABLE IF NOT EXISTS idempotency_map (
                idempotency_key TEXT PRIMARY KEY,
                order_id TEXT NOT NULL,
                client_order_id TEXT
            )
            """
        )
        self._conn.commit()
        self._load_persisted_idempotency()

    def _load_persisted_idempotency(self) -> None:
        rows = self._conn.execute("SELECT idempotency_key, order_id, client_order_id FROM idempotency_map").fetchall()
        for idempotency_key, order_id, client_order_id in rows:
            self._idempotency_map[str(idempotency_key)] = str(order_id)
            if client_order_id:
                self._client_order_id_map[str(idempotency_key)] = str(client_order_id)
                self._order_client_map[str(order_id)] = str(client_order_id)

    def _persist_idempotency(self, idempotency_key: str, order_id: str, client_order_id: str | None) -> bool:
        try:
            self._conn.execute(
                "INSERT INTO idempotency_map(idempotency_key, order_id, client_order_id) VALUES (?, ?, ?)",
                (idempotency_key, order_id, client_order_id),
            )
            self._conn.commit()
            return True
        except sqlite3.IntegrityError:
            return False

    def create_order(
        self,
        intent: OrderIntent,
        *,
        order_id: str | None = None,
        client_order_id: str | None = None,
    ) -> Order | None:
        with self._lock:
            if intent.idempotency_key in self._idempotency_map:
                return None

            order_id = order_id or f"paper-{uuid.uuid4()}"
            if not self._persist_idempotency(intent.idempotency_key, order_id, client_order_id):
                # race/restart-safe duplicate detected from persisted table
                self._idempotency_map[intent.idempotency_key] = order_id
                return None

            order = Order(
                order_id=order_id,
                status="ACCEPTED",
                accepted_ts_utc=datetime.now(timezone.utc),
                exchange=intent.exchange,
                symbol=intent.symbol,
                side=intent.side,
                qty=intent.qty,
                filled_qty=0.0,
            )

            self._orders[order_id] = order
            self._idempotency_map[intent.idempotency_key] = order_id
            if client_order_id is not None:
                self._client_order_id_map[intent.idempotency_key] = client_order_id
                self._order_client_map[order_id] = client_order_id

            return order

    def get_order(self, order_id: str) -> Order | None:
        with self._lock:
            return self._orders.get(order_id)

    def get_order_by_idempotency_key(self, idempotency_key: str) -> Order | None:
        with self._lock:
            order_id = self._idempotency_map.get(idempotency_key)
            if not order_id:
                return None
            return self._orders.get(order_id)

    def get_client_order_id_by_idempotency_key(self, idempotency_key: str) -> str | None:
        with self._lock:
            return self._client_order_id_map.get(idempotency_key)

    def get_client_order_id_by_order_id(self, order_id: str) -> str | None:
        with self._lock:
            return self._order_client_map.get(order_id)

    def cancel_order(self, order_id: str) -> Order | None:
        with self._lock:
            order = self._orders.get(order_id)
            if order is None:
                return None
            if order.status in {"FILLED", "REJECTED"}:
                return order
            updated = order.model_copy(update={"status": "CANCELED"})
            self._orders[order_id] = updated
            return updated

    def fill_order(self, order_id: str) -> Order | None:
        with self._lock:
            order = self._orders.get(order_id)
            if order is None:
                return None
            if order.status in {"CANCELED", "REJECTED"}:
                return order
            updated = order.model_copy(update={"status": "FILLED", "filled_qty": order.qty})
            self._orders[order_id] = updated
            fill = Fill(
                fill_id=f"fill-{uuid.uuid4()}",
                order_id=order.order_id,
                symbol=order.symbol,
                side=order.side,
                qty=order.qty,
                ts_utc=datetime.now(timezone.utc),
            )
            self._fills.append(fill)
            return updated

    def reject_order(self, order_id: str) -> Order | None:
        with self._lock:
            order = self._orders.get(order_id)
            if order is None:
                return None
            if order.status in {"FILLED", "CANCELED"}:
                return order
            updated = order.model_copy(update={"status": "REJECTED"})
            self._orders[order_id] = updated
            return updated

    def list_orders(self, *, page: int, page_size: int) -> tuple[list[Order], int]:
        with self._lock:
            rows = sorted(self._orders.values(), key=lambda o: o.accepted_ts_utc, reverse=True)
            total = len(rows)
            start = (page - 1) * page_size
            end = start + page_size
            return rows[start:end], total

    def list_fills(self, *, page: int, page_size: int) -> tuple[list[Fill], int]:
        with self._lock:
            rows = sorted(self._fills, key=lambda f: f.ts_utc, reverse=True)
            total = len(rows)
            start = (page - 1) * page_size
            end = start + page_size
            return rows[start:end], total


_storage: OrderStorage | None = None


def get_storage() -> OrderStorage:
    global _storage
    if _storage is None:
        from .config import get_settings

        _storage = OrderStorage(db_path=get_settings().execution_storage_db_path)
    return _storage
