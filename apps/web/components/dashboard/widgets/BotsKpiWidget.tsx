"use client";

import { normalizeStatus } from "../ui/badges";
import { type WidgetProps, useReportQuality, useWidgetQuery } from "./runtime";

type BotsSummary = { total: number; items: { state?: string; status?: string; degraded?: boolean }[] };

export function BotsKpiWidget({ reportQuality }: WidgetProps) {
  const { result, lastSuccess } = useWidgetQuery<BotsSummary>("/api/bots?page=1&page_size=50", 10000);
  const source = result.data ?? lastSuccess?.data;

  const total = source?.total ?? 0;
  const active = (source?.items ?? []).filter((b) => {
    const state = (b.state ?? b.status ?? "").toUpperCase();
    return state === "RUNNING" || state === "ACTIVE" || state === "OK";
  }).length;
  const degraded = (source?.items ?? []).filter((b) => b.degraded).length;

  const status = !source ? "UNKNOWN" : degraded > 0 ? "DEGRADED" : "OK";
  useReportQuality(reportQuality, { status: normalizeStatus(status), lastSuccessTs: lastSuccess?.ts });

  if (!result.ok && !lastSuccess) return <div className="text-sm">BOTS UNAVAILABLE</div>;

  return (
    <div className="text-sm" style={{ display: "grid", gap: 8 }}>
      <div>Total {total}</div>
      <div>Active {active}</div>
      <div>Degraded {degraded}</div>
      <div className="text-muted">Last refresh: {new Date(result.ts).toLocaleTimeString()}</div>
      {lastSuccess && <div className="text-muted">Last success: {new Date(lastSuccess.ts).toLocaleTimeString()}</div>}
    </div>
  );
}
