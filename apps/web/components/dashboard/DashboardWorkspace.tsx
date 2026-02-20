"use client";

import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";

import { WidgetCatalog } from "./catalog/WidgetCatalog";
import { WorkspaceHistory } from "./core/history";
import { type GlobalFilters } from "./core/filters";
import { type Panel, type Workspace, DASHBOARD_SCHEMA_VERSION } from "./core/model";
import { buildDashboardUrl, parseDashboardUrl } from "./core/router";
import { validateWorkspace } from "./core/schema";
import { clearDraft, commit, loadCommitted, loadDraft, loadFilters, saveDraft, saveFilters } from "./core/storage";
import { GridLayout } from "./layout/GridLayout";
import { PanelFrame } from "./panels/PanelFrame";
import incidentTemplate from "./templates/incident.json";
import executionTemplate from "./templates/execution.json";
import marketDataTemplate from "./templates/marketdata.json";
import { normalizeStatus } from "./ui/badges";
import { WIDGETS } from "./widgets/registry";
import { fetchJson, useWidgetCapabilities, type WidgetQuality } from "./widgets/runtime";

const FilterContext = createContext<GlobalFilters>({});
export const useDashboardFilters = () => useContext(FilterContext);

const templates = [marketDataTemplate, executionTemplate, incidentTemplate] as Workspace[];
const templateById = Object.fromEntries(templates.map((template) => [template.id, template])) as Record<string, Workspace>;

type StatusSummary = { overall_status?: string; components?: { name?: string; status?: string }[] };

function isoNow() {
  return new Date().toISOString();
}

function cloneWorkspace(workspace: Workspace): Workspace {
  return structuredClone(workspace);
}

function findFirstSlot(panels: Panel[], w: number, h: number, columns: number) {
  const occupied = new Set<string>();
  for (const panel of panels) {
    const grid = panel.frame.grid;
    if (!grid) continue;
    for (let y = grid.y; y < grid.y + grid.h; y += 1) {
      for (let x = grid.x; x < grid.x + grid.w; x += 1) occupied.add(`${x}:${y}`);
    }
  }

  for (let y = 0; y < 40; y += 1) {
    for (let x = 0; x <= columns - w; x += 1) {
      let clear = true;
      for (let dy = 0; dy < h && clear; dy += 1) {
        for (let dx = 0; dx < w; dx += 1) {
          if (occupied.has(`${x + dx}:${y + dy}`)) {
            clear = false;
            break;
          }
        }
      }
      if (clear) return { x, y };
    }
  }
  return { x: 0, y: panels.length + 1 };
}

export default function DashboardWorkspace() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const historyRef = useRef(new WorkspaceHistory(100));

  const [warnings, setWarnings] = useState<string[]>([]);
  const [importError, setImportError] = useState<string | null>(null);
  const [committed, setCommitted] = useState<Workspace | null>(null);
  const [draft, setDraft] = useState<Workspace | null>(null);
  const [mode, setMode] = useState<"view" | "edit">("view");
  const [activePageId, setActivePageId] = useState<string | null>(null);
  const [focusMode, setFocusMode] = useState(false);
  const [filters, setFilters] = useState<GlobalFilters>({ timeRange: "1h" });
  const [statusSummary, setStatusSummary] = useState<StatusSummary | null>(null);
  const [qualityMap, setQualityMap] = useState<Record<string, WidgetQuality>>({});

  useEffect(() => {
    const committedLoaded = loadCommitted();
    const draftLoaded = loadDraft();
    const prefs = loadFilters();
    const routeState = parseDashboardUrl(new URLSearchParams(searchParams.toString()));

    setWarnings([...committedLoaded.warnings.map((w) => w.message), ...draftLoaded.warnings.map((w) => w.message)]);

    const baseWorkspace = committedLoaded.workspace ?? null;
    const initialWorkspace = draftLoaded.workspace ?? baseWorkspace;
    setCommitted(baseWorkspace);
    setDraft(initialWorkspace ? cloneWorkspace(initialWorkspace) : null);

    const nextFilters = { ...prefs, ...routeState.filters };
    setFilters(nextFilters);
    saveFilters(nextFilters);

    const workspaceId = searchParams.get("workspace");
    const selectedTemplate = workspaceId ? templateById[workspaceId] : null;

    if (selectedTemplate) {
      const normalized = { ...selectedTemplate, createdAt: isoNow(), updatedAt: isoNow(), schema_version: DASHBOARD_SCHEMA_VERSION };
      setCommitted(normalized);
      setDraft(cloneWorkspace(normalized));
      setActivePageId(normalized.defaultPageId);
      commit(normalized);
    } else {
      const pageFromRoute = routeState.pageId ?? baseWorkspace?.defaultPageId ?? null;
      setActivePageId(pageFromRoute);
    }

    setMode(searchParams.get("mode") === "edit" ? "edit" : "view");
    setFocusMode(Boolean(routeState.focus));
  }, [searchParams]);

  useEffect(() => {
    if (!focusMode) {
      document.body.classList.remove("dashboard-focus-mode");
      return;
    }
    document.body.classList.add("dashboard-focus-mode");
    return () => document.body.classList.remove("dashboard-focus-mode");
  }, [focusMode]);

  useEffect(() => {
    const refresh = async () => {
      const result = await fetchJson<StatusSummary>("/api/status/summary", 6000);
      if (result.ok && result.data) setStatusSummary(result.data);
    };
    refresh();
    const id = setInterval(refresh, 10000);
    return () => clearInterval(id);
  }, []);

  useEffect(() => {
    const params = new URLSearchParams(searchParams.toString());
    const baseUrl = buildDashboardUrl({ pageId: activePageId ?? undefined, focus: focusMode, filters });
    const nextParams = new URLSearchParams(baseUrl.split("?")[1] ?? "");

    for (const key of ["mode", "catalog", "workspace"]) {
      const value = params.get(key);
      if (value) nextParams.set(key, value);
    }

    router.replace(`${pathname}?${nextParams.toString()}`, { scroll: false });
    saveFilters(filters);
  }, [activePageId, filters, focusMode, pathname, router, searchParams]);

  const workspace = mode === "edit" ? draft : committed;
  const activePage = workspace?.pages.find((p) => p.id === activePageId) ?? workspace?.pages[0] ?? null;
  const canUndo = mode === "edit" && historyRef.current.canUndo();
  const canRedo = mode === "edit" && historyRef.current.canRedo();
  const capabilities = useWidgetCapabilities(statusSummary);

  const patchDraft = useCallback((mutator: (draftWorkspace: Workspace) => Workspace) => {
    if (!draft) return;
    historyRef.current.push(draft);
    const nextDraft = mutator(cloneWorkspace(draft));
    setDraft(nextDraft);
    saveDraft(nextDraft);
  }, [draft]);

  const handleApply = () => {
    if (!draft) return;
    const next = { ...draft, updatedAt: isoNow() };
    setCommitted(next);
    setDraft(cloneWorkspace(next));
    commit(next);
    setMode("view");
    historyRef.current.clear();
  };

  const handleCancel = () => {
    if (!committed) return;
    const rollback = cloneWorkspace(committed);
    setDraft(rollback);
    clearDraft();
    setMode("view");
    historyRef.current.clear();
  };

  const handleUndo = () => {
    if (!draft) return;
    const prev = historyRef.current.undo(draft);
    if (!prev) return;
    setDraft(prev);
    saveDraft(prev);
  };

  const handleRedo = () => {
    if (!draft) return;
    const next = historyRef.current.redo(draft);
    if (!next) return;
    setDraft(next);
    saveDraft(next);
  };

  const updatePanelGrid = (panelId: string, grid: { x: number; y: number; w: number; h: number }) => {
    if (!activePage) return;
    patchDraft((ws) => {
      const page = ws.pages.find((p) => p.id === activePage.id);
      const panel = page?.layout.panels.find((p) => p.id === panelId);
      if (!panel) return ws;
      panel.frame.grid = grid;
      panel.updatedAt = isoNow();
      ws.updatedAt = isoNow();
      return ws;
    });
  };

  const addWidgetPanel = (widgetId: string) => {
    const definition = WIDGETS[widgetId];
    if (!definition || !activePage) return;
    patchDraft((ws) => {
      const page = ws.pages.find((p) => p.id === activePage.id);
      if (!page) return ws;
      const columns = page.layout.gridSpec?.columns ?? 12;
      const slot = findFirstSlot(page.layout.panels, definition.defaultGrid.w, definition.defaultGrid.h, columns);
      page.layout.panels.push({
        id: `panel-${Date.now()}-${Math.random().toString(16).slice(2, 7)}`,
        widgetId,
        title: definition.title,
        dataSpec: { queryKey: widgetId },
        viewSpec: { variant: "kpi", showBadges: true },
        frame: { grid: { x: slot.x, y: slot.y, w: definition.defaultGrid.w, h: definition.defaultGrid.h } },
        createdAt: isoNow(),
        updatedAt: isoNow()
      });
      ws.updatedAt = isoNow();
      return ws;
    });
  };

  const removePanel = (panelId: string) => {
    if (!activePage) return;
    const panel = activePage.layout.panels.find((item) => item.id === panelId);
    if (panel?.protected && !window.confirm("This panel is protected. Remove it anyway?")) return;
    patchDraft((ws) => {
      const page = ws.pages.find((p) => p.id === activePage.id);
      if (!page) return ws;
      page.layout.panels = page.layout.panels.filter((p) => p.id !== panelId);
      ws.updatedAt = isoNow();
      return ws;
    });
  };

  const duplicatePanel = (panelId: string) => {
    if (!activePage) return;
    patchDraft((ws) => {
      const page = ws.pages.find((p) => p.id === activePage.id);
      const panel = page?.layout.panels.find((p) => p.id === panelId);
      if (!page || !panel) return ws;
      const grid = panel.frame.grid ?? { x: 0, y: 0, w: 3, h: 2 };
      const columns = page.layout.gridSpec?.columns ?? 12;
      const slot = findFirstSlot(page.layout.panels, grid.w, grid.h, columns);
      page.layout.panels.push({
        ...structuredClone(panel),
        id: `panel-${Date.now()}-${Math.random().toString(16).slice(2, 7)}`,
        frame: { ...panel.frame, grid: { ...grid, x: slot.x, y: slot.y } },
        createdAt: isoNow(),
        updatedAt: isoNow()
      });
      ws.updatedAt = isoNow();
      return ws;
    });
  };

  const toggleFlag = (panelId: string, flag: "locked" | "pinned" | "protected") => {
    if (!activePage) return;
    patchDraft((ws) => {
      const page = ws.pages.find((p) => p.id === activePage.id);
      const panel = page?.layout.panels.find((p) => p.id === panelId);
      if (!panel) return ws;
      panel[flag] = !panel[flag];
      panel.updatedAt = isoNow();
      ws.updatedAt = isoNow();
      return ws;
    });
  };

  const handleTemplatePick = (template: Workspace) => {
    const normalized = { ...template, createdAt: isoNow(), updatedAt: isoNow(), schema_version: DASHBOARD_SCHEMA_VERSION };
    setCommitted(normalized);
    setDraft(cloneWorkspace(normalized));
    setActivePageId(normalized.defaultPageId);
    commit(normalized);
  };

  const handleExport = () => {
    if (!workspace) return;
    const blob = new Blob([JSON.stringify(workspace, null, 2)], { type: "application/json" });
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.download = `dashboard-workspace-${workspace.id}.json`;
    link.click();
    URL.revokeObjectURL(link.href);
  };

  const handleImport = async (file: File | null) => {
    if (!file) return;
    setImportError(null);
    try {
      const raw = await file.text();
      const parsed = JSON.parse(raw);
      const validated = validateWorkspace(parsed, { safeImportMode: true, strictWidgets: false });
      if (!validated.ok) {
        setImportError(`${validated.error} at ${validated.path}`);
        return;
      }
      const imported = { ...validated.value, updatedAt: isoNow() };
      setDraft(imported);
      setCommitted(imported);
      setActivePageId(imported.defaultPageId);
      commit(imported);
      setWarnings((prev) => [...prev, ...validated.warnings]);
    } catch {
      setImportError("Invalid JSON import. Workspace unchanged.");
    }
  };

  if (!workspace) {
    return (
      <div className="card">
        <h2 className="section-title">Choose a dashboard template</h2>
        <p className="text-muted">Start with a personal workspace template.</p>
        <div className="card-grid" style={{ marginTop: 12 }}>
          {templates.map((template) => (
            <button key={template.id} className="btn" onClick={() => handleTemplatePick(template)}>{template.name}</button>
          ))}
        </div>
      </div>
    );
  }

  return (
    <FilterContext.Provider value={filters}>
      <style jsx global>{`
        body.dashboard-focus-mode .status-ribbon,
        body.dashboard-focus-mode .sidebar,
        body.dashboard-focus-mode .mobile-menu-btn,
        body.dashboard-focus-mode .mobile-backdrop { display: none !important; }
        body.dashboard-focus-mode .layout { grid-template-columns: 1fr !important; }
      `}</style>
      <div>
        <div className="page-header" style={{ alignItems: "center", gap: 12 }}>
          <div className="page-header-left">
            <h1 className="page-title">Dashboard Workspace</h1>
            <p className="page-subtitle">Grid widgets with catalog, drag/resize, and quality badges.</p>
          </div>
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
            <button className={`btn ${mode === "view" ? "btn-primary" : ""}`} onClick={() => setMode("view")}>View</button>
            <button className={`btn ${mode === "edit" ? "btn-primary" : ""}`} onClick={() => setMode("edit")}>Edit</button>
            <button className="btn" onClick={() => setFocusMode((v) => !v)}>{focusMode ? "Exit Focus" : "Focus/Kiosk"}</button>
            <button className="btn" onClick={handleExport}>Export</button>
            <label className="btn" style={{ cursor: "pointer" }}>Import
              <input type="file" accept="application/json" style={{ display: "none" }} onChange={(e) => handleImport(e.target.files?.[0] ?? null)} />
            </label>
          </div>
        </div>

        <div className="card" style={{ marginBottom: 12 }}>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit,minmax(140px,1fr))", gap: 8 }}>
            <input placeholder="venue" value={filters.venue ?? ""} onChange={(e) => setFilters((prev) => ({ ...prev, venue: e.target.value || undefined }))} />
            <input placeholder="bot" value={filters.bot ?? ""} onChange={(e) => setFilters((prev) => ({ ...prev, bot: e.target.value || undefined }))} />
            <input placeholder="symbol" value={filters.symbol ?? ""} onChange={(e) => setFilters((prev) => ({ ...prev, symbol: e.target.value || undefined }))} />
            <select value={filters.timeRange ?? "1h"} onChange={(e) => setFilters((prev) => ({ ...prev, timeRange: e.target.value as GlobalFilters["timeRange"] }))}>
              <option value="15m">15m</option><option value="1h">1h</option><option value="1d">1d</option><option value="7d">7d</option>
            </select>
          </div>
        </div>

        {(warnings.length > 0 || importError) && (
          <div className="notice notice-warning" style={{ marginBottom: 12 }}>
            <span className="notice-icon">!</span>
            <div className="notice-content">{[...warnings, importError].filter(Boolean).map((msg) => <div key={msg}>{msg}</div>)}</div>
          </div>
        )}

        <div className="card" style={{ marginBottom: 12 }}>
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center" }}>
            {workspace.pages.map((page) => (
              <button key={page.id} className={`btn ${activePage?.id === page.id ? "btn-primary" : ""}`} onClick={() => setActivePageId(page.id)}>{page.title}</button>
            ))}
            {mode === "edit" && (
              <>
                <button className="btn" onClick={handleUndo} disabled={!canUndo}>Undo</button>
                <button className="btn" onClick={handleRedo} disabled={!canRedo}>Redo</button>
                <button className="btn btn-success" onClick={handleApply}>Apply</button>
                <button className="btn btn-danger" onClick={handleCancel}>Cancel</button>
              </>
            )}
          </div>
        </div>

        {mode === "edit" && <WidgetCatalog onAdd={addWidgetPanel} />}

        {activePage && (
          <GridLayout
            panels={activePage.layout.panels}
            columns={activePage.layout.gridSpec?.columns ?? 12}
            rowHeight={activePage.layout.gridSpec?.rowHeight ?? 140}
            gap={activePage.layout.gridSpec?.gap ?? 12}
            editMode={mode === "edit"}
            onPanelFrameChange={updatePanelGrid}
            renderPanel={(panel, bind) => {
              const definition = WIDGETS[panel.widgetId];
              const title = panel.title ?? definition?.title ?? panel.widgetId;
              return (
                <PanelFrame
                  title={title}
                  quality={qualityMap[panel.id]}
                  locked={panel.locked}
                  pinned={panel.pinned}
                  protectedPanel={panel.protected}
                  canEdit={mode === "edit"}
                  onDuplicate={() => duplicatePanel(panel.id)}
                  onRemove={() => removePanel(panel.id)}
                  onToggleLock={() => toggleFlag(panel.id, "locked")}
                  onTogglePin={() => toggleFlag(panel.id, "pinned")}
                  onToggleProtect={() => toggleFlag(panel.id, "protected")}
                  dragHandleProps={bind.dragHandleProps}
                  resizeHandle={bind.resizeHandle}
                >
                  {definition ? (
                    definition.render({
                      globalFilters: filters,
                      capabilities,
                      refreshNow: () => setStatusSummary((prev) => prev ? { ...prev } : prev),
                      reportQuality: (quality) => setQualityMap((prev) => ({ ...prev, [panel.id]: { ...quality, status: normalizeStatus(quality.status) } }))
                    })
                  ) : (
                    <div className="text-sm text-muted">Unknown widget: {panel.widgetId}</div>
                  )}
                </PanelFrame>
              );
            }}
          />
        )}
      </div>
    </FilterContext.Provider>
  );
}
