"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { formatNumber, formatTimestamp } from "../../lib/format";

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
  const requestSeqRef = useRef(0);

  useEffect(() => {
    let mounted = true;

    async function loadTicker() {
      if (activeControllerRef.current) {
        activeControllerRef.current.abort();
      }

      const requestSeq = requestSeqRef.current + 1;
      requestSeqRef.current = requestSeq;
      const controller = new AbortController();
      activeControllerRef.current = controller;

      try {
        const response = await fetch(
          `/api/markets/ticker/latest?exchange=${encodeURIComponent(exchange)}&symbol=${encodeURIComponent(symbol)}`,
          { cache: "no-store", signal: controller.signal }
        );
        const payload = (await response.json()) as ProxyResponse;

        if (!mounted || requestSeqRef.current !== requestSeq) return;

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
        if (!mounted || activeControllerRef.current !== controller) return;
        if (e instanceof Error && e.name === "AbortError") return;
        const msg = e instanceof Error ? e.message : "Failed to fetch ticker.";
        setError(msg);
        setTicker(null);
      } finally {
        if (activeControllerRef.current === controller) {
          activeControllerRef.current = null;
        }
        if (requestSeqRef.current === requestSeq && mounted) {
          setLoading(false);
        }
      }
    }

    setLoading(true);
    setTicker(null);
    setError(null);
    setRequestId(null);
    void loadTicker();

    const timer = setInterval(() => void loadTicker(), POLL_INTERVAL_MS);

    return () => {
      mounted = false;
      clearInterval(timer);
      if (activeControllerRef.current) {
        activeControllerRef.current.abort();
      }
      activeControllerRef.current = null;
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
      <div className="card-header">
        <div>
          <h2 className="card-title">Ticker</h2>
          <p className="card-description">Auto-refresh every 5s</p>
        </div>
        <div style={{ display: "flex", gap: "var(--space-2)", alignItems: "center" }}>
          {ticker && (
            <span className="badge">{ticker.exchange ?? exchange}:{ticker.symbol ?? symbol}</span>
          )}
          {ticker && (
            degradedReason ? (
              <span className="badge badge-error">DEGRADED</span>
            ) : (
              <span className="badge badge-success">HEALTHY</span>
            )
          )}
        </div>
      </div>

      {loading && !ticker && !error && (
        <div>
          {[1, 2, 3, 4].map((i) => (
            <div key={i} className="skeleton skeleton-row" />
          ))}
        </div>
      )}

      {error && (
        <div className="error-state" style={{ marginBottom: "var(--space-4)" }}>
          <p className="error-state-title">Ticker fetch failed</p>
          <p className="error-state-message">{error}</p>
          {requestId && (
            <p className="error-state-message" style={{ opacity: 0.7 }}>
              request_id: {requestId}
            </p>
          )}
        </div>
      )}

      {!loading && !error && !ticker && (
        <div className="empty-state">
          <div className="empty-state-icon">&#x1F4C8;</div>
          <h3 className="empty-state-title">No ticker data</h3>
          <p className="empty-state-description">
            No data available for this exchange/symbol pair.
          </p>
        </div>
      )}

      {ticker && (
        <div>
          {degradedReason && (
            <div className="notice notice-warning" style={{ marginBottom: "var(--space-4)" }}>
              <span className="notice-icon">!</span>
              <div className="notice-content">
                <div className="notice-title">Degraded</div>
                {degradedReason}
              </div>
            </div>
          )}

          <div className="card-grid-2" style={{ display: "grid", gap: "var(--space-4)", gridTemplateColumns: "repeat(auto-fill, minmax(160px, 1fr))" }}>
            <div className="kpi-card card-compact">
              <span className="kpi-label">Bid</span>
              <span className="kpi-value" style={{ fontSize: "var(--text-xl)" }}>{formatNumber(ticker.bid)}</span>
            </div>
            <div className="kpi-card card-compact">
              <span className="kpi-label">Ask</span>
              <span className="kpi-value" style={{ fontSize: "var(--text-xl)" }}>{formatNumber(ticker.ask)}</span>
            </div>
            <div className="kpi-card card-compact">
              <span className="kpi-label">Last</span>
              <span className="kpi-value" style={{ fontSize: "var(--text-xl)" }}>{formatNumber(ticker.last)}</span>
            </div>
            <div className="kpi-card card-compact">
              <span className="kpi-label">Mid</span>
              <span className="kpi-value" style={{ fontSize: "var(--text-xl)" }}>{formatNumber(ticker.mid)}</span>
            </div>
          </div>

          <table className="table-inline" style={{ marginTop: "var(--space-4)" }}>
            <tbody>
              <tr>
                <th>Ticker ts</th>
                <td>{formatTimestamp(ticker.ts)}</td>
              </tr>
              <tr>
                <th>Last updated</th>
                <td>{formatTimestamp(fetchedAt)}</td>
              </tr>
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
