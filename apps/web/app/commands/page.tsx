"use client";

import { FormEvent, useEffect, useMemo, useState } from "react";

import { DangerActionGuard } from "../../components/commands/DangerActionGuard";
import { formatTimestamp, copyToClipboard } from "../../lib/format";

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
  if (!value || typeof value !== "object") return null;
  return value as Record<string, unknown>;
}

function firstBoolean(values: unknown[]): boolean | null {
  for (const value of values) {
    if (typeof value === "boolean") return value;
  }
  return null;
}

function parseCapabilities(payload: unknown): CommandSafetyPolicy {
  const source = asRecord(payload);
  if (!source) return { dangerousActionsEnabled: false, environmentLabel: "unknown" };

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
    dangerousActionsEnabled: enabled === null ? false : enabled && !disabledByFeature,
    environmentLabel: environmentCandidate
  };
}

function parseError(payload: unknown, status: number, fallback: string): string {
  const record = asRecord(payload);
  if (!record) return fallback;
  const code = typeof record.code === "string" ? record.code : null;
  const message =
    typeof record.message === "string"
      ? record.message
      : typeof record.error === "string"
        ? record.error
        : fallback;
  return code ? `${code}: ${message}` : `${message} (${status})`;
}

function statusBadgeClass(status: string): string {
  switch (status) {
    case "applied": return "badge badge-success";
    case "nack": return "badge badge-error";
    case "pending": return "badge badge-warning";
    default: return "badge";
  }
}

function getCommandTypeBadgeClass(type: string): string {
  const badgeClassMap: Record<string, string> = {
    "PAUSE": "badge badge-warning",
    "RESUME": "badge badge-success",
  };

  return badgeClassMap[type] ?? "badge badge-neutral";
}

export default function CommandsPage() {
  const [botId, setBotId] = useState("simple-mm");
  const [activeBotId, setActiveBotId] = useState("simple-mm");
  const [commands, setCommands] = useState<Command[]>([]);
  const [loading, setLoading] = useState(true);
  const [submitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [safetyPolicy, setSafetyPolicy] = useState<CommandSafetyPolicy>({
    dangerousActionsEnabled: true,
    environmentLabel: "unknown"
  });
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const lastAck = useMemo(() => {
    for (const cmd of commands) {
      if (cmd.ack) return cmd.ack;
    }
    return null;
  }, [commands]);

  async function loadCommands(target = activeBotId) {
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
          setSafetyPolicy({ dangerousActionsEnabled: false, environmentLabel: "unknown" });
          return;
        }
        const payload = await res.json();
        setSafetyPolicy(parseCapabilities(payload));
      } catch {
        setSafetyPolicy({ dangerousActionsEnabled: false, environmentLabel: "unknown" });
      }
    }
    void loadCapabilities();
  }, []);

  function onApplyBotId(e: FormEvent) {
    e.preventDefault();
    const next = botId.trim() || "simple-mm";
    setBotId(next);
    setActiveBotId(next);
    setLoading(true);
  }

  async function handleCopyId(id: string) {
    const ok = await copyToClipboard(id);
    if (ok) {
      setCopiedId(id);
      setTimeout(() => setCopiedId(null), 1500);
    }
  }

  return (
    <div>
      <div className="page-header">
        <div className="page-header-left">
          <h1 className="page-title">Commands</h1>
          <p className="page-subtitle">Issue PAUSE/RESUME commands with 2-step confirmation</p>
        </div>
        <div className="page-header-actions">
          <span className="badge badge-accent">Target: {activeBotId}</span>
          <span className="badge">
            {safetyPolicy.environmentLabel.toUpperCase()}
          </span>
        </div>
      </div>

      {/* Bot Selector + Actions */}
      <div className="card" style={{ marginBottom: "var(--space-4)" }}>
        <div className="card-header">
          <h2 className="card-title">Bot Target & Actions</h2>
        </div>

        <form onSubmit={onApplyBotId} style={{ display: "flex", gap: "var(--space-3)", alignItems: "flex-end", flexWrap: "wrap", marginBottom: "var(--space-4)" }}>
          <label style={{ display: "grid", gap: "var(--space-1)" }}>
            <span className="text-xs text-muted">Bot ID</span>
            <input value={botId} onChange={(e) => setBotId(e.target.value)} placeholder="simple-mm" />
          </label>
          <button className="btn" type="submit">Apply Bot</button>
        </form>

        <div style={{ display: "flex", gap: "var(--space-2)", alignItems: "center", flexWrap: "wrap" }}>
          <DangerActionGuard
            actionLabel="PAUSE"
            scope={`bot:${activeBotId}`}
            payload={{ type: "PAUSE", target_bot_id: activeBotId, payload: {} }}
            disabled={submitting || !safetyPolicy.dangerousActionsEnabled}
            buttonClassName="btn btn-danger"
            onComplete={async () => {
              setError(null);
              await loadCommands(activeBotId);
            }}
          />
          <DangerActionGuard
            actionLabel="RESUME"
            scope={`bot:${activeBotId}`}
            payload={{ type: "RESUME", target_bot_id: activeBotId, payload: {} }}
            disabled={submitting || !safetyPolicy.dangerousActionsEnabled}
            buttonClassName="btn btn-success"
            onComplete={async () => {
              setError(null);
              await loadCommands(activeBotId);
            }}
          />

          {!safetyPolicy.dangerousActionsEnabled && (
            <span className="text-xs text-error">
              Dangerous command actions are disabled by policy.
            </span>
          )}
        </div>
      </div>

      {error && (
        <div className="error-state" style={{ marginBottom: "var(--space-4)" }}>
          <p className="error-state-title">Error</p>
          <p className="error-state-message">{error}</p>
        </div>
      )}

      {/* Last Ack */}
      {lastAck && (
        <div className="card card-compact" style={{ marginBottom: "var(--space-4)" }}>
          <div className="card-header" style={{ marginBottom: "var(--space-2)" }}>
            <h3 className="card-title">Last Acknowledgement</h3>
            <span className={`badge ${lastAck.ok ? "badge-success" : "badge-error"}`}>
              {lastAck.ok ? "OK" : "NACK"}
            </span>
          </div>
          <table className="table-inline">
            <tbody>
              <tr>
                <th>Command ID</th>
                <td>
                  <span className="text-mono">{lastAck.command_id.slice(0, 16)}</span>
                  <button
                    className="copy-btn"
                    onClick={() => handleCopyId(lastAck.command_id)}
                    style={{ marginLeft: "var(--space-2)" }}
                    aria-label="Copy command ID"
                  >
                    {copiedId === lastAck.command_id ? "\u2713" : "\u2398"}
                  </button>
                </td>
              </tr>
              <tr>
                <th>Reason</th>
                <td>{lastAck.reason ?? "-"}</td>
              </tr>
              <tr>
                <th>Timestamp</th>
                <td className="tabular-nums">{formatTimestamp(lastAck.ts)}</td>
              </tr>
            </tbody>
          </table>
        </div>
      )}

      {/* Commands Table */}
      <div className="card">
        <div className="card-header">
          <h2 className="card-title">Command History</h2>
          <span className="text-xs text-muted">Auto-refresh 4s</span>
        </div>

        {loading && commands.length === 0 && !error ? (
          <div>
            {[1, 2, 3].map((i) => (
              <div key={i} className="skeleton skeleton-row" />
            ))}
          </div>
        ) : commands.length === 0 && !error ? (
          <div className="empty-state">
            <div className="empty-state-icon">&#x1F4E8;</div>
            <h3 className="empty-state-title">No commands</h3>
            <p className="empty-state-description">
              No commands found for bot_id={activeBotId}. Issue a command above to get started.
            </p>
          </div>
        ) : commands.length > 0 ? (
          <div className="table-wrapper">
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
                    <td className="tabular-nums">{formatTimestamp(cmd.created_at)}</td>
                    <td>
                      <span className={getCommandTypeBadgeClass(cmd.type)}>
                        {cmd.type}
                      </span>
                    </td>
                    <td><span className={statusBadgeClass(cmd.status)}>{cmd.status}</span></td>
                    <td>
                      {cmd.ack ? (
                        <span className={`badge ${cmd.ack.ok ? "badge-success" : "badge-error"}`}>
                          {cmd.ack.ok ? "ok" : "nack"}
                          {cmd.ack.reason ? ` (${cmd.ack.reason})` : ""}
                        </span>
                      ) : (
                        <span className="text-muted">-</span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : null}
      </div>
    </div>
  );
}
