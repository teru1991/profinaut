"use client";

import type { ReactNode } from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";

const navItems = [
  {
    href: "/dashboard",
    label: "Dashboard",
    icon: (
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <rect x="3" y="3" width="7" height="7" rx="1" />
        <rect x="14" y="3" width="7" height="7" rx="1" />
        <rect x="3" y="14" width="7" height="7" rx="1" />
        <rect x="14" y="14" width="7" height="7" rx="1" />
      </svg>
    )
  },
  {
    href: "/incidents",
    label: "Incidents",
    icon: (
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
        <line x1="12" y1="9" x2="12" y2="13" />
        <line x1="12" y1="17" x2="12.01" y2="17" />
      </svg>
    )
  },
  {
    href: "/snapshots",
    label: "Snapshots",
    icon: (
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M21 19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h3l2-2h4l2 2h3a2 2 0 0 1 2 2z" />
        <circle cx="12" cy="13" r="4" />
      </svg>
    )
  },
  {
    href: "/audit",
    label: "Audit",
    icon: (
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M9 11l3 3L22 4" />
        <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
      </svg>
    )
  },
  {
    href: "/bots",
    label: "Bots",
    icon: (
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <rect x="3" y="11" width="18" height="10" rx="2" />
        <circle cx="12" cy="5" r="2" />
        <path d="M12 7v4" />
        <line x1="8" y1="16" x2="8" y2="16" />
        <line x1="16" y1="16" x2="16" y2="16" />
      </svg>
    )
  },
  {
    href: "/portfolio",
    label: "Portfolio",
    icon: (
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M21 12V7H5a2 2 0 0 1 0-4h14v4" />
        <path d="M3 5v14a2 2 0 0 0 2 2h16v-5" />
        <path d="M18 12a2 2 0 0 0 0 4h4v-4Z" />
      </svg>
    )
  },
  {
    href: "/markets",
    label: "Markets",
    icon: (
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <polyline points="22 7 13.5 15.5 8.5 10.5 2 17" />
        <polyline points="16 7 22 7 22 13" />
      </svg>
    )
  },
  {
    href: "/commands",
    label: "Commands",
    icon: (
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <polyline points="4 17 10 11 4 5" />
        <line x1="12" y1="19" x2="20" y2="19" />
      </svg>
    )
  },
  {
    href: "/analytics",
    label: "Analytics",
    icon: (
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <line x1="18" y1="20" x2="18" y2="10" />
        <line x1="12" y1="20" x2="12" y2="4" />
        <line x1="6" y1="20" x2="6" y2="14" />
      </svg>
    )
  },
  {
    href: "/datasets",
    label: "Datasets",
    icon: (
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <ellipse cx="12" cy="5" rx="9" ry="3" />
        <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" />
        <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" />
      </svg>
    )
  },
  {
    href: "/admin/modules",
    label: "Admin",
    icon: (
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <circle cx="12" cy="12" r="3" />
        <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
      </svg>
    )
  }
];

function SunIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="5" />
      <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
    </svg>
  );
}

type StatusSummary = { overall_status?: string; components?: { status?: string }[] };

let navStatusCache: { ts: number; data: StatusSummary | null } = { ts: 0, data: null };

const WORKSPACES = ["marketdata", "execution", "incident"] as const;

function iconFor(label: string) {
  const icons: Record<string, ReactNode> = {
    Dashboard: <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="3" y="3" width="7" height="7" rx="1" /><rect x="14" y="3" width="7" height="7" rx="1" /><rect x="3" y="14" width="7" height="7" rx="1" /><rect x="14" y="14" width="7" height="7" rx="1" /></svg>,
    Incidents: <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z" /><line x1="12" y1="9" x2="12" y2="13" /><line x1="12" y1="17" x2="12.01" y2="17" /></svg>,
    Bots: <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="3" y="11" width="18" height="10" rx="2" /><circle cx="12" cy="5" r="2" /><path d="M12 7v4" /></svg>,
    Portfolio: <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 12V7H5a2 2 0 0 1 0-4h14v4" /><path d="M3 5v14a2 2 0 0 0 2 2h16v-5" /></svg>,
    Markets: <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="22 7 13.5 15.5 8.5 10.5 2 17" /><polyline points="16 7 22 7 22 13" /></svg>,
    Commands: <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="4 17 10 11 4 5" /><line x1="12" y1="19" x2="20" y2="19" /></svg>,
    Analytics: <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="18" y1="20" x2="18" y2="10" /><line x1="12" y1="20" x2="12" y2="4" /><line x1="6" y1="20" x2="6" y2="14" /></svg>,
    Datasets: <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><ellipse cx="12" cy="5" rx="9" ry="3" /><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" /></svg>,
    Admin: <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="3" /><path d="M12 1v2M12 21v2M1 12h2M21 12h2" /></svg>
  };
  return icons[label] ?? null;
}

const statusBadgeClass = (status: string) => {
  if (status === "OK") return "badge badge-success";
  if (status === "UNAVAILABLE") return "badge badge-error";
  return "badge badge-warning";
};

function SunIcon() { return <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="5" /></svg>; }
function MoonIcon() { return <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" /></svg>; }
function MenuIcon() { return <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="3" y1="6" x2="21" y2="6" /><line x1="3" y1="12" x2="21" y2="12" /><line x1="3" y1="18" x2="21" y2="18" /></svg>; }
function ChevronLeftIcon() { return <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="15 18 9 12 15 6" /></svg>; }
function ChevronRightIcon() { return <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="9 18 15 12 9 6" /></svg>; }

export function NavShell({ children }: { children: ReactNode }) {
  const pathname = usePathname();
  const router = useRouter();
  const { theme, toggle } = useTheme();
  const [collapsed, setCollapsed] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);
  const [pinned, setPinned] = useState<string[]>([]);
  const [homeHref, setHome] = useState<string>("/dashboard");
  const [workspace, setWorkspace] = useState("marketdata");
  const [status, setStatus] = useState<string>("UNAVAILABLE");
  const [degradedCount, setDegradedCount] = useState(0);

  const readCurrentSearch = () => (typeof window === "undefined" ? new URLSearchParams() : new URLSearchParams(window.location.search));

  const closeMobile = useCallback(() => setMobileOpen(false), []);

  useEffect(() => {
    setPinned(loadPinnedHrefs());
    setHome(loadHomeHref() ?? "/dashboard");
    setWorkspace(loadWorkspaceTemplate() ?? "marketdata");
  }, []);

  useEffect(() => {
    const readStatus = async () => {
      const now = Date.now();
      if (now - navStatusCache.ts < 8000 && navStatusCache.data) {
        const summary = navStatusCache.data;
        setStatus(summary.overall_status?.toUpperCase() ?? "UNAVAILABLE");
        setDegradedCount((summary.components ?? []).filter((component) => (component.status ?? "").toUpperCase() !== "OK").length);
        return;
      }
      const result = await fetchJson<StatusSummary>("/api/status/summary", 5000);
      if (!result.ok || !result.data) {
        setStatus("UNAVAILABLE");
        return;
      }
      navStatusCache = { ts: now, data: result.data };
      setStatus(result.data.overall_status?.toUpperCase() ?? "UNAVAILABLE");
      setDegradedCount((result.data.components ?? []).filter((component) => (component.status ?? "").toUpperCase() !== "OK").length);
    };

    readStatus();
  }, [pathname]);

  useEffect(() => {
    setMobileOpen(false);
  }, [pathname]);

  const nav = useMemo(() => buildNav({ overallStatus: status, degradedCount, pinned, homeHref }), [degradedCount, homeHref, pinned, status]);

  const dashboardLink = useMemo(() => {
    if (pathname !== "/dashboard") return "/dashboard";
    return buildDashboardUrl(parseDashboardUrl(readCurrentSearch()));
  }, [pathname]);

  const resolveHref = (href: string) => {
    if (!href.startsWith("/dashboard")) return href;
    const route = parseDashboardUrl(readCurrentSearch());
    const current = buildDashboardUrl(route);
    if (href.includes("workspace=")) {
      return `${current}${current.includes("?") ? "&" : "?"}${href.split("?")[1]}`;
    }
    return pathname === "/dashboard" ? current : href;
  };

  return (
    <>
      <CommandPalette />
      <button className="mobile-menu-btn btn btn-ghost btn-icon" onClick={() => setMobileOpen(true)} aria-label="Open navigation menu" style={{ position: "fixed", top: 4, left: 8, zIndex: 41 }}>
        <MenuIcon />
      </button>
      {mobileOpen && <div className="mobile-backdrop" onClick={closeMobile} />}
      <div className={`layout${collapsed ? " sidebar-collapsed" : ""}`}>
        <aside className={`sidebar${mobileOpen ? " mobile-open" : ""}`}>
          <div className="sidebar-header" style={{ flexDirection: "column", alignItems: "stretch", gap: 8 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <div style={{ flex: 1, minWidth: 0 }}><div className="sidebar-logo">Profinaut</div><div className="sidebar-logo-sub">Control Plane</div></div>
              <button className="sidebar-toggle-btn" onClick={() => { setCollapsed((c) => !c); closeMobile(); }} aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}>{collapsed ? <ChevronRightIcon /> : <ChevronLeftIcon />}</button>
            </div>
            {!collapsed && <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", gap: 8 }}><span className={statusBadgeClass(status)}>Status: {status}</span><button className="btn btn-ghost" onClick={() => router.push(homeHref)}>Home</button></div>}
            {!collapsed && (
              <select
                value={workspace}
                onChange={(event) => {
                  const template = event.target.value;
                  setWorkspace(template);
                  saveWorkspaceTemplate(template);
                  const current = pathname === "/dashboard" ? buildDashboardUrl(parseDashboardUrl(readCurrentSearch())) : "/dashboard";
                  router.push(`${current}${current.includes("?") ? "&" : "?"}workspace=${template}`);
                }}
              >
                {WORKSPACES.map((template) => <option key={template} value={template}>Workspace: {template}</option>)}
              </select>
            )}
          </div>

          <nav className="nav">
            {nav.items.map((item) => {
              const href = item.href === "/dashboard" ? dashboardLink : resolveHref(item.href);
              const isActive = pathname === item.href.split("?")[0] || pathname.startsWith(`${item.href.split("?")[0]}/`);
              const isPinned = pinned.includes(item.href);
              const isHome = homeHref === item.href;
              return (
                <div key={item.href} style={{ display: "flex", alignItems: "center", gap: 4 }}>
                  <Link href={href} className={`nav-link${isActive ? " active" : ""}`} title={collapsed ? item.label : undefined}>
                    <span className="nav-link-icon">{iconFor(item.label)}</span>
                    <span className="nav-link-label">{item.label}</span>
                  </Link>
                  {!collapsed && (
                    <>
                      <button className="btn btn-ghost btn-icon" aria-label={`Pin ${item.label}`} onClick={() => setPinned(togglePinnedHref(item.href))}>{isPinned ? "★" : "☆"}</button>
                      <button className="btn btn-ghost btn-icon" aria-label={`Set ${item.label} as home`} onClick={() => { saveHomeHref(item.href); setHome(item.href); }}>{isHome ? "⌂" : "⌁"}</button>
                    </>
                  )}
                </div>
              );
            })}
            {!collapsed && pathname === "/dashboard" && (
              <button
                className="btn btn-ghost"
                onClick={async () => {
                  const href = `${window.location.origin}${buildDashboardUrl(parseDashboardUrl(readCurrentSearch()))}`;
                  await navigator.clipboard.writeText(href);
                }}
              >
                Copy link
              </button>
            )}
          </nav>

          <div className="sidebar-footer">
            <button className="theme-toggle" onClick={toggle} aria-label={`Switch to ${theme === "dark" ? "light" : "dark"} theme`}>
              {theme === "dark" ? <SunIcon /> : <MoonIcon />}<span className="theme-toggle-label">{theme === "dark" ? "Light mode" : "Dark mode"}</span>
            </button>
          </div>
        </aside>
        <main className="main">{children}</main>
      </div>
    </>
  );
}
