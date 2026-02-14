"use client";

import { useEffect, useMemo, useRef, useState } from "react";

type Ticker = {
  exchange?: string;
  symbol?: string;
  ts_utc?: string;
  timestamp?: string;
  bid?: number;
  ask?: number;
  last?: number;
  degraded_reason?: string | null;
};

const EXCHANGES = ["gmo", "binance"] as const;
const POLL_INTERVAL_MS = 5000;

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

  const [data, setData] = useState<Ticker | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const activeControllerRef = useRef<AbortController | null>(null);
  const inFlightRef = useRef(false);

  useEffect(() => {
    let mounted = true;

    async function load() {
      if (inFlightRef.current && activeControllerRef.current) {
        activeControllerRef.current.abort();
      }

      const controller = new AbortController();
      activeControllerRef.current = controller;
      inFlightRef.current = true;

      setLoading(true);
      try {
        const response = await fetch(
          `/api/ticker?exchange=${encodeURIComponent(activeExchange)}&symbol=${encodeURIComponent(activeSymbol)}`,
          { cache: "no-store", signal: controller.signal }
        );
        const contentType = response.headers.get("content-type") ?? "";
        const payload = contentType.includes("application/json") ? await response.json() : null;

        if (!mounted || activeControllerRef.current !== controller) {
          return;
        }

        if (!response.ok) {
          setError(payload?.error?.message ?? payload?.message ?? payload?.error ?? `ticker fetch failed (${response.status})`);
          setData(null);
          return;
        }

        setData(payload as Ticker);
        setError(null);
      } catch (e) {
        if (!mounted || activeControllerRef.current !== controller) {
          return;
        }
        if (e instanceof Error && e.name === "AbortError") {
          return;
        }
        setError(e instanceof Error ? e.message : "network error");
        setData(null);
      } finally {
        if (activeControllerRef.current === controller) {
          inFlightRef.current = false;
          activeControllerRef.current = null;
        }
        if (mounted) {
          setLoading(false);
        }
      }
    }

    void load();
    const timer = setInterval(() => {
      void load();
    }, POLL_INTERVAL_MS);

    return () => {
      mounted = false;
      clearInterval(timer);
      if (activeControllerRef.current) {
        activeControllerRef.current.abort();
      }
      inFlightRef.current = false;
      activeControllerRef.current = null;
    };
  }, [activeExchange, activeSymbol]);

  function applySelection() {
    const nextSymbol = symbol.trim() || "BTC_JPY";
    setActiveExchange(exchange);
    setActiveSymbol(nextSymbol);

    if (typeof window !== "undefined") {
      const params = new URLSearchParams(window.location.search);
      params.set("exchange", exchange);
      params.set("symbol", nextSymbol);
      const nextUrl = `${window.location.pathname}?${params.toString()}`;
      window.history.replaceState({}, "", nextUrl);
    }
  }

  const degraded = Boolean(data?.degraded_reason);

  return (
    <div className="card">
      <h2>Markets</h2>

      <div style={{ display: "flex", gap: 8, marginBottom: 12, alignItems: "end" }}>
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

      {loading ? <p>Loading ticker...</p> : null}
      {error ? (
        <div style={{ border: "1px solid #7f1d1d", background: "#3f1d1d", borderRadius: 8, padding: 10 }}>
          <strong>Error</strong>
          <p style={{ marginBottom: 0 }}>{error}</p>
        </div>
      ) : null}

      {!loading && !error && !data ? <p>No ticker data available.</p> : null}

      {data ? (
        <table className="table">
          <tbody>
            <tr>
              <th>Status</th>
              <td>
                <span className="badge" style={{ background: degraded ? "#7f1d1d" : "#14532d" }}>
                  {degraded ? "DEGRADED" : "HEALTHY"}
                </span>
              </td>
            </tr>
            <tr>
              <th>Exchange</th>
              <td>{data.exchange ?? activeExchange}</td>
            </tr>
            <tr>
              <th>Symbol</th>
              <td>{data.symbol ?? activeSymbol}</td>
            </tr>
            <tr>
              <th>Bid</th>
              <td>{data.bid ?? "-"}</td>
            </tr>
            <tr>
              <th>Ask</th>
              <td>{data.ask ?? "-"}</td>
            </tr>
            <tr>
              <th>Last</th>
              <td>{data.last ?? "-"}</td>
            </tr>
            <tr>
              <th>Updated</th>
              <td>{data.ts_utc ?? data.timestamp ?? "-"}</td>
            </tr>
            <tr>
              <th>Degraded reason</th>
              <td>{data.degraded_reason ?? "-"}</td>
            </tr>
          </tbody>
        </table>
      ) : null}
    </div>
  );
}
