from __future__ import annotations

import asyncio
import importlib
import importlib.util

import httpx
import pytest


def _scenario_log(name: str, expected: str) -> None:
    print(f"[chaos] scenario={name} expected={expected}")


def test_http_429_with_retry_after(chaos_http_server) -> None:
    _scenario_log("http-429", "status=429 + Retry-After header")
    response = httpx.get(f"{chaos_http_server.base_url}/429", timeout=2.0)

    assert response.status_code == 429
    assert response.headers.get("Retry-After") == "2"
    assert "/429" in chaos_http_server.state.requests


def test_http_503_service_unavailable(chaos_http_server) -> None:
    _scenario_log("http-503", "status=503")
    response = httpx.get(f"{chaos_http_server.base_url}/503", timeout=2.0)

    assert response.status_code == 503
    assert response.text == "service unavailable"
    assert "/503" in chaos_http_server.state.requests


def test_timeout_fault(chaos_http_server) -> None:
    _scenario_log("http-timeout", "client timeout raised")
    with pytest.raises(httpx.TimeoutException):
        httpx.get(f"{chaos_http_server.base_url}/timeout", timeout=0.2)

    assert "/timeout" in chaos_http_server.state.requests


def test_websocket_drop(free_tcp_port: int) -> None:
    if not importlib.util.find_spec("websockets"):
        pytest.skip("websockets dependency is not installed; skipping WS drop scenario")

    websockets = importlib.import_module("websockets")
    _scenario_log("ws-drop", "connection closes unexpectedly and client detects close")

    async def run_scenario() -> None:
        async def drop_handler(ws):
            await ws.send("connected")
            ws.transport.close()

        async with websockets.serve(drop_handler, "127.0.0.1", free_tcp_port):
            async with websockets.connect(f"ws://127.0.0.1:{free_tcp_port}") as client:
                first_message = await client.recv()
                assert first_message == "connected"
                with pytest.raises(websockets.exceptions.ConnectionClosed):
                    await client.recv()

    asyncio.run(run_scenario())
