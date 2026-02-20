import { DASHBOARD_SCHEMA_VERSION, migrateWorkspace, type Workspace } from "./model";
import { deserializeFilters, serializeFilters, type GlobalFilters } from "./filters";
import { validateWorkspace } from "./schema";

const DRAFT_KEY = "profinaut.dashboard.workspace.v1";
const COMMITTED_KEY = "profinaut.dashboard.workspace.committed.v1";
const PREFS_KEY = "profinaut.dashboard.prefs.v1";

export type StorageWarning = {
  key: string;
  message: string;
};

export type LoadedWorkspace = {
  workspace: Workspace | null;
  warnings: StorageWarning[];
};

function canUseStorage(): boolean {
  return typeof window !== "undefined" && !!window.localStorage;
}

function readWorkspace(key: string): LoadedWorkspace {
  if (!canUseStorage()) {
    return { workspace: null, warnings: [] };
  }

  try {
    const raw = window.localStorage.getItem(key);
    if (!raw) {
      return { workspace: null, warnings: [] };
    }
    const parsed = JSON.parse(raw);
    const migrated = migrateWorkspaceIfNeeded(parsed);
    const validated = validateWorkspace(migrated.workspace, { safeImportMode: true, strictWidgets: false });

    if (!validated.ok) {
      return {
        workspace: null,
        warnings: [{ key, message: `Validation failed for ${key}: ${validated.error}` }, ...migrated.warnings.map((message) => ({ key, message }))]
      };
    }

    return {
      workspace: validated.value,
      warnings: migrated.warnings.map((message) => ({ key, message }))
    };
  } catch {
    return {
      workspace: null,
      warnings: [{ key, message: `Corrupted JSON in ${key}; reset to defaults.` }]
    };
  }
}

export function migrateWorkspaceIfNeeded(input: unknown) {
  return migrateWorkspace(input);
}

export function saveDraft(workspaceDraft: Workspace): void {
  if (!canUseStorage()) return;
  window.localStorage.setItem(DRAFT_KEY, JSON.stringify(workspaceDraft));
}

export function loadDraft(): LoadedWorkspace {
  return readWorkspace(DRAFT_KEY);
}

export function commit(workspace: Workspace): void {
  if (!canUseStorage()) return;
  const committed: Workspace = {
    ...workspace,
    schema_version: DASHBOARD_SCHEMA_VERSION,
    updatedAt: new Date().toISOString()
  };
  window.localStorage.setItem(COMMITTED_KEY, JSON.stringify(committed));
  window.localStorage.removeItem(DRAFT_KEY);
}

export function loadCommitted(): LoadedWorkspace {
  return readWorkspace(COMMITTED_KEY);
}

export function clearDraft(): void {
  if (!canUseStorage()) return;
  window.localStorage.removeItem(DRAFT_KEY);
}

export function saveFilters(filters: GlobalFilters): void {
  if (!canUseStorage()) return;
  window.localStorage.setItem(PREFS_KEY, serializeFilters(filters));
}

export function loadFilters(): GlobalFilters {
  if (!canUseStorage()) return {};
  return deserializeFilters(window.localStorage.getItem(PREFS_KEY));
}
