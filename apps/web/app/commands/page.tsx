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

type CommandSafetyPolicy = {
  enforced: boolean;
};

function parseCommandSafetyPolicy(payload: unknown): CommandSafetyPolicy {
  if (!payload || typeof payload !== "object") {
    return { enforced: false };
  }

  const source = payload as Record<string, unknown>;
  const commandSafety =
    source.command_safety && typeof source.command_safety === "object"
      ? (source.command_safety as Record<string, unknown>)
      : null;

  const rootReasonRequired = source.reason_required === true;
  const rootEnforced = source.enforced === true;
  const nestedReasonRequired = commandSafety?.reason_required === true;
  const nestedEnforced = commandSafety?.enforced === true;
  const features = Array.isArray(source.features) ? source.features : [];
  const featureEnforced = features.includes("commands.reason_required") || features.includes("commands.enforced");

  return {
    enforced: rootReasonRequired || rootEnforced || nestedReasonRequired || nestedEnforced || featureEnforced
  };
}

export default function CommandsPage() {
  const [botId, setBotId] = useState("simple-mm");
  const [activeBotId, setActiveBotId] = useState("simple-mm");
  const [commands, setCommands] = useState<Command[]>([]);
  const [loading, setLoading] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [reason, setReason] = useState("");
  const [expiresAtLocal, setExpiresAtLocal] = useState("");
  const [safetyPolicy, setSafetyPolicy] = useState<CommandSafetyPolicy>({ enforced: false });
  const [confirmType, setConfirmType] = useState<"PAUSE" | "RESUME" | null>(null);
  const [confirmStep, setConfirmStep] = useState<1 | 2>(1);

  const parsedExpiresAtMs = expiresAtLocal ? Date.parse(expiresAtLocal) : Number.NaN;
  const expiresAtIsValid = Number.isFinite(parsedExpiresAtMs) && parsedExpiresAtMs > Date.now();
  const reasonIsValid = reason.trim().length > 0;
  const isSubmissionBlocked = safetyPolicy.enforced && (!reasonIsValid || !expiresAtIsValid);

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

  useEffect(() => {
    async function loadCapabilities() {
      try {
        const res = await fetch("/api/capabilities", { cache: "no-store" });
        if (!res.ok) {
          setSafetyPolicy({ enforced: false });
          return;
        }
        const payload = await res.json();
        setSafetyPolicy(parseCommandSafetyPolicy(payload));
      } catch {
        setSafetyPolicy({ enforced: false });
      }
    }

    void loadCapabilities();
  }, []);

  async function issueCommand(type: "PAUSE" | "RESUME") {
    const expiresAtIso = expiresAtLocal ? new Date(expiresAtLocal).toISOString() : null;
    const commandPayload: Record<string, string> = {};
    if (reason.trim()) {
      commandPayload.reason = reason.trim();
    }
    if (expiresAtIso) {
      commandPayload.expires_at = expiresAtIso;
    }

    setSubmitting(true);
    try {
      const res = await fetch("/api/commands", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          type,
          target_bot_id: activeBotId,
          payload: commandPayload
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

  function openConfirm(type: "PAUSE" | "RESUME") {
    setConfirmType(type);
    setConfirmStep(1);
  }

  async function onConfirmAction() {
    if (!confirmType || submitting || isSubmissionBlocked) {
      return;
    }

    if (confirmStep === 1) {
      setConfirmStep(2);
      return;
    }

    await issueCommand(confirmType);
    setConfirmType(null);
    setConfirmStep(1);
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
        <button type="button" onClick={() => openConfirm("PAUSE")} disabled={submitting || isSubmissionBlocked}>
          PAUSE
        </button>
        <button type="button" onClick={() => openConfirm("RESUME")} disabled={submitting || isSubmissionBlocked}>
          RESUME
        </button>
      </div>

      <div style={{ display: "grid", gap: 8 }}>
        <label>
          <div>Reason {safetyPolicy.enforced ? "(required)" : "(optional)"}</div>
          <input
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            placeholder="Why is this command being issued?"
          />
        </label>
        <label>
          <div>Expires At {safetyPolicy.enforced ? "(required, future)" : "(optional)"}</div>
          <input type="datetime-local" value={expiresAtLocal} onChange={(e) => setExpiresAtLocal(e.target.value)} />
        </label>
        {safetyPolicy.enforced && !reasonIsValid ? <small>Reason is required by current command-safety policy.</small> : null}
        {safetyPolicy.enforced && !expiresAtIsValid ? <small>Expires At must be a valid future timestamp.</small> : null}
      </div>

      {confirmType ? (
        <div
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0, 0, 0, 0.65)",
            display: "grid",
            placeItems: "center",
            zIndex: 50
          }}
        >
          <div className="card" style={{ width: "min(520px, 92vw)", display: "grid", gap: 10 }}>
            <h3 style={{ margin: 0 }}>Confirm command execution</h3>
            <p style={{ margin: 0 }}>
              You are about to execute <strong>{confirmType}</strong> for target bot <strong>{activeBotId}</strong>.
            </p>
            <p style={{ margin: 0, color: "#fecaca" }}>
              Warning: this action can immediately impact live behavior. Verify reason/expiry and confirm twice.
            </p>
            <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
              <button
                type="button"
                onClick={() => {
                  setConfirmType(null);
                  setConfirmStep(1);
                }}
                disabled={submitting}
              >
                Cancel
              </button>
              <button type="button" onClick={() => void onConfirmAction()} disabled={submitting || isSubmissionBlocked}>
                {confirmStep === 1 ? "Confirm (1/2)" : "Confirm (2/2) & Execute"}
              </button>
            </div>
          </div>
        </div>
      ) : null}

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
