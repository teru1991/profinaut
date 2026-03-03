pub fn supported_ws_ops() -> Vec<&'static str> {
    vec![
        "public.ws.market.kline",
        "public.ws.market.depth",
        "public.ws.market.bbo",
        "public.ws.market.detail",
        "public.ws.market.trade.detail",
        "private.ws.accounts.update",
        "private.ws.trade.clearing",
    ]
}
