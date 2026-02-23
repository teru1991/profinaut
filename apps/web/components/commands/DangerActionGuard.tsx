"use client";

import { useState } from "react";

import { appendEvent } from "../audit/auditStore";

type Props = {
  actionLabel: string;
  scope: string;
  payload: Record<string, unknown>;
  disabled?: boolean;
  buttonClassName?: string;
  onComplete?: () => Promise<void> | void;
};

function readErrorMessage(payload: unknown, fallback: string) {
  if (!payload || typeof payload !== "object") return fallback;
  const record = payload as Record<string, unknown>;
  const message = typeof record.message === "string" ? record.message : typeof record.error === "string" ? record.error : fallback;
  return message;
}

export function DangerActionGuard({ actionLabel, scope, payload, disabled, buttonClassName = "btn btn-danger", onComplete }: Props) {
  const [open, setOpen] = useState(false);
  const [reason, setReason] = useState("");
  const [ttlMinutes, setTtlMinutes] = useState(30);
  const [confirmText, setConfirmText] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const valid = reason.trim().length > 0 && [5, 15, 30, 60].includes(ttlMinutes) && confirmText.trim() === "CONFIRM";

  async function execute() {
    if (!valid) return;
    setSubmitting(true);
    setError(null);
    const ts = new Date().toISOString();
    try {
      const res = await fetch("/api/commands", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ ...payload, payload: { ...(payload.payload as Record<string, unknown> ?? {}), reason: reason.trim(), ttl_minutes: ttlMinutes } })
      });
      let body: unknown = null;
      try { body = await res.json(); } catch { body = null; }

      const record = body && typeof body === "object" ? body as Record<string, unknown> : null;
      const commandId = typeof record?.id === "string" ? record.id : undefined;

      appendEvent({
        id: crypto.randomUUID(),
        ts,
        actor: "local-user",
        action: actionLabel,
        scope,
        reason: reason.trim(),
        ttlMinutes,
        result: res.ok ? "SUCCESS" : "FAILED",
        details: res.ok ? undefined : readErrorMessage(body, `Request failed (${res.status})`),
        command_id: commandId
      });

      if (!res.ok) {
        setError(readErrorMessage(body, `Request failed (${res.status})`));
        return;
      }

      setOpen(false);
      setReason("");
      setConfirmText("");
      await onComplete?.();
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to execute command";
      appendEvent({
        id: crypto.randomUUID(),
        ts,
        actor: "local-user",
        action: actionLabel,
        scope,
        reason: reason.trim(),
        ttlMinutes,
        result: "FAILED",
        details: message
      });
      setError(message);
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <>
      <button className={buttonClassName} type="button" disabled={disabled} onClick={() => setOpen(true)}>{actionLabel}</button>
      {open && (
        <div className="dialog-overlay" role="dialog" aria-modal="true" aria-label={`${actionLabel} guard`}>
          <div className="dialog">
            <h3 className="dialog-title">{actionLabel} guard</h3>
            <p className="text-sm">Provide reason, TTL, and type <strong>CONFIRM</strong> to continue.</p>
            <label style={{ display: "grid", gap: "var(--space-1)", marginBottom: "var(--space-2)" }}>
              <span>Reason</span>
              <textarea rows={3} value={reason} onChange={(e) => setReason(e.target.value)} />
            </label>
            <label style={{ display: "grid", gap: "var(--space-1)", marginBottom: "var(--space-2)" }}>
              <span>TTL minutes</span>
              <select value={ttlMinutes} onChange={(e) => setTtlMinutes(Number(e.target.value))}>
                <option value={5}>5</option><option value={15}>15</option><option value={30}>30</option><option value={60}>60</option>
              </select>
            </label>
            <label style={{ display: "grid", gap: "var(--space-1)" }}>
              <span>Type CONFIRM</span>
              <input value={confirmText} onChange={(e) => setConfirmText(e.target.value)} placeholder="CONFIRM" />
            </label>
            {error && <p className="text-error">{error}</p>}
            <div className="dialog-footer">
              <button className="btn" type="button" onClick={() => setOpen(false)}>Cancel</button>
              <button className="btn btn-danger" type="button" disabled={!valid || submitting} onClick={() => void execute()}>Execute</button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
