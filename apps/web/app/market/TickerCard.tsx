"use client";

import { useEffect, useState } from "react";

type TickerLatest = {
  exchange?: string;
  symbol?: string;
  ts_utc?: string;
  timestamp?: string;
  bid?: number;
  ask?: number;
  last?: number;
};

type ErrorState = {
  status: number;
  message: string;
} | null;

const DEFAULT_EXCHANGE = "gmo";
const DEFAULT_SYMBOL = "BTC_JPY";

async function toErrorMessage(response: Response): Promise<string> {
  const contentType = response.headers.get("content-type");
  if (contentType?.includes("application/json")) {
    try {
      const payload = await response.json();
      return payload.message ?? payload.error ?? `ticker fetch failed (${response.status})`;
    } catch {
      return `ticker fetch failed (${response.status})`;
    }
  }
  const text = await response.text();
  return text || `ticker fetch failed (${response.status})`;
}

export function TickerCard() {
  const [data, setData] = useState<TickerLatest | null>(null);
  const [error, setError] = useState<ErrorState>(null);

  useEffect(() => {
    async function load() {
      try {
        const response = await fetch(`/api/ticker?exchange=${DEFAULT_EXCHANGE}&symbol=${DEFAULT_SYMBOL}`, {
          cache: "no-store"
        });

        if (!response.ok) {
          setError({
            status: response.status,
            message: await toErrorMessage(response)
          });
          setData(null);
          return;
        }

        const payload: TickerLatest = await response.json();
        setData(payload);
        setError(null);
      } catch (e) {
        setError({
          status: 0,
          message: e instanceof Error ? e.message : "network error"
        });
        setData(null);
      }
    }

    load();
    const timer = setInterval(load, 5000);
    return () => clearInterval(timer);
  }, []);

  return (
    <div className="card">
      <h2>Market Ticker</h2>
      <p>
        Polling every 5s ({DEFAULT_EXCHANGE}/{DEFAULT_SYMBOL})
      </p>
      {error ? (
        <div
          style={{
            padding: "12px",
            backgroundColor: "#7f1d1d",
            border: "1px solid #991b1b",
            borderRadius: "8px",
            marginBottom: "16px"
          }}
        >
          <div style={{ fontWeight: "600", marginBottom: "4px" }}>Error ({error.status})</div>
          <div style={{ fontSize: "0.875rem", opacity: 0.9 }}>{error.message}</div>
        </div>
      ) : null}
      <table className="table">
        <thead>
          <tr>
            <th>Exchange</th>
            <th>Symbol</th>
            <th>Timestamp (UTC)</th>
            <th>Bid</th>
            <th>Ask</th>
            <th>Last</th>
          </tr>
        </thead>
        <tbody>
          {data ? (
            <tr>
              <td>{data.exchange ?? "-"}</td>
              <td>{data.symbol ?? "-"}</td>
              <td>{data.ts_utc ?? data.timestamp ?? "-"}</td>
              <td>{data.bid ?? "-"}</td>
              <td>{data.ask ?? "-"}</td>
              <td>{data.last ?? "-"}</td>
            </tr>
          ) : (
            <tr>
              <td colSpan={6}>{error ? "Ticker unavailable." : "Loading ticker..."}</td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
