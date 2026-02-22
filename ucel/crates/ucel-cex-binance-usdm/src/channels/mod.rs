pub fn supported_ws_ops() -> Vec<&'static str> {
    vec![
        "usdm.public.ws.market.root",
        "usdm.public.ws.market.aggtrade",
        "usdm.public.ws.market.markprice",
        "usdm.public.ws.market.kline",
        "usdm.public.ws.market.bookticker",
        "usdm.public.ws.market.liquidation",
        "usdm.public.ws.market.depth.partial",
        "usdm.public.ws.market.depth.diff",
        "usdm.public.ws.wsapi.general",
        "usdm.private.ws.userdata.events",
    ]
}
