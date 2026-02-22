pub fn supported_ws_ops() -> Vec<&'static str> {
    vec![
        "crypto.private.ws.userdata.executionreport",
        "crypto.private.ws.wsapi.order.place",
        "crypto.public.ws.trades.trade",
        "crypto.public.ws.wsapi.time",
    ]
}
