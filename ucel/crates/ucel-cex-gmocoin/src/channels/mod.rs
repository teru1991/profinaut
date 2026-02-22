pub fn supported_ws_ops() -> Vec<&'static str> {
    vec![
        "crypto.public.ws.ticker.update",
        "crypto.public.ws.trades.update",
        "crypto.public.ws.orderbooks.update",
        "crypto.private.ws.executionevents.update",
        "crypto.private.ws.orderevents.update",
        "crypto.private.ws.positionevents.update",
        "fx.public.ws.ticker.update",
        "fx.public.ws.trades.update",
        "fx.public.ws.orderbooks.update",
        "fx.private.ws.executionevents.update",
        "fx.private.ws.orderevents.update",
        "fx.private.ws.positionevents.update",
    ]
}
