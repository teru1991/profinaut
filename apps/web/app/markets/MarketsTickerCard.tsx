"use client";

import { useEffect, useMemo, useRef, useState } from "react";

type TickerPayload = {
  exchange?: string;
  symbol?: string;
  bid?: number;
  ask?: number;
  last?: number;
  mid?: number;
  ts?: string;
  stale?: boolean;
  degraded_reason?: string | null;
};

type ProxyResponse = {
  request_id?: string;
  data?: TickerPayload;
  code?: string;
  message?: string;
  details?: Record<string, unknown>;
};

const POLL_INTERVAL_MS = 5000;

export function MarketsTickerCard({ exchange = "gmo", symbol = "BTC_JPY" }: { exchange?: string; symbol?: string }) {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [requestId, setRequestId] = useState<string | null>(null);
  const [ticker, setTicker] = useState<TickerPayload | null>(null);
  const [fetchedAt, setFetchedAt] = useState<string | null>(null);

  const activeControllerRef = useRef<AbortController | null>(null);
  const inFlightRef = useRef(false);

  useEffect(() => {
    let mounted = true;

    async function loadTicker() {
      if (inFlightRef.current) {
        return;
      }

      inFlightRef.current = true;
      const controller = new AbortController();
      activeControllerRef.current = controller;

      try {
        const response = await fetch(
          `/api/markets/ticker/latest?exchange=${encodeURIComponent(exchange)}&symbol=${encodeURIComponent(symbol)}`,
          {
            cache: "no-store",
            signal: controller.signal
          }
        );
        const payload = (await response.json()) as ProxyResponse;

        if (!mounted || activeControllerRef.current !== controller) {
          return;
        }

        if (!response.ok) {
          const msg = payload.message ?? payload.code ?? `Ticker fetch failed (${response.status})`;
          setError(msg);
          setRequestId(payload.request_id ?? null);
          setTicker(null);
          return;
        }

        if (!payload.data) {
          setError("Ticker response was empty.");
          setTicker(null);
          return;
        }

        setError(null);
        setRequestId(payload.request_id ?? null);
        setTicker(payload.data);
        setFetchedAt(new Date().toISOString());
      } catch (e) {
        if (!mounted || activeControllerRef.current !== controller) {
          return;
        }
        if (e instanceof Error && e.name === "AbortError") {
          return;
        }
        const msg = e instanceof Error ? e.message : "Failed to fetch ticker.";
        setError(msg);
        setTicker(null);
      } finally {
        if (activeControllerRef.current === controller) {
          activeControllerRef.current = null;
        }
        inFlightRef.current = false;
        if (mounted) {
          setLoading(false);
        }
      }
    }

    setLoading(true);
    setTicker(null);
    setError(null);
    setRequestId(null);
    void loadTicker();

    const timer = setInterval(() => {
      void loadTicker();
    }, POLL_INTERVAL_MS);

    return () => {
      mounted = false;
      clearInterval(timer);
      if (activeControllerRef.current) {
        activeControllerRef.current.abort();
      }
      activeControllerRef.current = null;
      inFlightRef.current = false;
    };
  }, [exchange, symbol]);

  const degradedReason = useMemo(() => {
    if (!ticker) return null;
    if (ticker.degraded_reason) return ticker.degraded_reason;
    if (ticker.stale) return "STALE_TICKER";
    return null;
  }, [ticker]);

  return (
    <div className="card">
      <h2>Markets</h2>
      <p>Latest ticker via dashboard-api marketdata proxy.</p>

      {loading ? <p>Loading latest ticker...</p> : null}

      {error ? (
        <div style={{ border: "1px solid #7f1d1d", background: "#3f1d1d", padding: 12, borderRadius: 8 }}>
          <strong>Ticker fetch failed</strong>
          <p style={{ marginBottom: 0 }}>{error}</p>
          {requestId ? <p style={{ marginTop: 8, opacity: 0.8 }}>request_id: {requestId}</p> : null}
        </div>
      ) : null}

      {!loading && !error && !ticker ? <p>No ticker data available.</p> : null}

      {ticker ? (
        <div>
          <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 12 }}>
            <span className="badge">{ticker.exchange ?? exchange}:{ticker.symbol ?? symbol}</span>
            {degradedReason ? (
              <span className="badge" style={{ background: "#7f1d1d" }}>
                DEGRADED
              </span>
            ) : (
              <span className="badge" style={{ background: "#14532d" }}>
                HEALTHY
              </span>
            )}
          </div>

          <table className="table">
            <tbody>
              <tr>
                <th>Bid</th>
                <td>{ticker.bid ?? "-"}</td>
              </tr>
              <tr>
                <th>Ask</th>
                <td>{ticker.ask ?? "-"}</td>
              </tr>
              <tr>
                <th>Last</th>
                <td>{ticker.last ?? "-"}</td>
              </tr>
              <tr>
                <th>Mid</th>
                <td>{ticker.mid ?? "-"}</td>
              </tr>
              <tr>
                <th>Ticker ts</th>
                <td>{ticker.ts ?? "-"}</td>
              </tr>
              <tr>
                <th>Last updated</th>
                <td>{fetchedAt ?? "-"}</td>
              </tr>
              <tr>
                <th>Degraded reason</th>
                <td>{degradedReason ?? "-"}</td>
              </tr>
            </tbody>
          </table>
        </div>
      ) : null}
    </div>
  );
}
