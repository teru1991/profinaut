"use client";

import { useEffect, useMemo, useState } from "react";

type ExecuteResponse = {
  ok: boolean;
  status: number;
  payload: unknown;
};

type ConfirmationChallenge = {
  token: string;
  expiresAtMs: number;
};

type Props = {
  actionLabel: string;
  targetLabel: string;
  environmentLabel: string;
  open: boolean;
  submitting: boolean;
  onClose: () => void;
  onExecute: (params: { reason: string; confirmToken?: string }) => Promise<ExecuteResponse>;
  onSuccess: () => Promise<void>;
};

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== "object") return null;
  return value as Record<string, unknown>;
}

function parseCode(payload: unknown): string | null {
  const record = asRecord(payload);
  if (!record) return null;
  const code = record.code;
  return typeof code === "string" ? code : null;
}

function parseMessage(payload: unknown, fallback: string): string {
  const record = asRecord(payload);
  if (!record) return fallback;
  const message = record.message ?? record.error;
  return typeof message === "string" && message.length > 0 ? message : fallback;
}

function parseChallenge(payload: unknown): ConfirmationChallenge | null {
  const record = asRecord(payload);
  if (!record) return null;
  const confirmation = asRecord(record.confirmation);
  const challenge = asRecord(record.challenge);
  const candidates = [
    record.confirm_token,
    confirmation?.token,
    challenge?.confirm_token,
    challenge?.token
  ];
  const token = candidates.find((v) => typeof v === "string" && v.length > 0);
  const expiresCandidates = [
    record.confirm_expires_at,
    record.expires_at,
    confirmation?.expires_at,
    challenge?.expires_at
  ];
  const expiresRaw = expiresCandidates.find((v) => typeof v === "string" && v.length > 0);
  const expiresAtMs = typeof expiresRaw === "string" ? Date.parse(expiresRaw) : Number.NaN;
  if (typeof token !== "string" || !Number.isFinite(expiresAtMs)) return null;
  return { token, expiresAtMs };
}

function formatCountdown(ms: number): string {
  const seconds = Math.max(0, Math.ceil(ms / 1000));
  const minutes = Math.floor(seconds / 60);
  const remainder = seconds % 60;
  return `${String(minutes).padStart(2, "0")}:${String(remainder).padStart(2, "0")}`;
}

export function DangerousActionDialog({
  actionLabel,
  targetLabel,
  environmentLabel,
  open,
  submitting,
  onClose,
  onExecute,
  onSuccess
}: Props) {
  const [reason, setReason] = useState("");
  const [challenge, setChallenge] = useState<ConfirmationChallenge | null>(null);
  const [nowMs, setNowMs] = useState(() => Date.now());
  const [error, setError] = useState<string | null>(null);

  const reasonIsValid = reason.trim().length > 0;
  const countdownMs = challenge ? challenge.expiresAtMs - nowMs : 0;

  useEffect(() => {
    if (!open) {
      setReason("");
      setChallenge(null);
      setError(null);
      return;
    }
    const id = setInterval(() => setNowMs(Date.now()), 500);
    return () => clearInterval(id);
  }, [open]);

  useEffect(() => {
    if (challenge && countdownMs <= 0) {
      setChallenge(null);
      setError("Confirmation expired. Please submit again to request a new challenge.");
    }
  }, [challenge, countdownMs]);

  const confirmLabel = useMemo(() => {
    if (!challenge) return "Submit (Step 1/2)";
    return "Confirm & Execute (Step 2/2)";
  }, [challenge]);

  async function handleSubmit() {
    if (submitting || !reasonIsValid) return;
    const response = await onExecute({ reason: reason.trim(), confirmToken: challenge?.token });
    const code = parseCode(response.payload);

    if (!response.ok && code === "CONFIRMATION_REQUIRED") {
      const parsedChallenge = parseChallenge(response.payload);
      if (!parsedChallenge) {
        setChallenge(null);
        setError("Server requested confirmation but did not provide a valid token or expiry.");
        return;
      }
      setChallenge(parsedChallenge);
      setError(null);
      return;
    }

    if (!response.ok) {
      const message = parseMessage(response.payload, `Request failed (${response.status})`);
      setError(code ? `${code}: ${message}` : message);
      if (code === "CONFIRMATION_EXPIRED" || code === "CONFIRM_TOKEN_EXPIRED") {
        setChallenge(null);
      }
      return;
    }

    setChallenge(null);
    setError(null);
    await onSuccess();
    onClose();
  }

  if (!open) return null;

  return (
    <div className="dialog-overlay" role="dialog" aria-modal="true" aria-label="Dangerous operation confirmation">
      <div className="dialog">
        <h3 className="dialog-title">Dangerous Operation</h3>

        <p className="text-sm" style={{ margin: 0 }}>
          You are about to run <strong>{actionLabel}</strong> against{" "}
          <strong>{targetLabel}</strong> in{" "}
          <strong>{environmentLabel.toUpperCase()}</strong> environment.
        </p>

        <div className="notice notice-error">
          <span className="notice-icon">!</span>
          <div className="notice-content">
            This can immediately change live behavior and may trigger irreversible side effects.
            Review reason and confirm deliberately.
          </div>
        </div>

        <label style={{ display: "grid", gap: "var(--space-2)" }}>
          <span>Reason (required)</span>
          <textarea
            rows={3}
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            placeholder="Describe why this operation is required and what safeguards were verified."
          />
        </label>
        {!reasonIsValid && (
          <span className="text-xs text-muted">A reason is required before submitting.</span>
        )}

        {challenge ? (
          <div className="notice notice-error">
            <span className="notice-icon">!</span>
            <div className="notice-content">
              <div className="notice-title">Step 2: Confirm execution</div>
              Challenge expires in <strong className="tabular-nums">{formatCountdown(countdownMs)}</strong>
            </div>
          </div>
        ) : (
          <div className="notice notice-warning">
            <span className="notice-icon">!</span>
            <div className="notice-content">
              Submit once to request a server confirmation challenge. You must confirm before expiry.
            </div>
          </div>
        )}

        {error && (
          <div className="error-state">
            <p className="error-state-title">Error</p>
            <p className="error-state-message">{error}</p>
          </div>
        )}

        <div className="dialog-footer">
          <button className="btn" type="button" onClick={onClose} disabled={submitting}>
            Cancel
          </button>
          <button
            className={`btn ${challenge ? "btn-danger" : "btn-primary"}`}
            type="button"
            onClick={() => void handleSubmit()}
            disabled={submitting || !reasonIsValid}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
