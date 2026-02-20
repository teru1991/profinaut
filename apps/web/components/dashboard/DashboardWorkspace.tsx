"use client";

import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";

import marketDataTemplate from "./templates/marketdata.json";
import executionTemplate from "./templates/execution.json";
import incidentTemplate from "./templates/incident.json";

import { WorkspaceHistory } from "./core/history";
import { type GlobalFilters } from "./core/filters";
import { parseDashboardUrl } from "./core/router";
import { validateWorkspace } from "./core/schema";
import { clearDraft, commit, loadCommitted, loadDraft, loadFilters, saveDraft, saveFilters } from "./core/storage";
import { DASHBOARD_SCHEMA_VERSION, type Panel, type Workspace } from "./core/model";

type StatusSummary = { overall_status: string; components?: { name: string; status: string; reason?: string | null }[] };
type BotsSummary = { total: number; items: { state?: string; status?: string; degraded?: boolean }[] };

const FilterContext = createContext<GlobalFilters>({});
export const useDashboardFilters = () => useContext(FilterContext);

const templates = [marketDataTemplate, executionTemplate, incidentTemplate] as Workspace[];

function isoNow() {
  return new Date().toISOString();
}

function cloneWorkspace(workspace: Workspace): Workspace {
  return structuredClone(workspace);
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
  const [status, setStatus] = useState<StatusSummary | null>(null);
  const [bots, setBots] = useState<BotsSummary | null>(null);
  const [statusError, setStatusError] = useState<string | null>(null);
  const [lastUpdated, setLastUpdated] = useState<string | null>(null);
  const [draggingPanelId, setDraggingPanelId] = useState<string | null>(null);

  useEffect(() => {
    const committedLoaded = loadCommitted();
    const draftLoaded = loadDraft();
    const prefs = loadFilters();
    const routeState = parseDashboardUrl(new URLSearchParams(searchParams.toString()));

    setWarnings([
      ...committedLoaded.warnings.map((w) => w.message),
      ...draftLoaded.warnings.map((w) => w.message)
    ]);

    const baseWorkspace = committedLoaded.workspace ?? null;
    const initialWorkspace = draftLoaded.workspace ?? baseWorkspace;
    setCommitted(baseWorkspace);
    setDraft(initialWorkspace ? cloneWorkspace(initialWorkspace) : null);

    const nextFilters = { ...prefs, ...routeState.filters };
    setFilters(nextFilters);
    saveFilters(nextFilters);

    const pageFromRoute = routeState.pageId ?? baseWorkspace?.defaultPageId ?? null;
    setActivePageId(pageFromRoute);
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

  const refreshData = useCallback(async () => {
    try {
      const [statusRes, botsRes] = await Promise.allSettled([
        fetch("/api/status/summary", { cache: "no-store" }),
        fetch("/api/bots?page=1&page_size=50", { cache: "no-store" })
      ]);

      if (statusRes.status === "fulfilled" && statusRes.value.ok) {
        setStatus(await statusRes.value.json());
        setStatusError(null);
      }
      if (botsRes.status === "fulfilled" && botsRes.value.ok) {
        setBots(await botsRes.value.json());
      }
      if (
        (statusRes.status === "rejected" || (statusRes.status === "fulfilled" && !statusRes.value.ok)) &&
        (botsRes.status === "rejected" || (botsRes.status === "fulfilled" && !botsRes.value.ok))
      ) {
        setStatusError("Status and bots endpoints are unavailable.");
      }
      setLastUpdated(new Date().toISOString());
    } catch {
      setStatusError("Unable to refresh dashboard snapshot.");
      setLastUpdated(new Date().toISOString());
    }
  }, []);

  useEffect(() => {
    refreshData();
    const id = setInterval(refreshData, 10000);
    return () => clearInterval(id);
  }, [refreshData]);

  useEffect(() => {
    const params = new URLSearchParams(searchParams.toString());
    if (activePageId) params.set("pageId", activePageId);
    else params.delete("pageId");
    params.set("focus", focusMode ? "1" : "0");
    filters.venue ? params.set("venue", filters.venue) : params.delete("venue");
    filters.bot ? params.set("bot", filters.bot) : params.delete("bot");
    filters.symbol ? params.set("symbol", filters.symbol) : params.delete("symbol");
    filters.timeRange ? params.set("range", filters.timeRange) : params.delete("range");

    router.replace(`${pathname}?${params.toString()}`, { scroll: false });
    saveFilters(filters);
  }, [activePageId, filters, focusMode, pathname, router, searchParams]);

  const workspace = mode === "edit" ? draft : committed;
  const activePage = workspace?.pages.find((p) => p.id === activePageId) ?? workspace?.pages[0] ?? null;

  const canUndo = mode === "edit" && historyRef.current.canUndo();
  const canRedo = mode === "edit" && historyRef.current.canRedo();

  const patchDraft = (mutator: (draftWorkspace: Workspace) => Workspace) => {
    if (!draft) return;
    historyRef.current.push(draft);
    const nextDraft = mutator(cloneWorkspace(draft));
    setDraft(nextDraft);
    saveDraft(nextDraft);
  };

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

  const addPlaceholderPanel = () => {
    if (!activePage) return;
    patchDraft((ws) => {
      const page = ws.pages.find((p) => p.id === activePage.id);
      if (!page) return ws;
      page.layout.panels.push({
        id: `panel-${Date.now()}`,
        widgetId: "placeholder",
        title: "New Panel",
        dataSpec: { queryKey: "placeholder" },
        viewSpec: { variant: "placeholder" },
        frame: { grid: { x: 0, y: page.layout.panels.length + 1, w: 3, h: 1 } },
        createdAt: isoNow(),
        updatedAt: isoNow()
      });
      ws.updatedAt = isoNow();
      return ws;
    });
  };

  const removePanel = (panelId: string) => {
    if (!activePage) return;
    patchDraft((ws) => {
      const page = ws.pages.find((p) => p.id === activePage.id);
      if (!page) return ws;
      page.layout.panels = page.layout.panels.filter((panel) => panel.id !== panelId);
      ws.updatedAt = isoNow();
      return ws;
    });
  };

  const reorderPanels = (sourceId: string, targetId: string) => {
    if (!activePage || sourceId === targetId) return;
    patchDraft((ws) => {
      const page = ws.pages.find((p) => p.id === activePage.id);
      if (!page) return ws;
      const next = [...page.layout.panels];
      const sourceIdx = next.findIndex((panel) => panel.id === sourceId);
      const targetIdx = next.findIndex((panel) => panel.id === targetId);
      if (sourceIdx < 0 || targetIdx < 0) return ws;
      const [item] = next.splice(sourceIdx, 1);
      next.splice(targetIdx, 0, item);
      page.layout.panels = next;
      ws.updatedAt = isoNow();
      return ws;
    });
  };

  const resizePanel = (panelId: string, delta: number) => {
    if (!activePage) return;
    patchDraft((ws) => {
      const page = ws.pages.find((p) => p.id === activePage.id);
      const panel = page?.layout.panels.find((p) => p.id === panelId);
      if (!panel) return ws;
      if (!panel.frame.grid) {
        panel.frame.grid = { x: 0, y: 0, w: 3, h: 1 };
      }
      panel.frame.grid.w = Math.max(2, Math.min(12, panel.frame.grid.w + delta));
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

  const snapshot = useMemo(() => {
    const totalBots = bots?.total ?? 0;
    const activeBots = (bots?.items ?? []).filter((b) => {
      const state = (b.state ?? b.status ?? "").toUpperCase();
      return state === "RUNNING" || state === "ACTIVE" || state === "OK";
    }).length;
    const degradedBots = (bots?.items ?? []).filter((b) => b.degraded).length;
    const degradedComponents = (status?.components ?? []).filter((c) => c.status.toUpperCase() !== "OK");

    return {
      overall: status?.overall_status ?? "UNKNOWN",
      totalBots,
      activeBots,
      degradedBots,
      degradedComponents
    };
  }, [bots, status]);

  if (!workspace) {
    return (
      <div className="card">
        <h2 className="section-title">Choose a dashboard template</h2>
        <p className="text-muted">Start with a personal workspace template.</p>
        <div className="card-grid" style={{ marginTop: 12 }}>
          {templates.map((template) => (
            <button key={template.id} className="btn" onClick={() => handleTemplatePick(template)}>
              {template.name}
            </button>
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
            <p className="page-subtitle">Customizable layout with safe Edit/Draft workflow.</p>
          </div>
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
            <button className={`btn ${mode === "view" ? "btn-primary" : ""}`} onClick={() => setMode("view")}>View</button>
            <button className={`btn ${mode === "edit" ? "btn-primary" : ""}`} onClick={() => setMode("edit")}>Edit</button>
            <button className="btn" onClick={() => setFocusMode((v) => !v)}>{focusMode ? "Exit Focus" : "Focus/Kiosk"}</button>
            <button className="btn" onClick={handleExport}>Export</button>
            <label className="btn" style={{ cursor: "pointer" }}>
              Import
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

        {(warnings.length > 0 || statusError || importError) && (
          <div className="notice notice-warning" style={{ marginBottom: 12 }}>
            <span className="notice-icon">!</span>
            <div className="notice-content">
              {[...warnings, statusError, importError].filter(Boolean).map((message) => (
                <div key={message}>{message}</div>
              ))}
              {lastUpdated && <div className="text-xs">Last snapshot: {new Date(lastUpdated).toLocaleTimeString()}</div>}
            </div>
          </div>
        )}

        <div className="card" style={{ marginBottom: 12 }}>
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center" }}>
            {workspace.pages.map((page) => (
              <button key={page.id} className={`btn ${activePage?.id === page.id ? "btn-primary" : ""}`} onClick={() => setActivePageId(page.id)}>{page.title}</button>
            ))}
            {mode === "edit" && (
              <>
                <button className="btn" onClick={addPlaceholderPanel}>Add Panel</button>
                <button className="btn" onClick={handleUndo} disabled={!canUndo}>Undo</button>
                <button className="btn" onClick={handleRedo} disabled={!canRedo}>Redo</button>
                <button className="btn btn-success" onClick={handleApply}>Apply</button>
                <button className="btn btn-danger" onClick={handleCancel}>Cancel</button>
              </>
            )}
          </div>
        </div>

        <div className="card-grid" style={{ gridTemplateColumns: "repeat(auto-fill,minmax(320px,1fr))" }}>
          {(activePage?.layout.panels ?? []).map((panel) => (
            <div
              key={panel.id}
              className="card"
              draggable={mode === "edit"}
              onDragStart={() => setDraggingPanelId(panel.id)}
              onDragOver={(event) => mode === "edit" && event.preventDefault()}
              onDrop={() => {
                if (mode === "edit" && draggingPanelId) reorderPanels(draggingPanelId, panel.id);
                setDraggingPanelId(null);
              }}
              style={{ opacity: draggingPanelId === panel.id ? 0.6 : 1 }}
            >
              <div className="card-header">
                <h3 className="card-title">{panel.title ?? panel.widgetId}</h3>
                {mode === "edit" && (
                  <div style={{ display: "flex", gap: 6 }}>
                    <button className="btn btn-sm" onClick={() => resizePanel(panel.id, -1)}>-W</button>
                    <button className="btn btn-sm" onClick={() => resizePanel(panel.id, 1)}>+W</button>
                    <button className="btn btn-sm btn-danger" onClick={() => removePanel(panel.id)}>Remove</button>
                  </div>
                )}
              </div>
              <PanelBody panel={panel} snapshot={snapshot} />
            </div>
          ))}
        </div>
      </div>
    </FilterContext.Provider>
  );
}

function PanelBody({ panel, snapshot }: { panel: Panel; snapshot: { overall: string; totalBots: number; activeBots: number; degradedBots: number; degradedComponents: StatusSummary["components"] } }) {
  if (panel.widgetId === "system-status") {
    return <div><div className={`badge ${snapshot.overall === "OK" ? "badge-success" : "badge-warning"}`}>{snapshot.overall}</div></div>;
  }
  if (panel.widgetId === "bots-overview") {
    return <div className="text-sm">Total {snapshot.totalBots} / Active {snapshot.activeBots} / Degraded {snapshot.degradedBots}</div>;
  }
  if (panel.widgetId === "degraded-components") {
    return (
      <div className="text-sm">
        {(snapshot.degradedComponents ?? []).length === 0 ? "No degraded components" : (snapshot.degradedComponents ?? []).map((component) => (
          <div key={component.name}>{component.name}: {component.status}{component.reason ? ` — ${component.reason}` : ""}</div>
        ))}
      </div>
    );
  }
  if (panel.widgetId === "quick-nav") {
    return (
      <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
        {[
          ["Bots", "/bots"],
          ["Portfolio", "/portfolio"],
          ["Markets", "/markets"],
          ["Commands", "/commands"],
          ["Analytics", "/analytics"]
        ].map(([label, href]) => (
          <a key={href} href={href} className="btn btn-sm">{label}</a>
        ))}
      </div>
    );
  }
  return <div className="text-muted text-sm">{panel.widgetId} widget placeholder</div>;
}
