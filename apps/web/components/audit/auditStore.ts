import { AuditEvent, AuditFilters } from "./auditModel";

const AUDIT_STORAGE_KEY = "profinaut.audit.events.v1";
const MAX_EVENTS = 2000;

function isBrowser() {
  return typeof window !== "undefined" && typeof window.localStorage !== "undefined";
}

function readEvents(): AuditEvent[] {
  if (!isBrowser()) return [];
  try {
    const raw = window.localStorage.getItem(AUDIT_STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? (parsed as AuditEvent[]) : [];
  } catch {
    return [];
  }
}

function writeEvents(events: AuditEvent[]) {
  if (!isBrowser()) return;
  window.localStorage.setItem(AUDIT_STORAGE_KEY, JSON.stringify(events));
}

export function appendEvent(event: AuditEvent) {
  const events = readEvents();
  events.push(event);
  const capped = events.length > MAX_EVENTS ? events.slice(events.length - MAX_EVENTS) : events;
  writeEvents(capped);
}

export function listEvents(filters?: AuditFilters): AuditEvent[] {
  const events = readEvents();
  const filtered = events.filter((event) => {
    if (filters?.action && event.action !== filters.action) return false;
    if (filters?.result && event.result !== filters.result) return false;
    if (filters?.fromTs && event.ts < filters.fromTs) return false;
    if (filters?.toTs && event.ts > filters.toTs) return false;
    return true;
  });
  return filtered.sort((a, b) => b.ts.localeCompare(a.ts));
}

export function clearOld(maxDays: number) {
  const events = readEvents();
  const cutoffMs = Date.now() - maxDays * 24 * 60 * 60 * 1000;
  const kept = events.filter((event) => Date.parse(event.ts) >= cutoffMs);
  writeEvents(kept);
}

export function exportRawEvents(): AuditEvent[] {
  return readEvents();
}
