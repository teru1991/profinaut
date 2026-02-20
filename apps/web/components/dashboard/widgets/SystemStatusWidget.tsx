"use client";

import { statusToBadgeClass, normalizeStatus } from "../ui/badges";
import { type WidgetProps, useReportQuality, useWidgetQuery } from "./runtime";

type StatusSummary = {
  overall_status?: string;
  updated_at?: string;
  components?: { name: string; status: string; reason?: string | null }[];
};

export function SystemStatusWidget({ reportQuality }: WidgetProps) {
  const { result, lastSuccess, refreshNow } = useWidgetQuery<StatusSummary>("/api/status/summary", 10000);
  const status = normalizeStatus(result.data?.overall_status ?? lastSuccess?.data.overall_status);
  useReportQuality(reportQuality, { status, lastSuccessTs: lastSuccess?.ts });

  if (!result.ok && !lastSuccess) {
    return <div className="text-sm">STATUS UNAVAILABLE</div>;
  }

  return (
    <div className="text-sm" style={{ display: "grid", gap: 8 }}>
      <div><span className={statusToBadgeClass(status)}>{status}</span></div>
      {result.error && <div className="text-muted">STATUS UNAVAILABLE ({result.error})</div>}
      <div className="text-muted">Last refresh: {new Date(result.ts).toLocaleTimeString()}</div>
      {lastSuccess && <div className="text-muted">Last success: {new Date(lastSuccess.ts).toLocaleTimeString()}</div>}
      <button className="btn btn-sm" onClick={refreshNow}>Refresh</button>
    </div>
  );
}
