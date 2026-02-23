"use client";

import { useEffect, useMemo, useState } from "react";

// --- Types ---

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
  /**
   * 親コンポーネントから渡される実行関数。
   * 親コンポーネントも "use client" である必要があります。
   */
  onExecute: (params: { reason: string; confirmToken?: string }) => Promise<ExecuteResponse>;
  onSuccess: () => Promise<void>;
};

// --- Helpers ---

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== "object" || value === null) return null;
  return value as Record<string, unknown>;
}

function parseCode(payload: unknown): string | null {
  const record = asRecord(payload);
  if (!record) return null;
  return typeof record.code === "string" ? record.code : null;
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

  const token = [
    record.confirm_token,
    confirmation?.token,
    challenge?.confirm_token,
    challenge?.token
  ].find((v): v is string => typeof v === "string" && v.length > 0);

  const expiresRaw = [
    record.confirm_expires_at,
    record.expires_at,
    confirmation?.expires_at,
    challenge?.expires_at
  ].find((v): v is string => typeof v === "string" && v.length > 0);

  const expiresAtMs = expiresRaw ? Date.parse(expiresRaw) : Number.NaN;

  if (!token || !Number.isFinite(expiresAtMs)) return null;
  return { token, expiresAtMs };
}

function formatCountdown(ms: number): string {
  const seconds = Math.max(0, Math.ceil(ms / 1000));
  const minutes = Math.floor(seconds / 60);
  const remainder = seconds % 60;
  return `${String(minutes).padStart(2, "0")}:${String(remainder).padStart(2, "0")}`;
}

// --- Component ---

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

  // Reset state and handle timer
  useEffect(() => {
    if (!open) {
      setReason("");
      setChallenge(null);
      setError(null);
      return;
    }
    const id = setInterval(() => setNowMs(Date.now()), 1000);
    return () => clearInterval(id);
  }, [open]);

  // Handle challenge expiration
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

  const handleSubmit = async () => {
    if (submitting || !reasonIsValid) return;

    try {
      const response = await onExecute({
        reason: reason.trim(),
        confirmToken: challenge?.token
      });

      const code = parseCode(response.payload);

      if (!response.ok) {
        // Handle two-step confirmation flow
        if (code === "CONFIRMATION_REQUIRED") {
          const parsedChallenge = parseChallenge(response.payload);
          if (!parsedChallenge) {
            setChallenge(null);
            setError("Server requested confirmation but did not provide a valid token.");
            return;
          }
          setChallenge(parsedChallenge);
          setError(null);
          return;
        }

        // General error handling
        const message = parseMessage(response.payload, `Request failed (${response.status})`);
        setError(code ? `${code}: ${message}` : message);

        if (code === "CONFIRMATION_EXPIRED" || code === "CONFIRM_TOKEN_EXPIRED") {
          setChallenge(null);
        }
        return;
      }

      // Success
      setChallenge(null);
      setError(null);
      await onSuccess();
      onClose();
    } catch (e) {
      setError(e instanceof Error ? e.message : "An unexpected error occurred.");
    }
  };

  if (!open) return null;

  return (
      <div className="dialog-overlay" role="dialog" aria-modal="true">
        <div className="dialog">
          <h3 className="dialog-title">Dangerous Operation</h3>

          <p className="text-sm">
            You are about to run <strong>{actionLabel}</strong> against{" "}
            <strong>{targetLabel}</strong> in{" "}
            <strong style={{ color: "red" }}>{environmentLabel.toUpperCase()}</strong>.
          </p>

          <div className="notice notice-error" style={{ border: "1px solid red", padding: "8px", marginBottom: "12px" }}>
            <strong>⚠️ Warning:</strong> This can immediately change live behavior. Review your reason carefully.
          </div>

          <div style={{ display: "grid", gap: "8px", marginBottom: "16px" }}>
            <label htmlFor="execution-reason">Reason (required)</label>
            <textarea
                id="execution-reason"
                rows={3}
                value={reason}
                onChange={(e) => setReason(e.target.value)}
                placeholder="Describe why this operation is required..."
                disabled={submitting}
            />
            {!reasonIsValid && (
                <span className="text-xs" style={{ color: "gray" }}>A reason is required before submitting.</span>
            )}
          </div>

          {challenge ? (
              <div className="notice challenge-active" style={{ backgroundColor: "#fff5f5", padding: "8px", marginBottom: "12px" }}>
                <strong>Step 2: Confirm execution</strong><br />
                Expires in: <span className="tabular-nums" style={{ fontWeight: "bold" }}>{formatCountdown(countdownMs)}</span>
              </div>
          ) : (
              <div className="notice info" style={{ fontSize: "0.85rem", color: "#666", marginBottom: "12px" }}>
                Submit once to request a server challenge.
              </div>
          )}

          {error && (
              <div className="error-box" style={{ color: "red", marginBottom: "12px", fontSize: "0.9rem" }}>
                <strong>Error:</strong> {error}
              </div>
          )}

          <div className="dialog-footer" style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
            <button type="button" onClick={onClose} disabled={submitting}>
              Cancel
            </button>
            <button
                className={`btn ${challenge ? "btn-danger" : "btn-primary"}`}
                type="button"
                onClick={handleSubmit}
                disabled={submitting || !reasonIsValid}
            >
              {submitting ? "Processing..." : confirmLabel}
            </button>
          </div>
        </div>
      </div>
  );
}