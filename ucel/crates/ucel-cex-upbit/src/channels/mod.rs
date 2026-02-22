pub fn supported_ws_ops() -> Vec<&'static str> {
    vec![
        "quotation.public.ws.ticker.stream",
        "quotation.public.ws.trade.stream",
        "quotation.public.ws.orderbook.stream",
        "quotation.public.ws.candle.stream",
        "other.public.ws.subscription.list",
        "exchange.private.ws.myordertrade.stream",
        "exchange.private.ws.myasset.stream",
    ]
}
