pub fn supported_ws_ops() -> Vec<&'static str> {
    vec![
        "crypto.public.ws.market_data.ticker",
        "crypto.public.ws.market_data.trades",
        "crypto.public.ws.market_data.orderbook",
    ]
}
