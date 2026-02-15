from __future__ import annotations

import contextlib
import socket
import threading
import time
from dataclasses import dataclass, field
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import pytest


@dataclass
class ChaosHTTPState:
    requests: list[str] = field(default_factory=list)


class ChaosFaultHandler(BaseHTTPRequestHandler):
    state: ChaosHTTPState

    def do_GET(self) -> None:  # noqa: N802
        self.state.requests.append(self.path)

        if self.path == "/429":
            self.send_response(429)
            self.send_header("Content-Type", "text/plain")
            self.send_header("Retry-After", "2")
            self.end_headers()
            self.wfile.write(b"rate limited")
            return

        if self.path == "/503":
            self.send_response(503)
            self.send_header("Content-Type", "text/plain")
            self.end_headers()
            self.wfile.write(b"service unavailable")
            return

        if self.path == "/timeout":
            time.sleep(1.5)
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.end_headers()
            with contextlib.suppress(BrokenPipeError):
                self.wfile.write(b"too late")
            return

        self.send_response(404)
        self.end_headers()

    def log_message(self, format: str, *args: object) -> None:
        return


@dataclass
class ChaosHTTPServer:
    base_url: str
    state: ChaosHTTPState


@pytest.fixture
def chaos_http_server() -> ChaosHTTPServer:
    state = ChaosHTTPState()
    handler = type("BoundChaosFaultHandler", (ChaosFaultHandler,), {"state": state})

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as probe:
        probe.bind(("127.0.0.1", 0))
        host, port = probe.getsockname()

    server = ThreadingHTTPServer((host, port), handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()

    try:
        yield ChaosHTTPServer(base_url=f"http://{host}:{port}", state=state)
    finally:
        server.shutdown()
        thread.join(timeout=2)
        server.server_close()
