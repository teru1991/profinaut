import json

from bots.simple_mm import main


def test_should_block_new_order_safe_mode():
    blocked, reason = main.should_block_new_order(False, {"degraded": False}, {"status": "ok"})
    assert blocked is False
    assert reason is None

    blocked, reason = main.should_block_new_order(True, {"degraded": False}, {"status": "ok"})
    assert blocked is True
    assert reason == "SAFE_MODE"


def test_run_blocks_on_degraded_execution(monkeypatch, capsys):
    monkeypatch.setenv("SAFE_MODE", "0")
    monkeypatch.setenv("BOT_ID", "bot-x")
    monkeypatch.setattr(main, "fetch_controlplane_capabilities", lambda *_args: {"status": "ok"})

    monkeypatch.setattr(
        main,
        "fetch_ticker",
        lambda *_args: {"degraded": False, "last": 1, "exchange": "gmo", "symbol": "BTC_JPY"},
    )
    monkeypatch.setattr(
        main,
        "fetch_execution_capabilities",
        lambda *_args: {"status": "degraded", "degraded_reason": "MAINTENANCE"},
    )

    submitted = {"called": False}

    def _submit(*_args):
        submitted["called"] = True
        return {}

    monkeypatch.setattr(main, "submit_order_intent", _submit)

    rc = main.run()
    assert rc == 0
    assert submitted["called"] is False

    out = capsys.readouterr().out.strip().splitlines()
    payload = json.loads(out[-1])
    assert payload["event"] == "new_order_blocked"
    assert payload["run_id"]
    assert payload["bot_id"] == "bot-x"


def test_run_safe_mode_skips_network_and_order(monkeypatch, capsys):
    monkeypatch.setenv("SAFE_MODE", "1")
    monkeypatch.setenv("BOT_ID", "bot-safe")

    monkeypatch.setattr(main, "fetch_ticker", lambda *_args: (_ for _ in ()).throw(AssertionError("no fetch")))
    monkeypatch.setattr(
        main,
        "fetch_controlplane_capabilities",
        lambda *_args: (_ for _ in ()).throw(AssertionError("no fetch")),
    )
    monkeypatch.setattr(
        main,
        "fetch_execution_capabilities",
        lambda *_args: (_ for _ in ()).throw(AssertionError("no fetch")),
    )
    monkeypatch.setattr(main, "submit_order_intent", lambda *_args: (_ for _ in ()).throw(AssertionError("no order")))

    rc = main.run()
    assert rc == 0

    out = capsys.readouterr().out.strip().splitlines()
    payload = json.loads(out[-1])
    assert payload["event"] == "new_order_blocked"
    assert payload["reason"] == "SAFE_MODE"
    assert payload["run_id"]
    assert payload["bot_id"] == "bot-safe"


def test_run_submits_order_and_logs_result(monkeypatch, capsys):
    monkeypatch.setenv("SAFE_MODE", "0")
    monkeypatch.setenv("BOT_ID", "bot-y")
    monkeypatch.setenv("ORDER_EXCHANGE", "binance")
    monkeypatch.setenv("ORDER_SYMBOL", "BTC/USDT")

    monkeypatch.setattr(
        main,
        "fetch_ticker",
        lambda *_args: {"degraded": False, "last": 100.0, "exchange": "gmo", "symbol": "BTC_JPY"},
    )
    monkeypatch.setattr(main, "fetch_controlplane_capabilities", lambda *_args: {"status": "ok"})
    monkeypatch.setattr(
        main,
        "fetch_execution_capabilities",
        lambda *_args: {"status": "ok", "degraded_reason": None},
    )

    captured_intent = {}

    def _submit(_base, intent):
        captured_intent.update(intent)
        return {
            "order_id": "paper-1",
            "status": "NEW",
            "exchange": intent["exchange"],
            "symbol": intent["symbol"],
            "side": intent["side"],
            "qty": intent["qty"],
            "filled_qty": 0.0,
            "accepted_ts_utc": "2026-01-01T00:00:00+00:00",
        }

    monkeypatch.setattr(main, "submit_order_intent", _submit)

    rc = main.run()
    assert rc == 0
    assert captured_intent["exchange"] == "binance"
    assert captured_intent["symbol"] == "BTC/USDT"

    out = capsys.readouterr().out.strip().splitlines()
    payload = json.loads(out[-1])
    assert payload["event"] == "order_result"
    assert payload["decision"] == "PLACE_ORDER"
    assert payload["state"] == "RUNNING"
    assert payload["order_id"] == "paper-1"
    assert payload["run_id"]
    assert payload["bot_id"] == "bot-y"
    assert payload["fills"] == []


def test_run_blocks_when_controlplane_unreachable(monkeypatch, capsys):
    monkeypatch.setenv("SAFE_MODE", "0")
    monkeypatch.setenv("BOT_ID", "bot-cp-down")

    monkeypatch.setattr(
        main,
        "fetch_controlplane_capabilities",
        lambda *_args: (_ for _ in ()).throw(main.BotError("controlplane down")),
    )
    monkeypatch.setattr(main, "fetch_ticker", lambda *_args: (_ for _ in ()).throw(AssertionError("no fetch")))
    monkeypatch.setattr(
        main,
        "fetch_execution_capabilities",
        lambda *_args: (_ for _ in ()).throw(AssertionError("no fetch")),
    )
    monkeypatch.setattr(main, "submit_order_intent", lambda *_args: (_ for _ in ()).throw(AssertionError("no order")))

    rc = main.run()
    assert rc == 0

    out = capsys.readouterr().out.strip().splitlines()
    payload = json.loads(out[-1])
    assert payload["event"] == "new_order_blocked"
    assert payload["reason"] == "CONTROLPLANE_UNREACHABLE"
    assert payload["state"] == "DEGRADED"
    assert payload["decision"] == "SKIP_ORDER"


def test_run_blocks_on_degraded_controlplane(monkeypatch, capsys):
    monkeypatch.setenv("SAFE_MODE", "0")
    monkeypatch.setenv("BOT_ID", "bot-cp-degraded")

    monkeypatch.setattr(
        main,
        "fetch_controlplane_capabilities",
        lambda *_args: {"status": "degraded", "degraded_reason": "CONTROLPLANE_MAINTENANCE"},
    )
    monkeypatch.setattr(main, "fetch_ticker", lambda *_args: (_ for _ in ()).throw(AssertionError("no fetch")))
    monkeypatch.setattr(
        main,
        "fetch_execution_capabilities",
        lambda *_args: (_ for _ in ()).throw(AssertionError("no fetch")),
    )
    monkeypatch.setattr(main, "submit_order_intent", lambda *_args: (_ for _ in ()).throw(AssertionError("no order")))

    rc = main.run()
    assert rc == 0

    out = capsys.readouterr().out.strip().splitlines()
    payload = json.loads(out[-1])
    assert payload["event"] == "new_order_blocked"
    assert payload["reason"] == "CONTROLPLANE_MAINTENANCE"
    assert payload["state"] == "DEGRADED"
    assert payload["decision"] == "SKIP_ORDER"
