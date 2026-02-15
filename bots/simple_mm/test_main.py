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


def test_fetch_pending_commands_uses_expected_filter(monkeypatch) -> None:
    called = {}

    def _http_json(method, url, **_kwargs):
        called["method"] = method
        called["url"] = url
        return 200, []

    monkeypatch.setattr(main, "http_json", _http_json)

    result = main.fetch_pending_commands("http://cp.local", "bot-1")

    assert result == []
    assert called["method"] == "GET"
    assert "target_bot_id=bot-1" in called["url"]
    assert "status=pending" in called["url"]


def test_pause_command_blocks_new_orders_and_sends_ack(monkeypatch, capsys) -> None:
    submit_calls: list[dict] = []
    ack_calls: list[tuple[str, bool, str | None]] = []

    monkeypatch.setenv("SAFE_MODE", "false")
    monkeypatch.setenv("BOT_MAX_LOOPS", "1")
    monkeypatch.setenv("COMMAND_POLL_INTERVAL_SEC", "0")

    monkeypatch.setattr(main, "fetch_controlplane_capabilities", lambda *_: {"status": "ok"})
    monkeypatch.setattr(main, "fetch_ticker", lambda *_: {"symbol": "BTC_JPY", "stale": False, "quality": {"status": "OK"}})
    monkeypatch.setattr(main, "fetch_execution_capabilities", lambda *_: {"status": "ok"})
    monkeypatch.setattr(main, "fetch_pending_commands", lambda *_: [{"id": "cmd-1", "type": "PAUSE"}])

    def _ack(_cp, command_id, ok, reason):
        ack_calls.append((command_id, ok, reason))

    def _submit(_base, intent):
        submit_calls.append(intent)
        return {"order_id": "paper-1", "filled_qty": 0}

    monkeypatch.setattr(main, "send_command_ack", _ack)
    monkeypatch.setattr(main, "submit_order_intent", _submit)

    rc = main.run()

    assert rc == 0
    assert submit_calls == []
    assert ack_calls == [("cmd-1", True, None)]

    events = [json.loads(line) for line in capsys.readouterr().out.strip().splitlines() if line.strip()]
    assert any(e.get("event") == "new_order_blocked" and e.get("reason") == "PAUSED" for e in events)


def test_resume_restores_order_placement_when_not_safe_mode(monkeypatch) -> None:
    submit_calls: list[dict] = []
    ack_calls: list[tuple[str, bool, str | None]] = []
    polls = {"count": 0}

    monkeypatch.setenv("SAFE_MODE", "false")
    monkeypatch.setenv("BOT_MAX_LOOPS", "2")
    monkeypatch.setenv("BOT_LOOP_INTERVAL_SECONDS", "0")
    monkeypatch.setenv("COMMAND_POLL_INTERVAL_SEC", "0")

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

    def _pending(*_args, **_kwargs):
        polls["count"] += 1
        if polls["count"] == 1:
            return [{"id": "cmd-pause", "type": "PAUSE"}]
        return [{"id": "cmd-resume", "type": "RESUME"}]

    def _ack(_cp, command_id, ok, reason):
        ack_calls.append((command_id, ok, reason))

    def _submit(_base, intent):
        submit_calls.append(intent)
        return {"order_id": "paper-1", "filled_qty": 0}

    monkeypatch.setattr(main, "fetch_pending_commands", _pending)
    monkeypatch.setattr(main, "send_command_ack", _ack)
    monkeypatch.setattr(main, "submit_order_intent", _submit)

    rc = main.run()

    assert rc == 0
    assert len(submit_calls) == 1
    assert ack_calls == [("cmd-pause", True, None), ("cmd-resume", True, None)]


def test_unknown_command_is_nacked(monkeypatch) -> None:
    acks: list[tuple[str, bool, str | None]] = []

    monkeypatch.setenv("BOT_MAX_LOOPS", "1")
    monkeypatch.setenv("COMMAND_POLL_INTERVAL_SEC", "0")
    monkeypatch.setattr(main, "fetch_controlplane_capabilities", lambda *_: {"status": "ok"})
    monkeypatch.setattr(main, "fetch_pending_commands", lambda *_: [{"id": "cmd-x", "type": "REBOOT"}])
    monkeypatch.setattr(main, "fetch_ticker", lambda *_: {"symbol": "BTC_JPY", "stale": False, "quality": {"status": "OK"}})
    monkeypatch.setattr(main, "fetch_execution_capabilities", lambda *_: {"status": "ok"})
    monkeypatch.setattr(main, "submit_order_intent", lambda *_: {"order_id": "paper", "filled_qty": 0})

    def _ack(_cp, command_id, ok, reason):
        acks.append((command_id, ok, reason))

    monkeypatch.setattr(main, "send_command_ack", _ack)

    rc = main.run()

    assert rc == 0
    assert len(acks) == 1
    assert acks[0][0] == "cmd-x"
    assert acks[0][1] is False
    assert "Unsupported command type" in (acks[0][2] or "")


def test_marketdata_degraded_blocks_order_submission(monkeypatch) -> None:
    calls: list[str] = []

    monkeypatch.setenv("SAFE_MODE", "false")
    monkeypatch.setenv("EXECUTION_MODE", "paper")
    monkeypatch.setenv("BOT_MAX_LOOPS", "1")

    monkeypatch.setattr(main, "fetch_controlplane_capabilities", lambda *_: {"status": "ok"})
    monkeypatch.setattr(main, "fetch_pending_commands", lambda *_: [])
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
