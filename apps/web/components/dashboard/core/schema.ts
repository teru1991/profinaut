import { DASHBOARD_SCHEMA_VERSION, type LayoutKind, type Page, type Panel, type Workspace } from "./model";

type ValidationSuccess = { ok: true; value: Workspace; warnings: string[] };
type ValidationFailure = { ok: false; error: string; path: string };

export type ValidateWorkspaceOptions = {
  strictWidgets?: boolean;
  safeImportMode?: boolean;
  allowedWidgetIds?: string[];
};

const DEFAULT_WIDGETS = new Set([
  "system-status",
  "bots-overview",
  "degraded-components",
  "quick-nav",
  "markets-overview",
  "commands-summary",
  "incident-feed",
  "placeholder"
]);

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function asString(value: unknown, path: string): string {
  if (typeof value !== "string" || !value.trim()) {
    throw new Error(`${path}: expected non-empty string`);
  }
  return value;
}

function normalizePanel(
  panel: unknown,
  path: string,
  options: Required<ValidateWorkspaceOptions>,
  warnings: string[]
): Panel {
  if (!isObject(panel)) {
    throw new Error(`${path}: expected object`);
  }

  const widgetId = asString(panel.widgetId, `${path}.widgetId`);
  const isKnownWidget = options.allowedWidgetIds.includes(widgetId);

  if (!isKnownWidget && options.strictWidgets && !options.safeImportMode) {
    throw new Error(`${path}.widgetId: unknown widget '${widgetId}'`);
  }

  const now = new Date().toISOString();

  if (!isKnownWidget && options.safeImportMode) {
    warnings.push(`Unknown widget '${widgetId}' converted to placeholder`);
  }

  return {
    id: asString(panel.id, `${path}.id`),
    widgetId: isKnownWidget ? widgetId : "placeholder",
    title: typeof panel.title === "string" ? panel.title : undefined,
    dataSpec: isObject(panel.dataSpec) ? (panel.dataSpec as Panel["dataSpec"]) : { queryKey: "placeholder" },
    viewSpec: isObject(panel.viewSpec) ? (panel.viewSpec as Panel["viewSpec"]) : { variant: "placeholder" },
    frame: isObject(panel.frame) ? (panel.frame as Panel["frame"]) : { grid: { x: 0, y: 0, w: 3, h: 1 } },
    locked: Boolean(panel.locked),
    pinned: Boolean(panel.pinned),
    protected: Boolean(panel.protected),
    createdAt: typeof panel.createdAt === "string" ? panel.createdAt : now,
    updatedAt: typeof panel.updatedAt === "string" ? panel.updatedAt : now
  };
}

export function validateWorkspace(
  input: unknown,
  options: ValidateWorkspaceOptions = {}
): ValidationSuccess | ValidationFailure {
  try {
    const resolvedOptions: Required<ValidateWorkspaceOptions> = {
      strictWidgets: options.strictWidgets ?? true,
      safeImportMode: options.safeImportMode ?? false,
      allowedWidgetIds: options.allowedWidgetIds ?? [...DEFAULT_WIDGETS]
    };

    if (!isObject(input)) {
      return { ok: false, error: "Workspace must be an object", path: "$" };
    }

    const warnings: string[] = [];

    const schemaVersion = typeof input.schema_version === "number" ? input.schema_version : undefined;
    if (!schemaVersion) {
      throw new Error("$.schema_version: required number");
    }
    if (schemaVersion > DASHBOARD_SCHEMA_VERSION) {
      throw new Error(`$.schema_version: unsupported future schema ${schemaVersion}`);
    }

    if (!Array.isArray(input.pages) || input.pages.length === 0) {
      throw new Error("$.pages: expected non-empty array");
    }

    const pages: Page[] = input.pages.map((page, pageIndex) => {
      const pagePath = `$.pages[${pageIndex}]`;
      if (!isObject(page)) {
        throw new Error(`${pagePath}: expected object`);
      }
      const rawPanels = isObject(page.layout) && Array.isArray(page.layout.panels) ? page.layout.panels : [];
      const panels = rawPanels.map((panel, panelIndex) =>
        normalizePanel(panel, `${pagePath}.layout.panels[${panelIndex}]`, resolvedOptions, warnings)
      );

      const layoutKind: LayoutKind =
        isObject(page.layout) &&
        (page.layout.kind === "grid" || page.layout.kind === "freeform" || page.layout.kind === "split")
          ? page.layout.kind
          : "grid";

      return {
        id: asString(page.id, `${pagePath}.id`),
        title: asString(page.title, `${pagePath}.title`),
        layout: {
          kind: layoutKind,
          panels,
          gridSpec: isObject(page.layout) && isObject(page.layout.gridSpec)
            ? (page.layout.gridSpec as Workspace["pages"][number]["layout"]["gridSpec"])
            : undefined,
          freeformSpec: isObject(page.layout) && isObject(page.layout.freeformSpec)
            ? (page.layout.freeformSpec as Workspace["pages"][number]["layout"]["freeformSpec"])
            : undefined,
          splitSpec: isObject(page.layout) && isObject(page.layout.splitSpec)
            ? (page.layout.splitSpec as Workspace["pages"][number]["layout"]["splitSpec"])
            : undefined
        }
      };
    });

    const workspace: Workspace = {
      id: asString(input.id, "$.id"),
      name: asString(input.name, "$.name"),
      pages,
      defaultPageId: asString(input.defaultPageId, "$.defaultPageId"),
      createdAt: typeof input.createdAt === "string" ? input.createdAt : new Date().toISOString(),
      updatedAt: typeof input.updatedAt === "string" ? input.updatedAt : new Date().toISOString(),
      schema_version: schemaVersion
    };

    return { ok: true, value: workspace, warnings };
  } catch (error) {
    const message = error instanceof Error ? error.message : "validation failed";
    const pathMatch = message.match(/^([^:]+):/);
    return {
      ok: false,
      error: message,
      path: pathMatch ? pathMatch[1] : "$"
    };
  }
}
