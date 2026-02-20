export type HealthStatus = "OK" | "DEGRADED" | "DOWN" | "UNKNOWN";

export function normalizeStatus(status: string | null | undefined): HealthStatus {
  const normalized = (status ?? "UNKNOWN").toUpperCase();
  if (normalized === "OK" || normalized === "DEGRADED" || normalized === "DOWN") return normalized;
  return "UNKNOWN";
}

export function statusToBadgeClass(status: string | null | undefined): string {
  const normalized = normalizeStatus(status);
  if (normalized === "OK") return "badge badge-success";
  if (normalized === "DEGRADED") return "badge badge-warning";
  if (normalized === "DOWN") return "badge badge-error";
  return "badge badge-info";
}

export function formatAge(ms: number): string {
  if (!Number.isFinite(ms) || ms < 0) return "0s";
  const seconds = Math.floor(ms / 1000);
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h`;
  const days = Math.floor(hours / 24);
  return `${days}d`;
}
