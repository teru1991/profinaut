"use client";

import { useEffect, useState } from "react";
import { formatNumber, formatTimestamp } from "../../lib/format";

type ExposureBySymbol = {
  symbol: string;
  net_exposure: number;
  gross_exposure: number;
};

type ExposureSummary = {
  generated_at: string;
  total_net_exposure: number;
  total_gross_exposure: number;
  key_metrics: {
    latest_equity: number;
    tracked_positions: number;
    tracked_symbols: number;
  };
  by_symbol: ExposureBySymbol[];
};


function formatLastUpdatedTime(value: Date): string {
  return value.toLocaleTimeString("en-US", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false
  });
}

export default function PortfolioPage() {
  const [data, setData] = useState<ExposureSummary | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);

  useEffect(() => {
    async function load() {
      try {
        const response = await fetch("/api/portfolio/exposure", { cache: "no-store" });
        if (!response.ok) {
          throw new Error(`Portfolio fetch failed (${response.status})`);
        }
        const payload: ExposureSummary = await response.json();
        setData(payload);
        setError(null);
        setLastUpdated(new Date());
      } catch (e) {
        setError(e instanceof Error ? e.message : "Unknown error");
      } finally {
        setLoading(false);
      }
    }

    load();
    const timer = setInterval(load, 5000);
    return () => clearInterval(timer);
  }, []);

  if (loading && !data && !error) {
    return (
      <div>
        <div className="page-header">
          <div className="page-header-left">
            <div className="skeleton skeleton-heading" />
            <div className="skeleton skeleton-text" style={{ width: "180px" }} />
          </div>
        </div>
        <div className="card-grid">
          {[1, 2, 3, 4].map((i) => (
            <div key={i} className="skeleton skeleton-card" />
          ))}
        </div>
        <div className="card" style={{ marginTop: "var(--space-4)" }}>
          {[1, 2, 3].map((i) => (
            <div key={i} className="skeleton skeleton-row" />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div>
      <div className="page-header">
        <div className="page-header-left">
          <h1 className="page-title">Portfolio</h1>
          <p className="page-subtitle">Exposure summary and position breakdown</p>
        </div>
        {lastUpdated && (
          <div className="last-updated">
            <span className="last-updated-dot" />
            Auto-refresh 5s &middot; {formatLastUpdatedTime(lastUpdated)}
          </div>
        )}
      </div>

      {error && (
        <div className="error-state" style={{ marginBottom: "var(--space-4)" }}>
          <p className="error-state-title">Failed to load portfolio data</p>
          <p className="error-state-message">{error}</p>
        </div>
      )}

      <div className="card-grid">
        <div className="kpi-card">
          <span className="kpi-label">Net Exposure</span>
          <span className="kpi-value">{formatNumber(data?.total_net_exposure)}</span>
          <span className="kpi-sub">Total net across all positions</span>
        </div>

        <div className="kpi-card">
          <span className="kpi-label">Gross Exposure</span>
          <span className="kpi-value">{formatNumber(data?.total_gross_exposure)}</span>
          <span className="kpi-sub">Total absolute exposure</span>
        </div>

        <div className="kpi-card">
          <span className="kpi-label">Latest Equity</span>
          <span className="kpi-value">{formatNumber(data?.key_metrics.latest_equity)}</span>
          <span className="kpi-sub">Most recent equity reading</span>
        </div>

        <div className="kpi-card">
          <span className="kpi-label">Tracked</span>
          <span className="kpi-value">
            {data?.key_metrics.tracked_positions ?? 0} / {data?.key_metrics.tracked_symbols ?? 0}
          </span>
          <span className="kpi-sub">Positions / Symbols</span>
        </div>
      </div>

      <div className="section" style={{ marginTop: "var(--space-6)" }}>
        <h2 className="section-title">Exposure by Symbol</h2>
        <div className="card">
          {data && data.by_symbol.length === 0 ? (
            <div className="empty-state">
              <div className="empty-state-icon">&#x1F4CA;</div>
              <h3 className="empty-state-title">No exposure data</h3>
              <p className="empty-state-description">
                Start trading bots to populate position data.
              </p>
            </div>
          ) : data ? (
            <div className="table-wrapper">
              <table className="table">
                <thead>
                  <tr>
                    <th>Symbol</th>
                    <th className="text-right">Net Exposure</th>
                    <th className="text-right">Gross Exposure</th>
                  </tr>
                </thead>
                <tbody>
                  {data.by_symbol.map((row) => (
                    <tr key={row.symbol}>
                      <td className="font-medium">{row.symbol}</td>
                      <td className="text-right tabular-nums">{formatNumber(row.net_exposure)}</td>
                      <td className="text-right tabular-nums">{formatNumber(row.gross_exposure)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : null}

          {data?.generated_at && (
            <p className="text-xs text-muted" style={{ marginTop: "var(--space-3)", marginBottom: 0 }}>
              Generated at: {formatTimestamp(data.generated_at)}
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
