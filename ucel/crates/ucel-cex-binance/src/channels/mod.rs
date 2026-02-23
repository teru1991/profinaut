pub fn supported_ws_ops() -> Vec<&'static str> {
    vec![
        "crypto.public.ws.trades.trade",
        "crypto.private.ws.userdata.executionreport",
        "crypto.public.ws.wsapi.time",
        "crypto.private.ws.wsapi.order.place",
    ]
}
