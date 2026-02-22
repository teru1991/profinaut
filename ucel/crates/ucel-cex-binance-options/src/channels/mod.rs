pub fn supported_ws_ops() -> Vec<&'static str> {
    vec![
        "options.public.ws.trade",
        "options.public.ws.ticker",
        "options.public.ws.kline",
        "options.public.ws.depth",
        "options.public.ws.markprice",
        "options.public.ws.indexprice",
        "options.data.ws.market",
    ]
}
