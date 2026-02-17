"use client";

import { useCallback, useEffect, useState } from "react";

type StatusSummary = {
  overall_status: string;
  components?: { name: string; status: string; reason?: string | null }[];
};

type BotsSummary = {
  total: number;
  items: { state?: string; status?: string; degraded?: boolean }[];
};

export default function DashboardPage() {
  const [status, setStatus] = useState<StatusSummary | null>(null);
  const [bots, setBots] = useState<BotsSummary | null>(null);
  const [loading, setLoading] = useState(true);

  const loadAll = useCallback(async () => {
    try {
      const [statusRes, botsRes] = await Promise.allSettled([
        fetch("/api/status/summary", { cache: "no-store" }),
        fetch("/api/bots?page=1&page_size=50", { cache: "no-store" })
      ]);
      if (statusRes.status === "fulfilled" && statusRes.value.ok) {
        setStatus(await statusRes.value.json());
      }
      if (botsRes.status === "fulfilled" && botsRes.value.ok) {
        setBots(await botsRes.value.json());
      }
    } catch {
      // silently fail - dashboard is best-effort
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadAll();
    const id = setInterval(loadAll, 10000);
    return () => clearInterval(id);
  }, [loadAll]);

  const totalBots = bots?.total ?? 0;
  const activeBots = (bots?.items ?? []).filter((b) => {
    const s = (b.state ?? b.status ?? "").toUpperCase();
    return s === "RUNNING" || s === "ACTIVE" || s === "OK";
  }).length;
  const degradedBots = (bots?.items ?? []).filter((b) => b.degraded).length;
  const overallStatus = status?.overall_status ?? "UNKNOWN";
  const degradedComponents = (status?.components ?? []).filter(
    (c) => c.status.toUpperCase() !== "OK"
  );

  if (loading) {
    return (
      <div>
        <div className="page-header">
          <div className="page-header-left">
            <div className="skeleton skeleton-heading" />
            <div className="skeleton skeleton-text" style={{ width: "200px" }} />
          </div>
        </div>
        <div className="card-grid">
          {[1, 2, 3, 4].map((i) => (
            <div key={i} className="skeleton skeleton-card" />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div>
      <div className="page-header">
        <div className="page-header-left">
          <h1 className="page-title">Dashboard</h1>
          <p className="page-subtitle">Control-plane overview and key metrics</p>
        </div>
      </div>

      <div className="card-grid">
        <div className={`kpi-card ${overallStatus === "OK" ? "kpi-success" : overallStatus === "DEGRADED" ? "kpi-warning" : "kpi-error"}`}>
          <span className="kpi-label">System Status</span>
          <span className="kpi-value">{overallStatus}</span>
          <span className="kpi-sub">
            {degradedComponents.length > 0
              ? `${degradedComponents.length} component(s) degraded`
              : "All systems operational"}
          </span>
        </div>

        <div className="kpi-card">
          <span className="kpi-label">Total Bots</span>
          <span className="kpi-value">{totalBots}</span>
          <span className="kpi-sub">Registered in control plane</span>
        </div>

        <div className={`kpi-card ${activeBots > 0 ? "kpi-success" : ""}`}>
          <span className="kpi-label">Active Bots</span>
          <span className="kpi-value">{activeBots}</span>
          <span className="kpi-sub">Running / heartbeating</span>
        </div>

        <div className={`kpi-card ${degradedBots > 0 ? "kpi-warning" : ""}`}>
          <span className="kpi-label">Degraded</span>
          <span className="kpi-value">{degradedBots}</span>
          <span className="kpi-sub">Bots in degraded state</span>
        </div>
      </div>

      {degradedComponents.length > 0 && (
        <div className="section" style={{ marginTop: "var(--space-6)" }}>
          <h2 className="section-title">Degraded Components</h2>
          <div className="card">
            <div className="table-wrapper">
              <table className="table">
                <thead>
                  <tr>
                    <th>Component</th>
                    <th>Status</th>
                    <th>Reason</th>
                  </tr>
                </thead>
                <tbody>
                  {degradedComponents.map((c) => (
                    <tr key={c.name}>
                      <td className="font-medium">{c.name}</td>
                      <td>
                        <span className={`badge ${c.status.toUpperCase() === "DOWN" ? "badge-error" : "badge-warning"}`}>
                          {c.status}
                        </span>
                      </td>
                      <td className="text-muted">{c.reason ?? "-"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      <div className="section" style={{ marginTop: "var(--space-6)" }}>
        <h2 className="section-title">Quick Navigation</h2>
        <div style={{ display: "grid", gap: "var(--space-4)", gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))" }}>
          {[
            { label: "Bots", href: "/bots", desc: "View and monitor all registered bots" },
            { label: "Portfolio", href: "/portfolio", desc: "Exposure metrics and positions" },
            { label: "Markets", href: "/markets", desc: "Live market data and ticker" },
            { label: "Commands", href: "/commands", desc: "Issue PAUSE/RESUME commands" },
            { label: "Analytics", href: "/analytics", desc: "Performance and execution analytics" },
            { label: "Admin", href: "/admin/modules", desc: "Module registry and configuration" }
          ].map((item) => (
            <a key={item.href} href={item.href} className="card card-compact" style={{ textDecoration: "none" }}>
              <h3 className="card-title" style={{ fontSize: "var(--text-base)" }}>{item.label}</h3>
              <p className="card-description">{item.desc}</p>
            </a>
          ))}
        </div>
      </div>
    </div>
  );
}
