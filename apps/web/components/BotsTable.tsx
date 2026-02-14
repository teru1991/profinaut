"use client";

import { useEffect, useMemo, useState } from "react";

type BotRow = {
  bot_id: string;
  name: string;
  strategy_name: string;
  instance_id: string | null;
  runtime_mode: string | null;
  exchange: string | null;
  symbol: string | null;
  status: string | null;
  state?: string;
  degraded?: boolean;
  degraded_reason?: string | null;
  last_seen: string | null;
  version: string | null;
};

type BotsResponse = {
  page: number;
  page_size: number;
  total: number;
  items: BotRow[];
};

export function BotsTable() {
  const [data, setData] = useState<BotsResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function load() {
    try {
      const response = await fetch("/api/bots?page=1&page_size=50", { cache: "no-store" });
      if (!response.ok) {
        throw new Error(`Failed to load bots (${response.status})`);
      }
      const payload: BotsResponse = await response.json();
      setData(payload);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Unknown error");
    }
  }

  useEffect(() => {
    load();
    const id = setInterval(load, 5000);
    return () => clearInterval(id);
  }, []);

  const rows = useMemo(() => data?.items ?? [], [data]);

  return (
    <div className="card">
      <h2>Bots</h2>
      <p>
        Polling every 5s. Total: <strong>{data?.total ?? 0}</strong>
      </p>
      {error ? <p style={{ color: "#f87171" }}>{error}</p> : null}
      <table className="table">
        <thead>
          <tr>
            <th>Bot</th>
            <th>State</th>
            <th>Last Seen (UTC)</th>
            <th>Version</th>
            <th>Runtime</th>
            <th>Exchange</th>
            <th>Symbol</th>
          </tr>
        </thead>
        <tbody>
          {rows.length === 0 ? (
            <tr>
              <td colSpan={7}>No bots yet. Start an agent heartbeat to populate this view.</td>
            </tr>
          ) : (
            rows.map((row) => {
              const state = row.state ?? row.status ?? "UNKNOWN";
              const isDegraded = row.degraded === true;
              return (
                <tr key={`${row.bot_id}-${row.instance_id ?? "none"}`}>
                  <td>{row.name}</td>
                  <td>
                    <span className="badge">{state}</span>
                    {isDegraded ? (
                      <>
                        {" "}
                        <span className="badge" style={{ backgroundColor: "#991b1b", color: "#fecaca" }}>
                          DEGRADED
                        </span>
                      </>
                    ) : null}
                    {row.degraded_reason ? (
                      <div style={{ fontSize: "0.75rem", opacity: 0.8 }}>{row.degraded_reason}</div>
                    ) : null}
                  </td>
                  <td>{row.last_seen ?? "-"}</td>
                  <td>{row.version ?? "-"}</td>
                  <td>{row.runtime_mode ?? "-"}</td>
                  <td>{row.exchange ?? "-"}</td>
                  <td>{row.symbol ?? "-"}</td>
                </tr>
              );
            })
          )}
        </tbody>
      </table>
    </div>
  );
}
