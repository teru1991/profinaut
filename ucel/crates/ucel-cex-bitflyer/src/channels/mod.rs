pub fn supported_ws_ops() -> Vec<&'static str> {
    vec![
        "crypto.public.ws.ticker",
        "crypto.public.ws.executions",
        "crypto.public.ws.board",
        "crypto.public.ws.board_snapshot",
        "crypto.private.ws.child_order_events",
        "crypto.private.ws.parent_order_events",
        "fx.public.ws.ticker",
        "fx.public.ws.executions",
        "fx.public.ws.board",
        "fx.public.ws.board_snapshot",
        "fx.private.ws.child_order_events",
        "fx.private.ws.parent_order_events",
    ]
}
