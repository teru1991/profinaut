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
  return date.toLocaleTimeString("en-US", { hour12: false });
}

function statusBadgeClass(status: string): string {
  switch (status.toUpperCase()) {
    case "OK": return "badge badge-success";
    case "DEGRADED": return "badge badge-warning";
    case "DOWN": return "badge badge-error";
    default: return "badge";
  }
}

function ribbonBgVar(status: string): string {
  switch (status.toUpperCase()) {
    case "OK": return "var(--ribbon-ok-bg)";
    case "DEGRADED": return "var(--ribbon-degraded-bg)";
    case "DOWN": return "var(--ribbon-down-bg)";
    default: return "var(--ribbon-ok-bg)";
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
    ? "var(--ribbon-unknown-bg)"
    : ribbonBgVar(overallStatus);

  return (
    <div className="status-ribbon" style={{ background: bg }}>
      <div className="status-ribbon-content">
        {state.loading && !state.data && !state.error ? (
          <span className="status-ribbon-text">Loading system status...</span>
        ) : state.error ? (
          <>
            <span className="badge badge-info" style={{ fontSize: "var(--text-xs)" }}>
              STATUS UNAVAILABLE
            </span>
            <span className="status-ribbon-text" style={{ opacity: 0.8 }}>
              {state.error}
            </span>
          </>
        ) : (
          <>
            <span className={statusBadgeClass(overallStatus)} style={{ fontSize: "var(--text-xs)" }}>
              {overallStatus}
            </span>
            {degradedComponents.length > 0 && (
              <span className="status-ribbon-text">
                {degradedComponents.map((c) => (
                  <span key={c.name} style={{ marginRight: "var(--space-3)" }}>
                    <strong>{c.name}</strong>: {c.status}
                    {c.reason ? ` \u2014 ${c.reason}` : ""}
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
          aria-label="Refresh system status"
        >
          &#x21bb;
        </button>
      </div>
    </div>
  );
}
