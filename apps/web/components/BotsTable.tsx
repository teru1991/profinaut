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

type ErrorState = {
  status: number;
  message: string;
} | null;

export function BotsTable() {
  const [data, setData] = useState<BotsResponse | null>(null);
  const [error, setError] = useState<ErrorState>(null);

  async function load() {
    try {
      const response = await fetch("/api/bots?page=1&page_size=50", { cache: "no-store" });
      if (!response.ok) {
        const contentType = response.headers.get("content-type");
        let message = `Failed to load bots`;
        
        if (contentType?.includes("application/json")) {
          try {
            const errorData = await response.json();
            message = errorData.message || errorData.error || message;
          } catch {
            message = await response.text() || message;
          }
        } else {
          message = await response.text() || message;
        }
        
        setError({ status: response.status, message });
        setData(null);
        return;
      }
      const payload: BotsResponse = await response.json();
      setData(payload);
      setError(null);
    } catch (e) {
      setError({ 
        status: 0, 
        message: e instanceof Error ? e.message : "Network error or unknown error" 
      });
      setData(null);
    }
  }

  useEffect(() => {
    load();
    const id = setInterval(load, 5000);
    return () => clearInterval(id);
  }, []);

  const rows = useMemo(() => data?.items ?? [], [data]);

  return (
    <div>
      <div className="card">
        <h2>Bots</h2>
        <p>
          Polling every 5s. Total: <strong>{data?.total ?? 0}</strong>
        </p>
        {error ? (
          <div style={{ 
            padding: "12px", 
            backgroundColor: "#7f1d1d", 
            border: "1px solid #991b1b", 
            borderRadius: "8px",
            marginBottom: "16px"
          }}>
            <div style={{ fontWeight: "600", marginBottom: "4px" }}>
              Error {error.status > 0 ? `(${error.status})` : ""}
            </div>
            <div style={{ fontSize: "0.875rem", opacity: 0.9 }}>{error.message}</div>
          </div>
        ) : null}
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
            {rows.length === 0 && !error ? (
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
                      <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
                        <div>
                          <span className="badge">{state}</span>
                          {isDegraded ? (
                            <>
                              {" "}
                              <span className="badge" style={{ backgroundColor: "#991b1b", color: "#fecaca" }}>
                                DEGRADED
                              </span>
                            </>
                          ) : null}
                        </div>
                        {row.degraded_reason ? (
                          <div style={{ fontSize: "0.75rem", color: "#fca5a5" }}>
                            {row.degraded_reason}
                          </div>
                        ) : null}
                      </div>
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

      {/* Read-only Kill Switch Panel */}
      <div className="card" style={{ marginTop: "24px" }}>
        <h2>Kill Switch</h2>
        <div style={{ 
          padding: "12px", 
          backgroundColor: "#1f2937", 
          border: "1px solid #374151", 
          borderRadius: "8px",
          marginTop: "12px"
        }}>
          <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "8px" }}>
            <span className="badge" style={{ backgroundColor: "#065f46", color: "#6ee7b7" }}>
              INACTIVE
            </span>
            <span style={{ fontSize: "0.875rem", opacity: 0.8 }}>
              Read-only mode
            </span>
          </div>
          <div style={{ fontSize: "0.875rem", opacity: 0.7, lineHeight: "1.5" }}>
            Kill switch is currently inactive. All bots can operate normally.
            Control actions are not available in this view.
          </div>
        </div>
      </div>
    </div>
  );
}
