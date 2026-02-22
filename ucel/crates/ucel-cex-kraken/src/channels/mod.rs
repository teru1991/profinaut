pub fn supported_ws_ops() -> Vec<&'static str> {
    vec![
        "spot.public.ws.v1.market.book.subscribe",
        "spot.public.ws.v1.market.trade.subscribe",
        "spot.private.ws.v1.account.open_orders.subscribe",
        "spot.private.ws.v1.trade.add_order.request",
        "spot.public.ws.v2.market.book.subscribe",
        "spot.public.ws.v2.market.instrument.subscribe",
        "spot.private.ws.v2.trade.add_order",
        "futures.public.ws.other.market.ticker.subscribe",
        "futures.public.ws.other.market.book.subscribe",
        "futures.private.ws.other.account.open_positions.subscribe",
    ]
}
