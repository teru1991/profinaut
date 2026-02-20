"use client";

import { normalizeStatus, statusToBadgeClass } from "../ui/badges";
import { type WidgetProps, useReportQuality, useWidgetQuery } from "./runtime";

type StatusSummary = {
  overall_status?: string;
  components?: { name: string; status: string; reason?: string | null }[];
};

export function DegradedComponentsWidget({ reportQuality }: WidgetProps) {
  const { result, lastSuccess } = useWidgetQuery<StatusSummary>("/api/status/summary", 10000);
  const source = result.data ?? lastSuccess?.data;
  const degraded = (source?.components ?? []).filter((component) => normalizeStatus(component.status) !== "OK");
  const status = normalizeStatus(source?.overall_status);
  useReportQuality(reportQuality, { status, lastSuccessTs: lastSuccess?.ts });

  if (!result.ok && !lastSuccess) return <div className="text-sm">STATUS UNAVAILABLE</div>;

  return (
    <div className="text-sm" style={{ display: "grid", gap: 8 }}>
      {degraded.length === 0 ? (
        <div>No degraded components.</div>
      ) : (
        degraded.map((component) => (
          <div key={component.name} style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
            <strong>{component.name}</strong>
            <span className={statusToBadgeClass(component.status)}>{normalizeStatus(component.status)}</span>
            {component.reason && <span className="text-muted">{component.reason}</span>}
          </div>
        ))
      )}
      <div className="text-muted">Last refresh: {new Date(result.ts).toLocaleTimeString()}</div>
      {lastSuccess && <div className="text-muted">Last success: {new Date(lastSuccess.ts).toLocaleTimeString()}</div>}
    </div>
  );
}
