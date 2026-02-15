import json

from bots.simple_mm import main


def test_deadman_latches_and_recovers() -> None:
    dms = main.DeadmanSwitch(timeout_seconds=5, recovery_successes_required=2)

    safe, reason = dms.note_failure(now_monotonic=10)
    assert safe is False
    assert reason is None

    safe, reason = dms.note_failure(now_monotonic=16)
    assert safe is True
    assert reason == "CONTROLPLANE_UNREACHABLE"

    safe, reason = dms.note_success()
    assert safe is True
    assert reason is None

    safe, reason = dms.note_success()
    assert safe is False
    assert reason == "DEADMAN_RECOVERED"


def test_safe_mode_blocks_order_submission(monkeypatch) -> None:
    calls: list[str] = []

    monkeypatch.setenv("SAFE_MODE", "true")
    monkeypatch.setenv("BOT_MAX_LOOPS", "1")

    monkeypatch.setattr(main, "fetch_controlplane_capabilities", lambda *_: {"status": "ok"})
    monkeypatch.setattr(main, "fetch_ticker", lambda *_: {"symbol": "BTC_JPY", "stale": False})
    monkeypatch.setattr(main, "fetch_execution_capabilities", lambda *_: {"status": "ok"})

    def _submit(*_args, **_kwargs):
        calls.append("submitted")
        return {"order_id": "x", "filled_qty": 0}

    monkeypatch.setattr(main, "submit_order_intent", _submit)

    rc = main.run()

    assert rc == 0
    assert calls == []


def test_marketdata_degraded_blocks_order_submission(monkeypatch) -> None:
    calls: list[str] = []

    monkeypatch.setenv("SAFE_MODE", "false")
    monkeypatch.setenv("EXECUTION_MODE", "paper")
    monkeypatch.setenv("BOT_MAX_LOOPS", "1")

    monkeypatch.setattr(main, "fetch_controlplane_capabilities", lambda *_: {"status": "ok"})
    monkeypatch.setattr(
        main,
        "fetch_ticker",
        lambda *_: {"symbol": "BTC_JPY", "stale": True, "degraded_reason": "STALE_TICKER", "quality": {"status": "STALE"}},
    )
    monkeypatch.setattr(main, "fetch_execution_capabilities", lambda *_: {"status": "ok"})

    def _submit(*_args, **_kwargs):
        calls.append("submitted")
        return {"order_id": "x", "filled_qty": 0}

    monkeypatch.setattr(main, "submit_order_intent", _submit)

    rc = main.run()

    assert rc == 0
    assert calls == []


def test_deadman_safe_mode_logs_skip_and_blocks_new_orders(monkeypatch, capsys) -> None:
    calls: list[str] = []

    monkeypatch.setenv("SAFE_MODE", "false")
    monkeypatch.setenv("BOT_MAX_LOOPS", "2")
    monkeypatch.setenv("DEADMAN_TIMEOUT_SECONDS", "0")
    monkeypatch.setenv("BOT_LOOP_INTERVAL_SECONDS", "0")

    def _controlplane(*_):
        raise main.BotError("controlplane down")

    monkeypatch.setattr(main, "fetch_controlplane_capabilities", _controlplane)

    def _submit(*_args, **_kwargs):
        calls.append("submitted")
        return {"order_id": "x", "filled_qty": 0}

    monkeypatch.setattr(main, "submit_order_intent", _submit)

    rc = main.run()
    assert rc == 0
    assert calls == []

    events = [json.loads(line) for line in capsys.readouterr().out.strip().splitlines() if line.strip()]
    blocked = [e for e in events if e.get("event") == "new_order_blocked"]
    assert blocked
    assert any(e.get("reason") == "SAFE_MODE" and e.get("decision") == "SKIP_ORDER" for e in blocked)


def test_live_requires_explicit_enable(monkeypatch) -> None:
    calls: list[str] = []

    monkeypatch.setenv("SAFE_MODE", "false")
    monkeypatch.setenv("EXECUTION_MODE", "live")
    monkeypatch.setenv("EXECUTION_LIVE_ENABLED", "false")
    monkeypatch.setenv("BOT_MAX_LOOPS", "1")

    monkeypatch.setattr(main, "fetch_controlplane_capabilities", lambda *_: {"status": "ok"})
    monkeypatch.setattr(main, "fetch_ticker", lambda *_: {"symbol": "BTC_JPY", "stale": False})
    monkeypatch.setattr(main, "fetch_execution_capabilities", lambda *_: {"status": "ok"})

    def _submit(*_args, **_kwargs):
        calls.append("submitted")
        return {"order_id": "x", "filled_qty": 0}

    monkeypatch.setattr(main, "submit_order_intent", _submit)

    rc = main.run()

    assert rc == 0
    assert calls == []


def test_paper_e2e_submits_order_when_healthy(monkeypatch) -> None:
    calls: list[dict] = []

    monkeypatch.setenv("SAFE_MODE", "false")
    monkeypatch.setenv("EXECUTION_MODE", "paper")
    monkeypatch.setenv("BOT_MAX_LOOPS", "1")
    monkeypatch.setenv("DEADMAN_TIMEOUT_SECONDS", "1")

    monkeypatch.setattr(main, "fetch_controlplane_capabilities", lambda *_: {"status": "ok"})
    monkeypatch.setattr(
        main,
        "fetch_ticker",
        lambda *_: {
            "symbol": "BTC_JPY",
            "bid": 100,
            "ask": 101,
            "last": 100.5,
            "stale": False,
            "degraded_reason": None,
            "quality": {"status": "OK"},
        },
    )
    monkeypatch.setattr(main, "fetch_execution_capabilities", lambda *_: {"status": "ok"})

    def _submit(_base_url, intent):
        calls.append(intent)
        return {"order_id": "paper-1", "filled_qty": 0}

    monkeypatch.setattr(main, "submit_order_intent", _submit)

    rc = main.run()

    assert rc == 0
    assert len(calls) == 1
    assert calls[0]["type"] == "MARKET"
