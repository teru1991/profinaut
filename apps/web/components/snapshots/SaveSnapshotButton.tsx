"use client";

import { appendEvent } from "../audit/auditStore";
import { saveSnapshot } from "./snapshotStore";

type StatusComponent = { name?: string; status?: string; reason?: string; ts?: string };

type Props = {
  workspaceId: string;
  pageId: string;
  globalFilters: Record<string, string | undefined>;
  overallStatus: string;
  components: StatusComponent[];
  actor?: string;
  note?: string;
};

export function SaveSnapshotButton({ workspaceId, pageId, globalFilters, overallStatus, components, actor = "local-user", note }: Props) {
  function handleSave() {
    const snapshot = {
      id: crypto.randomUUID(),
      ts: new Date().toISOString(),
      dashboardUrl: window.location.href,
      workspaceId,
      pageId,
      globalFilters,
      overall_status: overallStatus,
      degraded_components: components
        .filter((item) => (item.status ?? "").toLowerCase() !== "healthy")
        .map((item) => ({ name: item.name ?? "unknown", status: item.status ?? "unknown", reason: item.reason, ts: item.ts })),
      note
    };
    saveSnapshot(snapshot);
    appendEvent({
      id: crypto.randomUUID(),
      ts: snapshot.ts,
      actor,
      action: "SNAPSHOT_SAVE",
      scope: `${workspaceId}:${pageId}`,
      reason: note?.trim() || "Snapshot saved",
      ttlMinutes: 0,
      result: "SUCCESS",
      details: JSON.stringify({ overall_status: snapshot.overall_status, degraded_count: snapshot.degraded_components.length })
    });
  }

  return <button className="btn" onClick={handleSave}>Save Snapshot</button>;
}
