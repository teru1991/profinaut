"use client";

import { useMemo, useState } from "react";

import { appendEvent } from "../../components/audit/auditStore";
import { exportSnapshots, listSnapshots, updateSnapshotNote } from "../../components/snapshots/snapshotStore";
import { formatTimestamp } from "../../lib/format";

export default function SnapshotsPage() {
  const [tick, setTick] = useState(0);
  const [noteDraft, setNoteDraft] = useState<Record<string, string>>({});

  const snapshots = useMemo(() => listSnapshots(), [tick]);

  function exportJson() {
    const blob = new Blob([JSON.stringify(exportSnapshots(), null, 2)], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `snapshots-${new Date().toISOString()}.json`;
    link.click();
    URL.revokeObjectURL(url);
  }

  function saveNote(id: string) {
    const note = (noteDraft[id] ?? "").trim();
    updateSnapshotNote(id, note);
    appendEvent({
      id: crypto.randomUUID(),
      ts: new Date().toISOString(),
      actor: "local-user",
      action: "NOTES",
      scope: `snapshot:${id}`,
      reason: "Snapshot annotation",
      ttlMinutes: 0,
      result: "SUCCESS",
      details: note
    });
    setTick((v) => v + 1);
  }

  return (
    <div>
      <div className="page-header">
        <div className="page-header-left">
          <h1 className="page-title">Snapshots</h1>
          <p className="page-subtitle">Saved incident evidence snapshots from dashboard/incidents.</p>
        </div>
        <div className="page-header-actions"><button className="btn" onClick={exportJson}>Export JSON</button></div>
      </div>

      <div className="card">
        {snapshots.length === 0 ? <p className="text-muted">No snapshots yet.</p> : snapshots.map((snapshot) => (
          <div key={snapshot.id} className="card" style={{ marginBottom: "var(--space-3)" }}>
            <p><strong>Time:</strong> {formatTimestamp(snapshot.ts)}</p>
            <p><strong>Status:</strong> {snapshot.overall_status}</p>
            <p><strong>URL:</strong> <a href={snapshot.dashboardUrl}>{snapshot.dashboardUrl}</a></p>
            <pre style={{ overflow: "auto" }}>{JSON.stringify(snapshot.degraded_components, null, 2)}</pre>
            <textarea
              rows={2}
              placeholder="Add/edit note"
              value={noteDraft[snapshot.id] ?? snapshot.note ?? ""}
              onChange={(e) => setNoteDraft((prev) => ({ ...prev, [snapshot.id]: e.target.value }))}
            />
            <div><button className="btn" onClick={() => saveNote(snapshot.id)}>Save Note</button></div>
          </div>
        ))}
      </div>
    </div>
  );
}
