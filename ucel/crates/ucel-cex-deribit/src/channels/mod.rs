pub fn supported_ws_ops() -> Vec<&'static str> {
    vec![
        "ws.public.sub.book.instrument.raw",
        "ws.public.sub.ticker.instrument.interval",
        "ws.public.sub.trades.instrument.interval",
    ]
}
