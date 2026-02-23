"use client";

import { useMemo, useState } from "react";

import { AuditResult } from "../../components/audit/auditModel";
import { clearOld, exportRawEvents, listEvents } from "../../components/audit/auditStore";
import { formatTimestamp } from "../../lib/format";

export default function AuditPage() {
  const [action, setAction] = useState("");
  const [result, setResult] = useState<AuditResult | "">("");
  const [fromTs, setFromTs] = useState("");
  const [toTs, setToTs] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [refreshTick, setRefreshTick] = useState(0);

  const events = useMemo(
    () => listEvents({ action: action || undefined, result: result || undefined, fromTs: fromTs || undefined, toTs: toTs || undefined }),
    [action, result, fromTs, toTs, refreshTick]
  );

  const selected = events.find((event) => event.id === selectedId) ?? null;

  function exportJson() {
    const raw = JSON.stringify(exportRawEvents(), null, 2);
    const blob = new Blob([raw], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `audit-events-${new Date().toISOString()}.json`;
    link.click();
    URL.revokeObjectURL(url);
  }

  return (
    <div>
      <div className="page-header">
        <div className="page-header-left">
          <h1 className="page-title">Personal Audit Log</h1>
          <p className="page-subtitle">Local, personal-grade action trail.</p>
        </div>
        <div className="page-header-actions">
          <button className="btn" onClick={exportJson}>Export JSON</button>
          <button className="btn" onClick={() => { clearOld(30); setRefreshTick((v) => v + 1); }}>Clear &gt;30d</button>
        </div>
      </div>

      <div className="card" style={{ marginBottom: "var(--space-4)" }}>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(140px, 1fr))", gap: "var(--space-2)" }}>
          <input placeholder="Action" value={action} onChange={(e) => setAction(e.target.value)} />
          <select value={result} onChange={(e) => setResult(e.target.value as AuditResult | "") }>
            <option value="">Any result</option>
            <option value="SUCCESS">SUCCESS</option>
            <option value="FAILED">FAILED</option>
          </select>
          <input type="datetime-local" value={fromTs} onChange={(e) => setFromTs(e.target.value ? new Date(e.target.value).toISOString() : "")} />
          <input type="datetime-local" value={toTs} onChange={(e) => setToTs(e.target.value ? new Date(e.target.value).toISOString() : "")} />
        </div>
      </div>

      <div className="card">
        <div className="table-wrapper">
          <table className="table">
            <thead><tr><th>Time</th><th>Action</th><th>Scope</th><th>Result</th><th>Reason</th></tr></thead>
            <tbody>
              {events.map((event) => (
                <tr key={event.id} onClick={() => setSelectedId(event.id)} style={{ cursor: "pointer" }}>
                  <td>{formatTimestamp(event.ts)}</td>
                  <td>{event.action}</td>
                  <td>{event.scope}</td>
                  <td><span className={`badge ${event.result === "SUCCESS" ? "badge-success" : "badge-error"}`}>{event.result}</span></td>
                  <td>{event.reason}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {selected && (
        <div className="card" style={{ marginTop: "var(--space-4)" }}>
          <h2 className="card-title">Event Detail</h2>
          <pre style={{ margin: 0, overflow: "auto" }}>{JSON.stringify(selected, null, 2)}</pre>
        </div>
      )}
    </div>
  );
}
