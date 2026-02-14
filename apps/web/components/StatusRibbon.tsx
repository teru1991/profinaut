"use client";

import { useCallback, useEffect, useState } from "react";

type ComponentStatus = {
  name: string;
  status: string;
  reason?: string | null;
};

type StatusSummary = {
  overall_status: string;
  components?: ComponentStatus[];
};

type FetchState = {
  data: StatusSummary | null;
  error: string | null;
  loading: boolean;
  lastRefresh: Date | null;
};

const POLL_INTERVAL_MS = 30000;

function formatTime(date: Date): string {
  return date.toLocaleTimeString();
}

function statusColor(status: string): string {
  switch (status.toUpperCase()) {
    case "OK":
      return "#065f46";
    case "DEGRADED":
      return "#92400e";
    case "DOWN":
      return "#7f1d1d";
    default:
      return "#1f2937";
  }
}

function statusTextColor(status: string): string {
  switch (status.toUpperCase()) {
    case "OK":
      return "#6ee7b7";
    case "DEGRADED":
      return "#fcd34d";
    case "DOWN":
      return "#fca5a5";
    default:
      return "#e6edf3";
  }
}

function ribbonBackground(status: string): string {
  switch (status.toUpperCase()) {
    case "OK":
      return "#0f172a";
    case "DEGRADED":
      return "#451a03";
    case "DOWN":
      return "#450a0a";
    default:
      return "#0f172a";
  }
}

export function StatusRibbon() {
  const [state, setState] = useState<FetchState>({
    data: null,
    error: null,
    loading: true,
    lastRefresh: null
  });

  const fetchStatus = useCallback(async () => {
    try {
      const response = await fetch("/api/status/summary", { cache: "no-store" });
      if (!response.ok) {
        const contentType = response.headers.get("content-type");
        let message = "Failed to load system status";
        if (contentType?.includes("application/json")) {
          try {
            const errorData = await response.json();
            message = errorData.message || errorData.error || message;
          } catch {
            message = await response.text() || message;
          }
        }
        setState((prev) => ({
          ...prev,
          error: message,
          loading: false,
          lastRefresh: new Date()
        }));
        return;
      }
      const payload: StatusSummary = await response.json();
      setState({
        data: payload,
        error: null,
        loading: false,
        lastRefresh: new Date()
      });
    } catch (e) {
      setState((prev) => ({
        ...prev,
        error: e instanceof Error ? e.message : "Network error",
        loading: false,
        lastRefresh: new Date()
      }));
    }
  }, []);

  useEffect(() => {
    fetchStatus();
    const id = setInterval(fetchStatus, POLL_INTERVAL_MS);
    return () => clearInterval(id);
  }, [fetchStatus]);

  const handleRefresh = () => {
    setState((prev) => ({ ...prev, loading: true }));
    fetchStatus();
  };

  const overallStatus = state.data?.overall_status ?? "UNKNOWN";
  const degradedComponents = (state.data?.components ?? []).filter(
    (c) => c.status.toUpperCase() !== "OK"
  );

  const bg = state.error
    ? "#1e1b4b"
    : ribbonBackground(overallStatus);

  return (
    <div
      className="status-ribbon"
      style={{ background: bg }}
    >
      <div className="status-ribbon-content">
        {state.loading && !state.data && !state.error ? (
          <span className="status-ribbon-text">Loading system status…</span>
        ) : state.error ? (
          <>
            <span className="badge status-ribbon-badge-unavailable">
              STATUS UNAVAILABLE
            </span>
            <span className="status-ribbon-text" style={{ opacity: 0.8 }}>
              {state.error}
            </span>
          </>
        ) : (
          <>
            <span
              className="badge"
              style={{
                backgroundColor: statusColor(overallStatus),
                color: statusTextColor(overallStatus),
                fontSize: "11px"
              }}
            >
              {overallStatus}
            </span>
            {degradedComponents.length > 0 && (
              <span className="status-ribbon-text">
                {degradedComponents.map((c) => (
                  <span key={c.name} style={{ marginRight: "12px" }}>
                    <strong>{c.name}</strong>: {c.status}
                    {c.reason ? ` — ${c.reason}` : ""}
                  </span>
                ))}
              </span>
            )}
            {degradedComponents.length === 0 && overallStatus.toUpperCase() === "OK" && (
              <span className="status-ribbon-text" style={{ opacity: 0.7 }}>
                All systems operational
              </span>
            )}
          </>
        )}
      </div>

      <div className="status-ribbon-actions">
        {state.lastRefresh && (
          <span className="status-ribbon-time">
            {formatTime(state.lastRefresh)}
          </span>
        )}
        <button
          className="status-ribbon-refresh"
          onClick={handleRefresh}
          disabled={state.loading}
          title="Refresh status"
        >
          ↻
        </button>
      </div>
    </div>
  );
}
