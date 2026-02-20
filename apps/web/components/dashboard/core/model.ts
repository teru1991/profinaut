export const DASHBOARD_SCHEMA_VERSION = 1;

export type LayoutKind = "grid" | "freeform" | "split";

export type DataSpec = {
  queryKey: string;
  endpointRef?: string;
  params?: Record<string, string | number | boolean | null | undefined>;
  refreshMs?: number;
  capabilityGate?: string;
};

export type ViewSpec = {
  variant?: "kpi" | "table" | "list" | "chart" | "status" | "placeholder";
  thresholds?: {
    warning?: number;
    error?: number;
  };
  compactMode?: boolean;
  showBadges?: boolean;
};

export type PanelFrame = {
  grid?: { x: number; y: number; w: number; h: number };
  freeform?: { left: number; top: number; width: number; height: number; zIndex: number };
};

export type Panel = {
  id: string;
  widgetId: string;
  title?: string;
  dataSpec: DataSpec;
  viewSpec: ViewSpec;
  frame: PanelFrame;
  locked?: boolean;
  pinned?: boolean;
  protected?: boolean;
  createdAt: string;
  updatedAt: string;
};

export type Layout = {
  kind: LayoutKind;
  panels: Panel[];
  gridSpec?: {
    columns: number;
    rowHeight: number;
    gap: number;
  };
  freeformSpec?: {
    width: number;
    height: number;
  };
  splitSpec?: {
    direction: "horizontal" | "vertical";
    ratio: number;
  };
};

export type Page = {
  id: string;
  title: string;
  layout: Layout;
};

export type Workspace = {
  id: string;
  name: string;
  pages: Page[];
  defaultPageId: string;
  createdAt: string;
  updatedAt: string;
  schema_version: number;
};

export type WorkspaceMigrationResult = {
  workspace: Workspace;
  migrated: boolean;
  warnings: string[];
};

export function migrateWorkspace(input: unknown): WorkspaceMigrationResult {
  if (!input || typeof input !== "object") {
    throw new Error("Workspace migration failed: invalid payload");
  }

  const candidate = input as Partial<Workspace> & { schema_version?: number };
  const version = typeof candidate.schema_version === "number" ? candidate.schema_version : 0;

  if (version > DASHBOARD_SCHEMA_VERSION) {
    throw new Error(`Workspace schema ${version} is newer than supported ${DASHBOARD_SCHEMA_VERSION}`);
  }

  if (version === DASHBOARD_SCHEMA_VERSION) {
    return {
      workspace: candidate as Workspace,
      migrated: false,
      warnings: []
    };
  }

  return {
    workspace: {
      ...(candidate as Workspace),
      schema_version: DASHBOARD_SCHEMA_VERSION,
      updatedAt: new Date().toISOString()
    },
    migrated: true,
    warnings: [`Migrated workspace schema ${version} -> ${DASHBOARD_SCHEMA_VERSION}`]
  };
}
