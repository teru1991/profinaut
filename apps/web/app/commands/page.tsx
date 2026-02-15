"use client";

import { FormEvent, useEffect, useMemo, useState } from "react";

import { DangerousActionDialog } from "../../components/DangerousActionDialog";

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
  dangerousActionsEnabled: boolean;
  environmentLabel: string;
};

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== "object") {
    return null;
  }
  return value as Record<string, unknown>;
}

function firstBoolean(values: unknown[]): boolean | null {
  for (const value of values) {
    if (typeof value === "boolean") {
      return value;
    }
  }
  return null;
}

function parseCapabilities(payload: unknown): CommandSafetyPolicy {
  const source = asRecord(payload);
  if (!source) {
    return { dangerousActionsEnabled: false, environmentLabel: "unknown" };
  }

  const features = Array.isArray(source.features) ? source.features : [];
  const dangerousOps = asRecord(source.dangerous_operations);
  const commandSafety = asRecord(source.command_safety);

  const enabled = firstBoolean([
    dangerousOps?.enabled,
    commandSafety?.enabled,
    source.dangerous_actions_enabled,
    source.commands_enabled,
    source.command_execution_enabled
  ]);

  const disabledByFeature =
    features.includes("dangerous_ops.disabled") ||
    features.includes("commands.disabled") ||
    features.includes("command_execution.disabled");

  const environmentCandidate =
    (typeof source.environment === "string" && source.environment) ||
    (typeof source.mode === "string" && source.mode) ||
    (typeof source.venue === "string" && source.venue) ||
    (typeof source.profile === "string" && source.profile) ||
    "unknown";

  return {
    dangerousActionsEnabled: enabled === null ? false : enabled,
    environmentLabel: environmentCandidate
  };
}

function parseError(payload: unknown, status: number, fallback: string): string {
  const record = asRecord(payload);
  if (!record) {
    return fallback;
  }
  const code = typeof record.code === "string" ? record.code : null;
  const message =
    typeof record.message === "string"
      ? record.message
      : typeof record.error === "string"
        ? record.error
        : fallback;
  return code ? `${code}: ${message}` : `${message} (${status})`;
}

export default function CommandsPage() {
  const [botId, setBotId] = useState("simple-mm");
  const [activeBotId, setActiveBotId] = useState("simple-mm");
  const [commands, setCommands] = useState<Command[]>([]);
  const [loading, setLoading] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [safetyPolicy, setSafetyPolicy] = useState<CommandSafetyPolicy>({
    dangerousActionsEnabled: true,
    environmentLabel: "unknown"
  });
  const [dialogType, setDialogType] = useState<"PAUSE" | "RESUME" | null>(null);

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
        setError(parseError(payload, res.status, `Failed to load commands (${res.status})`));
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
          setSafetyPolicy({ dangerousActionsEnabled: true, environmentLabel: "unknown" });
          return;
        }
        const payload = await res.json();
        setSafetyPolicy(parseCapabilities(payload));
      } catch {
        setSafetyPolicy({ dangerousActionsEnabled: true, environmentLabel: "unknown" });
      }
    }

    void loadCapabilities();
  }, []);

  async function issueCommand(type: "PAUSE" | "RESUME", reason: string, confirmToken?: string) {
    const commandPayload: Record<string, string> = {
      reason
    };

    const requestBody: Record<string, unknown> = {
      type,
      target_bot_id: activeBotId,
      payload: commandPayload
    };

    if (confirmToken) {
      requestBody.confirm_token = confirmToken;
    }

    setSubmitting(true);
    try {
      const res = await fetch("/api/commands", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(requestBody)
      });

      let payload: unknown = null;
      try {
        payload = await res.json();
      } catch {
        payload = null;
      }

      return { ok: res.ok, status: res.status, payload };
    } catch (e) {
      setError(e instanceof Error ? e.message : `Failed to issue ${type}`);
      return { ok: false, status: 0, payload: { message: `Failed to issue ${type}` } };
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
      <p>Issue dangerous PAUSE/RESUME commands and monitor latest ack for a bot.</p>

      <form onSubmit={onApplyBotId} style={{ display: "flex", gap: 8, alignItems: "end", flexWrap: "wrap" }}>
        <label>
          <div>Bot ID</div>
          <input value={botId} onChange={(e) => setBotId(e.target.value)} placeholder="simple-mm" />
        </label>
        <button type="submit">Apply Bot</button>
      </form>

      <div style={{ display: "flex", gap: 8 }}>
        <button
          type="button"
          onClick={() => setDialogType("PAUSE")}
          disabled={submitting || !safetyPolicy.dangerousActionsEnabled}
          title={safetyPolicy.dangerousActionsEnabled ? "" : "Disabled by policy"}
        >
          PAUSE
        </button>
        <button
          type="button"
          onClick={() => setDialogType("RESUME")}
          disabled={submitting || !safetyPolicy.dangerousActionsEnabled}
          title={safetyPolicy.dangerousActionsEnabled ? "" : "Disabled by policy"}
        >
          RESUME
        </button>
      </div>

      {!safetyPolicy.dangerousActionsEnabled ? (
        <small style={{ color: "#fca5a5" }}>Dangerous command actions are disabled by policy.</small>
      ) : null}

      {dialogType ? (
        <DangerousActionDialog
          actionLabel={dialogType}
          targetLabel={`bot:${activeBotId}`}
          environmentLabel={safetyPolicy.environmentLabel}
          open={dialogType !== null}
          submitting={submitting}
          onClose={() => setDialogType(null)}
          onExecute={(params) => issueCommand(dialogType, params.reason, params.confirmToken)}
          onSuccess={async () => {
            setError(null);
            await loadCommands(activeBotId);
          }}
        />
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
                <td>{cmd.ack ? `${cmd.ack.ok ? "ok" : "nack"}${cmd.ack.reason ? ` (${cmd.ack.reason})` : ""}` : "-"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      ) : null}
    </div>
  );
}
