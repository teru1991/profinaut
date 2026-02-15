"use client";

import { FormEvent, useEffect, useMemo, useState } from "react";

type Ack = {
  command_id: string;
  ok: boolean;
  reason?: string | null;
  ts: string;
};

type Command = {
  id: string;
  type: string;
  target_bot_id: string;
  payload?: Record<string, unknown>;
  status: "pending" | "applied" | "nack";
  created_at: string;
  ack?: Ack | null;
};

export default function CommandsPage() {
  const [botId, setBotId] = useState("simple-mm");
  const [activeBotId, setActiveBotId] = useState("simple-mm");
  const [commands, setCommands] = useState<Command[]>([]);
  const [loading, setLoading] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const lastAck = useMemo(() => {
    for (const cmd of commands) {
      if (cmd.ack) return cmd.ack;
    }
    return null;
  }, [commands]);

  async function loadCommands(target = activeBotId) {
    setLoading(true);
    try {
      const res = await fetch(`/api/commands?target_bot_id=${encodeURIComponent(target)}`, { cache: "no-store" });
      const payload = await res.json();

      if (!res.ok) {
        setError(payload?.message ?? payload?.error ?? `Failed to load commands (${res.status})`);
        setCommands([]);
        return;
      }

      const items = Array.isArray(payload) ? (payload as Command[]) : [];
      setCommands(items);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load commands");
      setCommands([]);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void loadCommands();
    const timer = setInterval(() => void loadCommands(), 4000);
    return () => clearInterval(timer);
  }, [activeBotId]);

  async function issueCommand(type: "PAUSE" | "RESUME") {
    setSubmitting(true);
    try {
      const res = await fetch("/api/commands", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          type,
          target_bot_id: activeBotId,
          payload: {}
        })
      });
      const payload = await res.json();

      if (!res.ok) {
        setError(payload?.message ?? payload?.error ?? `Failed to issue ${type} (${res.status})`);
        return;
      }

      setError(null);
      await loadCommands(activeBotId);
    } catch (e) {
      setError(e instanceof Error ? e.message : `Failed to issue ${type}`);
    } finally {
      setSubmitting(false);
    }
  }

  function onApplyBotId(e: FormEvent) {
    e.preventDefault();
    const next = botId.trim() || "simple-mm";
    setBotId(next);
    setActiveBotId(next);
  }

  return (
    <div className="card" style={{ display: "grid", gap: 12 }}>
      <h2>Commands</h2>
      <p>Issue PAUSE/RESUME commands and monitor latest ack for a bot.</p>

      <form onSubmit={onApplyBotId} style={{ display: "flex", gap: 8, alignItems: "end", flexWrap: "wrap" }}>
        <label>
          <div>Bot ID</div>
          <input value={botId} onChange={(e) => setBotId(e.target.value)} placeholder="simple-mm" />
        </label>
        <button type="submit">Apply Bot</button>
      </form>

      <div style={{ display: "flex", gap: 8 }}>
        <button type="button" onClick={() => void issueCommand("PAUSE")} disabled={submitting}>
          PAUSE
        </button>
        <button type="button" onClick={() => void issueCommand("RESUME")} disabled={submitting}>
          RESUME
        </button>
      </div>

      {error ? (
        <div style={{ border: "1px solid #7f1d1d", background: "#3f1d1d", borderRadius: 8, padding: 10 }}>
          <strong>Error</strong>
          <p style={{ marginBottom: 0 }}>{error}</p>
        </div>
      ) : null}

      {loading ? <p>Loading commands...</p> : null}

      {!loading && commands.length === 0 ? <p>No commands found for bot_id={activeBotId}.</p> : null}

      {lastAck ? (
        <div className="card" style={{ background: "rgba(255,255,255,0.02)" }}>
          <strong>Last ack</strong>
          <p style={{ margin: "8px 0 0" }}>
            command_id={lastAck.command_id} ok={String(lastAck.ok)} reason={lastAck.reason ?? "-"} ts={lastAck.ts}
          </p>
        </div>
      ) : null}

      {commands.length > 0 ? (
        <table className="table">
          <thead>
            <tr>
              <th>Created</th>
              <th>Type</th>
              <th>Status</th>
              <th>Ack</th>
            </tr>
          </thead>
          <tbody>
            {commands.map((cmd) => (
              <tr key={cmd.id}>
                <td>{cmd.created_at}</td>
                <td>{cmd.type}</td>
                <td>{cmd.status}</td>
                <td>
                  {cmd.ack ? `${cmd.ack.ok ? "ok" : "nack"}${cmd.ack.reason ? ` (${cmd.ack.reason})` : ""}` : "-"}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      ) : null}
    </div>
  );
}
