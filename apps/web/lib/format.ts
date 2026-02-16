/** Format a number with locale-aware grouping */
export function formatNumber(value: number | null | undefined, decimals = 2): string {
  if (value == null || !Number.isFinite(value)) return "-";
  return value.toLocaleString("en-US", {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals
  });
}

/** Format a number as compact (e.g. 1.2K, 3.4M) */
export function formatCompact(value: number | null | undefined): string {
  if (value == null || !Number.isFinite(value)) return "-";
  if (Math.abs(value) >= 1e6) return `${(value / 1e6).toFixed(2)}M`;
  if (Math.abs(value) >= 1e3) return `${(value / 1e3).toFixed(1)}K`;
  return value.toFixed(2);
}

/** Format a number as currency (JPY-style, no decimals by default) */
export function formatCurrency(value: number | null | undefined, currency = "JPY"): string {
  if (value == null || !Number.isFinite(value)) return "-";
  const decimals = currency === "JPY" ? 0 : 2;
  const prefix = currency === "JPY" ? "\u00a5" : "$";
  return `${prefix}${value.toLocaleString("en-US", { minimumFractionDigits: decimals, maximumFractionDigits: decimals })}`;
}

/** Format basis points */
export function formatBps(value: number | null | undefined): string {
  if (value == null || !Number.isFinite(value)) return "-";
  return `${value.toFixed(1)} bps`;
}

/** Format a percentage */
export function formatPct(value: number | null | undefined, decimals = 2): string {
  if (value == null || !Number.isFinite(value)) return "-";
  return `${value.toFixed(decimals)}%`;
}

/** Format a duration in ms to human-readable */
export function formatDuration(ms: number | null | undefined): string {
  if (ms == null || !Number.isFinite(ms)) return "-";
  if (ms < 1000) return `${Math.round(ms)}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  const minutes = Math.floor(ms / 60000);
  const seconds = Math.round((ms % 60000) / 1000);
  return `${minutes}m ${seconds}s`;
}

/** Format ISO timestamp to local time string */
export function formatTimestamp(iso: string | null | undefined): string {
  if (!iso) return "-";
  try {
    const date = new Date(iso);
    if (isNaN(date.getTime())) return iso;
    return date.toLocaleString("en-US", {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hour12: false
    });
  } catch {
    return iso;
  }
}

/** Format ISO timestamp to short relative time */
export function formatRelative(iso: string | null | undefined): string {
  if (!iso) return "-";
  try {
    const date = new Date(iso);
    if (isNaN(date.getTime())) return iso;
    const diff = Date.now() - date.getTime();
    if (diff < 0) return "just now";
    if (diff < 60000) return `${Math.floor(diff / 1000)}s ago`;
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return `${Math.floor(diff / 86400000)}d ago`;
  } catch {
    return iso;
  }
}

/** Copy text to clipboard, returns success boolean */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}
