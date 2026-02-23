pub fn supported_ws_ops() -> Vec<&'static str> {
    vec![
        "crypto.public.ws.ticker.update",
        "crypto.public.ws.trades.update",
        "crypto.public.ws.orderbooks.update",
        "crypto.private.ws.executionevents.update",
        "crypto.private.ws.orderevents.update",
        "crypto.private.ws.positionevents.update",
    ]
}
