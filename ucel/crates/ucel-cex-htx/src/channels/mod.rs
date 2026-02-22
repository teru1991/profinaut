pub fn supported_ws_ops() -> Vec<&'static str> {
    vec![
        "spot.public.ws.market.catalog.index",
        "spot.private.ws.account.catalog.index",
        "futures.public.ws.market.catalog.index",
        "futures.private.ws.account.catalog.index",
        "swap.public.ws.market.catalog.index",
        "swap.private.ws.account.catalog.index",
        "options.public.ws.market.catalog.index",
        "other.public.ws.common.protocol",
        "other.public.ws.other.not_applicable",
    ]
}
