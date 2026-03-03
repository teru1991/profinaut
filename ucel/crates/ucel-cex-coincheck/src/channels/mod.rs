pub fn supported_ws_ops() -> Vec<&'static str> {
    vec![
        "coincheck.ws.public.trades",
        "coincheck.ws.public.orderbook",
        "coincheck.ws.private.order_events",
        "coincheck.ws.private.execution_events",
    ]
}
