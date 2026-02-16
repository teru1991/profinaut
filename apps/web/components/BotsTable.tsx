"use client";

import { useEffect, useMemo, useState } from "react";
import { formatTimestamp, copyToClipboard } from "../lib/format";

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

type SortKey = "name" | "state" | "last_seen" | "exchange" | "symbol";
type SortDir = "asc" | "desc";

function stateBadgeClass(state: string): string {
  const upper = state.toUpperCase();
  if (upper === "RUNNING" || upper === "ACTIVE" || upper === "OK") return "badge badge-success";
  if (upper === "PAUSED" || upper === "IDLE") return "badge badge-warning";
  if (upper === "STOPPED" || upper === "ERROR" || upper === "DOWN") return "badge badge-error";
  return "badge";
}

export function BotsTable() {
  const [data, setData] = useState<BotsResponse | null>(null);
  const [error, setError] = useState<ErrorState>(null);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [sortKey, setSortKey] = useState<SortKey>("name");
  const [sortDir, setSortDir] = useState<SortDir>("asc");
  const [copiedId, setCopiedId] = useState<string | null>(null);

  async function load() {
    try {
      const response = await fetch("/api/bots?page=1&page_size=50", { cache: "no-store" });
      if (!response.ok) {
        const contentType = response.headers.get("content-type");
        let message = "Failed to load bots";
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
        message: e instanceof Error ? e.message : "Network error"
      });
      setData(null);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
    const id = setInterval(load, 5000);
    return () => clearInterval(id);
  }, []);

  const rows = useMemo(() => {
    let items = data?.items ?? [];
    if (search.trim()) {
      const q = search.toLowerCase();
      items = items.filter(
        (r) =>
          r.name.toLowerCase().includes(q) ||
          r.bot_id.toLowerCase().includes(q) ||
          (r.exchange ?? "").toLowerCase().includes(q) ||
          (r.symbol ?? "").toLowerCase().includes(q)
      );
    }
    items = [...items].sort((a, b) => {
      const aVal = (a[sortKey] ?? a.status ?? "").toString().toLowerCase();
      const bVal = (b[sortKey] ?? b.status ?? "").toString().toLowerCase();
      const cmp = aVal.localeCompare(bVal);
      return sortDir === "asc" ? cmp : -cmp;
    });
    return items;
  }, [data, search, sortKey, sortDir]);

  function toggleSort(key: SortKey) {
    if (sortKey === key) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortKey(key);
      setSortDir("asc");
    }
  }

  function sortIndicator(key: SortKey) {
    if (sortKey !== key) return <span className="table-sort-icon">&uarr;&darr;</span>;
    return (
      <span className="table-sort-icon active">
        {sortDir === "asc" ? "\u2191" : "\u2193"}
      </span>
    );
  }

  async function handleCopyId(id: string) {
    const ok = await copyToClipboard(id);
    if (ok) {
      setCopiedId(id);
      setTimeout(() => setCopiedId(null), 1500);
    }
  }

  // Loading skeleton
  if (loading && !data && !error) {
    return (
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)" }}>
        <div className="card">
          <div className="skeleton skeleton-heading" />
          <div className="skeleton skeleton-text" />
          {[1, 2, 3, 4].map((i) => (
            <div key={i} className="skeleton skeleton-row" />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)" }}>
      {/* Bots Table Card */}
      <div className="card">
        <div className="card-header">
          <div>
            <h2 className="card-title">Bots</h2>
            <p className="card-description">
              Auto-refresh every 5s &middot; Total: <strong>{data?.total ?? 0}</strong>
            </p>
          </div>
          <div style={{ display: "flex", gap: "var(--space-2)", alignItems: "center" }}>
            <input
              className="search-input"
              type="text"
              placeholder="Search bots..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              aria-label="Search bots"
            />
          </div>
        </div>

        {error && (
          <div className="error-state" style={{ marginBottom: "var(--space-4)" }}>
            <p className="error-state-title">
              Error{error.status > 0 ? ` (${error.status})` : ""}
            </p>
            <p className="error-state-message">{error.message}</p>
          </div>
        )}

        {!error && rows.length === 0 && data ? (
          <div className="empty-state">
            <div className="empty-state-icon">&#x1F916;</div>
            <h3 className="empty-state-title">
              {search ? "No matching bots" : "No bots registered"}
            </h3>
            <p className="empty-state-description">
              {search
                ? `No bots match "${search}". Try a different search term.`
                : "Start an agent heartbeat to populate this view."}
            </p>
          </div>
        ) : !error ? (
          <div className="table-wrapper">
            <table className="table">
              <thead>
                <tr>
                  <th className="table-sortable" onClick={() => toggleSort("name")}>
                    Bot {sortIndicator("name")}
                  </th>
                  <th className="table-sortable" onClick={() => toggleSort("state")}>
                    State {sortIndicator("state")}
                  </th>
                  <th className="table-sortable" onClick={() => toggleSort("last_seen")}>
                    Last Seen {sortIndicator("last_seen")}
                  </th>
                  <th>Version</th>
                  <th>Runtime</th>
                  <th className="table-sortable" onClick={() => toggleSort("exchange")}>
                    Exchange {sortIndicator("exchange")}
                  </th>
                  <th className="table-sortable" onClick={() => toggleSort("symbol")}>
                    Symbol {sortIndicator("symbol")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {rows.map((row) => {
                  const state = row.state ?? row.status ?? "UNKNOWN";
                  const isDegraded = row.degraded === true;
                  return (
                    <tr key={`${row.bot_id}-${row.instance_id ?? "none"}`}>
                      <td>
                        <div style={{ display: "flex", flexDirection: "column", gap: "2px" }}>
                          <span className="font-medium">{row.name}</span>
                          <span style={{ display: "flex", alignItems: "center", gap: "var(--space-1)" }}>
                            <span className="text-mono text-muted">{row.bot_id.slice(0, 12)}</span>
                            <button
                              className="copy-btn"
                              onClick={() => handleCopyId(row.bot_id)}
                              title="Copy bot ID"
                              aria-label={`Copy bot ID ${row.bot_id}`}
                            >
                              {copiedId === row.bot_id ? "\u2713" : "\u2398"}
                            </button>
                          </span>
                        </div>
                      </td>
                      <td>
                        <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
                          <div style={{ display: "flex", gap: "var(--space-1)", flexWrap: "wrap" }}>
                            <span className={stateBadgeClass(state)}>{state}</span>
                            {isDegraded && (
                              <span className="badge badge-error">DEGRADED</span>
                            )}
                          </div>
                          {row.degraded_reason && (
                            <span className="text-xs text-error">{row.degraded_reason}</span>
                          )}
                        </div>
                      </td>
                      <td className="text-mono tabular-nums">{formatTimestamp(row.last_seen)}</td>
                      <td>{row.version ?? <span className="text-muted">-</span>}</td>
                      <td>
                        {row.runtime_mode ? (
                          <span className="badge badge-accent">{row.runtime_mode}</span>
                        ) : (
                          <span className="text-muted">-</span>
                        )}
                      </td>
                      <td>{row.exchange ?? <span className="text-muted">-</span>}</td>
                      <td>{row.symbol ?? <span className="text-muted">-</span>}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        ) : null}
      </div>

      {/* Kill Switch Panel */}
      <div className="card card-compact">
        <div className="card-header" style={{ marginBottom: "var(--space-2)" }}>
          <h3 className="card-title">Kill Switch</h3>
          <span className="badge badge-success">INACTIVE</span>
        </div>
        <p className="text-sm text-muted" style={{ margin: 0, lineHeight: "var(--leading-relaxed)" }}>
          Kill switch is currently inactive. All bots can operate normally.
          Control actions are not available in this view.
        </p>
      </div>
    </div>
  );
}
