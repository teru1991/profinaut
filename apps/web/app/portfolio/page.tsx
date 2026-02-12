"use client";

import { useEffect, useState } from "react";

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

export default function PortfolioPage() {
  const [data, setData] = useState<ExposureSummary | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function load() {
      try {
        const response = await fetch("/api/portfolio/exposure", { cache: "no-store" });
        if (!response.ok) {
          throw new Error(`portfolio fetch failed (${response.status})`);
        }
        const payload: ExposureSummary = await response.json();
        setData(payload);
        setError(null);
      } catch (e) {
        setError(e instanceof Error ? e.message : "unknown error");
      }
    }

    load();
    const timer = setInterval(load, 5000);
    return () => clearInterval(timer);
  }, []);

  return (
    <div className="card">
      <h2>Portfolio Exposure</h2>
      {error ? <p style={{ color: "#f87171" }}>{error}</p> : null}
      <p>
        Total Net Exposure: <strong>{data?.total_net_exposure ?? 0}</strong>
      </p>
      <p>
        Total Gross Exposure: <strong>{data?.total_gross_exposure ?? 0}</strong>
      </p>
      <p>
        Latest Equity: <strong>{data?.key_metrics.latest_equity ?? 0}</strong>
      </p>
      <p>
        Tracked Positions / Symbols: <strong>{data?.key_metrics.tracked_positions ?? 0}</strong> /{" "}
        <strong>{data?.key_metrics.tracked_symbols ?? 0}</strong>
      </p>

      <table className="table">
        <thead>
          <tr>
            <th>Symbol</th>
            <th>Net Exposure</th>
            <th>Gross Exposure</th>
          </tr>
        </thead>
        <tbody>
          {(data?.by_symbol ?? []).length === 0 ? (
            <tr>
              <td colSpan={3}>No exposure data yet.</td>
            </tr>
          ) : (
            (data?.by_symbol ?? []).map((row) => (
              <tr key={row.symbol}>
                <td>{row.symbol}</td>
                <td>{row.net_exposure}</td>
                <td>{row.gross_exposure}</td>
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}
