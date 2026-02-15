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
  if (!value || typeof value !== "object") {
    return null;
  }
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

  const candidates = [
    record.confirm_token,
    asRecord(record.confirmation)?.token,
    asRecord(record.challenge)?.confirm_token,
    asRecord(record.challenge)?.token
  ];
  const token = candidates.find((value) => typeof value === "string" && value.length > 0);

  const expiresCandidates = [
    record.confirm_expires_at,
    record.expires_at,
    asRecord(record.confirmation)?.expires_at,
    asRecord(record.challenge)?.expires_at
  ];
  const expiresRaw = expiresCandidates.find((value) => typeof value === "string" && value.length > 0);
  const expiresAtMs = typeof expiresRaw === "string" ? Date.parse(expiresRaw) : Number.NaN;

  if (typeof token !== "string" || !Number.isFinite(expiresAtMs)) {
    return null;
  }

  return {
    token,
    expiresAtMs
  };
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
    if (submitting || !reasonIsValid) {
      return;
    }

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

  if (!open) {
    return null;
  }

  return (
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
      <div className="card" style={{ width: "min(620px, 92vw)", display: "grid", gap: 12 }}>
        <h3 style={{ margin: 0 }}>Dangerous operation confirmation</h3>
        <p style={{ margin: 0 }}>
          You are about to run <strong>{actionLabel}</strong> against <strong>{targetLabel}</strong> in
          <strong> {environmentLabel.toUpperCase()}</strong> environment.
        </p>
        <p style={{ margin: 0, color: "#fecaca" }}>
          This can immediately change live behavior and may trigger irreversible side effects. Review reason and confirm
          deliberately.
        </p>

        <label style={{ display: "grid", gap: 6 }}>
          <span>Reason (required)</span>
          <textarea
            rows={3}
            value={reason}
            onChange={(event) => setReason(event.target.value)}
            placeholder="Describe why this operation is required and what safeguards were verified."
          />
        </label>
        {!reasonIsValid ? <small>Reason is required.</small> : null}

        {challenge ? (
          <div style={{ border: "1px solid #7f1d1d", background: "#3f1d1d", borderRadius: 8, padding: 10 }}>
            <strong>Step 2 required by server</strong>
            <p style={{ margin: "6px 0 0" }}>Challenge expires in {formatCountdown(countdownMs)}.</p>
          </div>
        ) : (
          <div style={{ border: "1px solid #78350f", background: "#3f2a14", borderRadius: 8, padding: 10 }}>
            Submit once to request a server confirmation challenge. You must confirm before expiry.
          </div>
        )}

        {error ? (
          <div style={{ border: "1px solid #7f1d1d", background: "#3f1d1d", borderRadius: 8, padding: 10 }}>
            <strong>Error</strong>
            <p style={{ margin: "6px 0 0" }}>{error}</p>
          </div>
        ) : null}

        <div style={{ display: "flex", justifyContent: "flex-end", gap: 8 }}>
          <button type="button" onClick={onClose} disabled={submitting}>
            Cancel
          </button>
          <button type="button" onClick={() => void handleSubmit()} disabled={submitting || !reasonIsValid}>
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
