from __future__ import annotations

import argparse
import asyncio
import os
from datetime import UTC, datetime
from typing import Any

from fastapi import FastAPI, WebSocket
from fastapi.responses import JSONResponse
import uvicorn

app = FastAPI(title="fake-gmo", version="0.1")


def _now_ts() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


@app.get("/public/v1/ticker")
def ticker(symbol: str = "BTC_JPY") -> JSONResponse:
    return JSONResponse(
        status_code=200,
        content={
            "status": 0,
            "data": [
                {
                    "symbol": symbol,
                    "timestamp": _now_ts(),
                    "bid": "100.0",
                    "ask": "101.0",
                    "last": "100.5",
                }
            ],
        },
    )


@app.get("/public/v1/klines")
def klines(symbol: str = "BTC_JPY", interval: str = "1min", page: int = 1) -> JSONResponse:
    base = "2026-02-16T00:00:00Z"
    return JSONResponse(
        status_code=200,
        content={
            "status": 0,
            "data": [
                {
                    "openTime": base,
                    "open": "100.0",
                    "high": "102.0",
                    "low": "99.0",
                    "close": "101.0",
                    "volume": "10",
                    "is_final": True,
                    "symbol": symbol,
                    "interval": interval,
                    "page": page,
                }
            ],
        },
    )


@app.websocket("/ws/public/v1")
async def ws_public(websocket: WebSocket) -> None:
    await websocket.accept()
    # consume subscribe frames
    subscribed = set()
    for _ in range(3):
        msg = await websocket.receive_json()
        if isinstance(msg, dict) and msg.get("command") == "subscribe":
            ch = str(msg.get("channel") or "")
            if ch:
                subscribed.add(ch)

    if "orderbooks" in subscribed:
        await websocket.send_json(
            {
                "channel": "orderbooks",
                "symbol": "BTC_JPY",
                "type": "snapshot",
                "sequence": 1,
                "bids": [{"price": "100", "size": "1"}],
                "asks": [{"price": "101", "size": "1"}],
            }
        )
        await asyncio.sleep(0.05)

        next_seq = 3 if os.getenv("FAKE_GMO_GAP", "0") == "1" else 2
        await websocket.send_json(
            {
                "channel": "orderbooks",
                "symbol": "BTC_JPY",
                "type": "delta",
                "sequence": next_seq,
                "changes": {
                    "bids": [{"price": "100.5", "size": "1.1"}],
                    "asks": [],
                },
            }
        )

    await asyncio.sleep(2)


def main() -> int:
    parser = argparse.ArgumentParser(description="Run fake GMO REST+WS server")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=19091)
    args = parser.parse_args()
    uvicorn.run(app, host=args.host, port=args.port, log_level="warning")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
