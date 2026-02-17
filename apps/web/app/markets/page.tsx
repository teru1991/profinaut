"use client";

import { useMemo, useState } from "react";

import { MarketsTickerCard } from "./MarketsTickerCard";

const EXCHANGES = ["gmo", "binance"] as const;

function readSelectionFromUrl(): { exchange: string; symbol: string } {
  if (typeof window === "undefined") {
    return { exchange: "gmo", symbol: "BTC_JPY" };
  }
  const params = new URLSearchParams(window.location.search);
  return {
    exchange: params.get("exchange") ?? "gmo",
    symbol: params.get("symbol") ?? "BTC_JPY"
  };
}

export default function MarketsPage() {
  const initial = useMemo(() => readSelectionFromUrl(), []);

  const [exchange, setExchange] = useState(initial.exchange);
  const [symbol, setSymbol] = useState(initial.symbol);
  const [activeExchange, setActiveExchange] = useState(initial.exchange);
  const [activeSymbol, setActiveSymbol] = useState(initial.symbol);

  function applySelection() {
    const nextExchange = exchange || "gmo";
    const nextSymbol = symbol.trim() || "BTC_JPY";
    setActiveExchange(nextExchange);
    setActiveSymbol(nextSymbol);

    if (typeof window !== "undefined") {
      const params = new URLSearchParams(window.location.search);
      params.set("exchange", nextExchange);
      params.set("symbol", nextSymbol);
      window.history.replaceState({}, "", `${window.location.pathname}?${params.toString()}`);
    }
  }

  return (
    <div>
      <div className="page-header">
        <div className="page-header-left">
          <h1 className="page-title">Markets</h1>
          <p className="page-subtitle">Live ticker data via dashboard-api marketdata proxy</p>
        </div>
      </div>

      {/* Market Selector */}
      <div className="card" style={{ marginBottom: "var(--space-4)" }}>
        <div className="card-header">
          <h2 className="card-title">Market Selector</h2>
          <span className="badge badge-accent">
            {activeExchange.toUpperCase()}:{activeSymbol}
          </span>
        </div>

        <div style={{ display: "flex", gap: "var(--space-3)", alignItems: "flex-end", flexWrap: "wrap" }}>
          <label style={{ display: "grid", gap: "var(--space-1)" }}>
            <span className="text-xs text-muted">Exchange</span>
            <select value={exchange} onChange={(e) => setExchange(e.target.value)}>
              {EXCHANGES.map((item) => (
                <option key={item} value={item}>
                  {item.toUpperCase()}
                </option>
              ))}
            </select>
          </label>

          <label style={{ display: "grid", gap: "var(--space-1)" }}>
            <span className="text-xs text-muted">Symbol</span>
            <input value={symbol} onChange={(e) => setSymbol(e.target.value)} placeholder="BTC_JPY" />
          </label>

          <button className="btn btn-primary" type="button" onClick={applySelection}>
            Apply
          </button>
        </div>
      </div>

      <MarketsTickerCard exchange={activeExchange} symbol={activeSymbol} />
    </div>
  );
}
