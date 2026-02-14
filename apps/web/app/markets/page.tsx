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
    <div style={{ display: "grid", gap: 12 }}>
      <div className="card">
        <h2>Market selector</h2>
        <p>Choose exchange/symbol and persist to query params.</p>

        <div style={{ display: "flex", gap: 8, alignItems: "end", flexWrap: "wrap" }}>
          <label>
            <div>Exchange</div>
            <select value={exchange} onChange={(e) => setExchange(e.target.value)}>
              {EXCHANGES.map((item) => (
                <option key={item} value={item}>
                  {item.toUpperCase()}
                </option>
              ))}
            </select>
          </label>

          <label>
            <div>Symbol</div>
            <input value={symbol} onChange={(e) => setSymbol(e.target.value)} placeholder="BTC_JPY" />
          </label>

          <button type="button" onClick={applySelection}>
            Apply
          </button>
        </div>

        <p style={{ marginTop: 8, opacity: 0.8 }}>
          Active query: ?exchange={activeExchange}&amp;symbol={activeSymbol}
        </p>
      </div>

      <MarketsTickerCard exchange={activeExchange} symbol={activeSymbol} />
    </div>
  );
}
