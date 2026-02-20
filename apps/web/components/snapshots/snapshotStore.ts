export type Snapshot = {
  id: string;
  ts: string;
  dashboardUrl: string;
  workspaceId: string;
  pageId: string;
  globalFilters: Record<string, string | undefined>;
  overall_status: string;
  degraded_components: Array<{ name: string; status: string; reason?: string; ts?: string }>;
  note?: string;
};

const SNAPSHOT_KEY = "profinaut.snapshots.v1";
const MAX_SNAPSHOTS = 500;

function isBrowser() {
  return typeof window !== "undefined" && typeof window.localStorage !== "undefined";
}

function readSnapshots(): Snapshot[] {
  if (!isBrowser()) return [];
  try {
    const raw = window.localStorage.getItem(SNAPSHOT_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? (parsed as Snapshot[]) : [];
  } catch {
    return [];
  }
}

function writeSnapshots(snapshots: Snapshot[]) {
  if (!isBrowser()) return;
  window.localStorage.setItem(SNAPSHOT_KEY, JSON.stringify(snapshots));
}

export function saveSnapshot(snapshot: Snapshot) {
  const snapshots = readSnapshots();
  snapshots.push(snapshot);
  const capped = snapshots.length > MAX_SNAPSHOTS ? snapshots.slice(snapshots.length - MAX_SNAPSHOTS) : snapshots;
  writeSnapshots(capped);
}

export function listSnapshots(): Snapshot[] {
  return readSnapshots().sort((a, b) => b.ts.localeCompare(a.ts));
}

export function updateSnapshotNote(id: string, note: string) {
  const snapshots = readSnapshots();
  const next = snapshots.map((snapshot) => snapshot.id === id ? { ...snapshot, note } : snapshot);
  writeSnapshots(next);
}

export function exportSnapshots(): Snapshot[] {
  return readSnapshots();
}
