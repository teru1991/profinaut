"use client";

import { useEffect, useMemo, useState } from "react";

import { appendEvent } from "../audit/auditStore";
import { SaveSnapshotButton } from "../snapshots/SaveSnapshotButton";
import { formatTimestamp } from "../../lib/format";

type ComponentStatus = {
  name?: string;
  status?: string;
  reason?: string;
  ts?: string;
  updated_at?: string;
};

type StatusSummary = {
  overall_status?: string;
  ts?: string;
  components?: ComponentStatus[];
};

const CHECKLIST = [
  "Validate market data freshness",
  "Check bot command backlog and ack failures",
  "Confirm upstream API health and auth",
  "Review recent deploys or config changes"
];

const CHECKLIST_KEY = "profinaut.incident.checklist.v1";

function readChecklist(): Record<string, boolean> {
  try {
    const raw = window.localStorage.getItem(CHECKLIST_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw);
    return parsed && typeof parsed === "object" ? parsed as Record<string, boolean> : {};
  } catch {
    return {};
  }
}

function writeChecklist(value: Record<string, boolean>) {
  window.localStorage.setItem(CHECKLIST_KEY, JSON.stringify(value));
}

export function IncidentConsole() {
  const [summary, setSummary] = useState<StatusSummary | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [lastRefresh, setLastRefresh] = useState<string | null>(null);
  const [selected, setSelected] = useState<ComponentStatus | null>(null);
  const [checklist, setChecklist] = useState<Record<string, boolean>>({});
  const [stepNote, setStepNote] = useState("");

  useEffect(() => setChecklist(readChecklist()), []);

  async function refresh() {
    try {
      const res = await fetch("/api/status/summary", { cache: "no-store" });
      const payload = await res.json();
      if (!res.ok) {
        setError(`UNAVAILABLE (${res.status})`);
        setSummary(null);
        setLastRefresh(new Date().toISOString());
        return;
      }
      setSummary(payload as StatusSummary);
      setError(null);
      setLastRefresh(new Date().toISOString());
    } catch (err) {
      setError(err instanceof Error ? `UNAVAILABLE: ${err.message}` : "UNAVAILABLE");
      setSummary(null);
      setLastRefresh(new Date().toISOString());
    }
  }

  useEffect(() => {
    void refresh();
    const id = setInterval(() => void refresh(), 10000);
    return () => clearInterval(id);
  }, []);

  const degraded = useMemo(
    () => (summary?.components ?? []).filter((item) => (item.status ?? "healthy").toLowerCase() !== "healthy"),
    [summary]
  );

  function toggleStep(step: string) {
    const next = { ...checklist, [step]: !checklist[step] };
    setChecklist(next);
    writeChecklist(next);
    appendEvent({
      id: crypto.randomUUID(),
      ts: new Date().toISOString(),
      actor: "local-user",
      action: "NOTES",
      scope: `incident-checklist:${selected?.name ?? "unknown"}`,
      reason: `Checklist ${next[step] ? "checked" : "unchecked"}: ${step}`,
      ttlMinutes: 0,
      result: "SUCCESS",
      details: stepNote || undefined
    });
  }

  function saveStepNote() {
    if (!stepNote.trim()) return;
    appendEvent({
      id: crypto.randomUUID(),
      ts: new Date().toISOString(),
      actor: "local-user",
      action: "NOTES",
      scope: `incident:${selected?.name ?? "unknown"}`,
      reason: "Incident annotation",
      ttlMinutes: 0,
      result: "SUCCESS",
      details: stepNote.trim()
    });
    setStepNote("");
  }

  return (
    <div>
      <div className="page-header">
        <div className="page-header-left">
          <h1 className="page-title">Incident Console</h1>
          <p className="page-subtitle">Degraded reasons, drilldown, and personal checks.</p>
        </div>
        <div className="page-header-actions">
          <SaveSnapshotButton
            workspaceId="incident-console"
            pageId="overview"
            globalFilters={{}}
            overallStatus={summary?.overall_status ?? "UNAVAILABLE"}
            components={summary?.components ?? []}
          />
          <button className="btn" onClick={() => void refresh()}>Refresh</button>
        </div>
      </div>

      <div className="card" style={{ marginBottom: "var(--space-4)" }}>
        <p><strong>Overall:</strong> {error ? "UNAVAILABLE" : summary?.overall_status ?? "unknown"}</p>
        <p><strong>Last refresh:</strong> {lastRefresh ? formatTimestamp(lastRefresh) : "-"}</p>
        {error && <p className="text-error">{error}</p>}
      </div>

      <div className="card" style={{ marginBottom: "var(--space-4)" }}>
        <h2 className="card-title">Degraded Components</h2>
        {degraded.length === 0 ? <p className="text-muted">No degraded components reported.</p> : (
          <div className="table-wrapper">
            <table className="table">
              <thead><tr><th>Name</th><th>Status</th><th>Reason</th></tr></thead>
              <tbody>
                {degraded.map((item, idx) => (
                  <tr key={`${item.name ?? "component"}-${idx}`} onClick={() => setSelected(item)} style={{ cursor: "pointer" }}>
                    <td>{item.name ?? "unknown"}</td>
                    <td><span className="badge badge-warning">{item.status ?? "degraded"}</span></td>
                    <td>{item.reason ?? "n/a"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {selected && (
        <div className="card">
          <h2 className="card-title">Component Drilldown: {selected.name}</h2>
          <p><strong>Reason:</strong> {selected.reason ?? "No reason provided"}</p>
          <p><strong>Timestamp:</strong> {formatTimestamp(selected.updated_at ?? selected.ts ?? new Date().toISOString())}</p>

          <h3 style={{ marginTop: "var(--space-3)" }}>Suggested checks</h3>
          {CHECKLIST.map((step) => (
            <label key={step} style={{ display: "flex", gap: "var(--space-2)", alignItems: "center", marginBottom: "var(--space-1)" }}>
              <input type="checkbox" checked={Boolean(checklist[step])} onChange={() => toggleStep(step)} />
              <span>{step}</span>
            </label>
          ))}

          <div style={{ marginTop: "var(--space-3)", display: "grid", gap: "var(--space-2)" }}>
            <textarea rows={3} placeholder="Add incident note" value={stepNote} onChange={(e) => setStepNote(e.target.value)} />
            <button className="btn" onClick={saveStepNote}>Save Note</button>
          </div>
        </div>
      )}
    </div>
  );
}
