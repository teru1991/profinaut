"use client";

import { useEffect, useState } from "react";

type MarketTicker = {
  symbol?: string;
  bid?: number;
  ask?: number;
  last?: number;
  mid?: number;
  ts_utc?: string;
  timestamp?: string;
  degraded_reason?: string | null;
};

type TickerResponse = {
  data?: MarketTicker | null;
};

type ErrorState = {
  status: number;
  message: string;
} | null;

const DEFAULT_SYMBOL = "BTC_JPY";

function parseLastUpdated(data: MarketTicker): string {
  return data.ts_utc ?? data.timestamp ?? "-";
}

async function toErrorMessage(response: Response): Promise<string> {
  const contentType = response.headers.get("content-type");
  if (contentType?.includes("application/json")) {
    try {
      const payload = await response.json();
      if (typeof payload?.message === "string") {
        return payload.message;
      }
      if (typeof payload?.detail?.message === "string") {
        return payload.detail.message;
      }
      if (typeof payload?.detail === "string") {
        return payload.detail;
      }
      return `ticker fetch failed (${response.status})`;
    } catch {
      return `ticker fetch failed (${response.status})`;
    }
  }

  const text = await response.text();
  return text || `ticker fetch failed (${response.status})`;
}

export default function MarketsPage() {
  const [data, setData] = useState<MarketTicker | null>(null);
  const [error, setError] = useState<ErrorState>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      setLoading(true);
      try {
        const search = new URLSearchParams({ symbol: DEFAULT_SYMBOL });
        const response = await fetch(`/api/markets/ticker/latest?${search.toString()}`, {
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

        const payload: TickerResponse = await response.json();
        setData(payload.data ?? null);
        setError(null);
      } catch (e) {
        setError({
          status: 0,
          message: e instanceof Error ? e.message : "network error"
        });
        setData(null);
      } finally {
        setLoading(false);
      }
    }

    load();
  }, []);

  const isDegraded = Boolean(data?.degraded_reason);

  return (
    <div className="card">
      <h2>Markets</h2>
      <p>Latest ticker snapshot ({DEFAULT_SYMBOL})</p>

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

      {loading ? (
        <p>Loading latest ticker...</p>
      ) : data ? (
        <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <span className="badge">{data.symbol ?? DEFAULT_SYMBOL}</span>
            {isDegraded ? (
              <span className="badge" style={{ backgroundColor: "#991b1b", color: "#fecaca" }}>
                DEGRADED
              </span>
            ) : null}
          </div>
          {data.degraded_reason ? (
            <div style={{ fontSize: "0.875rem", color: "#fca5a5" }}>Degraded reason: {data.degraded_reason}</div>
          ) : null}
          <table className="table">
            <thead>
              <tr>
                <th>Bid</th>
                <th>Ask</th>
                <th>Last</th>
                <th>Mid</th>
                <th>Last Updated</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>{data.bid ?? "-"}</td>
                <td>{data.ask ?? "-"}</td>
                <td>{data.last ?? "-"}</td>
                <td>{data.mid ?? "-"}</td>
                <td>{parseLastUpdated(data)}</td>
              </tr>
            </tbody>
          </table>
        </div>
      ) : (
        <p>No ticker data available yet.</p>
      )}
    </div>
  );
}
