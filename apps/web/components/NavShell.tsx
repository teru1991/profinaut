"use client";

import type { ReactNode } from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";

// --- Types ---

type StatusSummary = {
  overall_status?: string;
  components?: { status?: string }[];
};

type NavItem = {
  href: string;
  label: string;
};

// --- Utilities (本来は外部ファイルからインポートすべきもの) ---

// ダミーのテーマ管理（実際のライブラリに合わせて調整してください）
function useTheme() {
  const [theme, setTheme] = useState<"dark" | "light">("dark");
  const toggle = () => setTheme((t) => (t === "dark" ? "light" : "dark"));
  return { theme, toggle };
}

// 共通Fetch関数
async function fetchJson<T>(url: string, timeoutMs: number): Promise<{ ok: boolean; data: T | null }> {
  try {
    const controller = new AbortController();
    const id = setTimeout(() => controller.abort(), timeoutMs);
    const res = await fetch(url, { signal: controller.signal });
    clearTimeout(id);
    if (!res.ok) return { ok: false, data: null };
    return { ok: true, data: await res.json() };
  } catch {
    return { ok: false, data: null };
  }
}

// ダミーのURLパーサー（要件に合わせて実装）
const parseDashboardUrl = (search: URLSearchParams) => ({ workspace: search.get("workspace") });
const buildDashboardUrl = (route: { workspace: string | null }) =>
    route.workspace ? `/dashboard?workspace=${route.workspace}` : "/dashboard";

// ローカルストレージ操作の安全なラッパー
const storage = {
  get: (key: string) => (typeof window !== "undefined" ? localStorage.getItem(key) : null),
  set: (key: string, val: string) => typeof window !== "undefined" && localStorage.setItem(key, val),
};

// --- Components ---

const SunIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="5" /><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" /></svg>
);
const MoonIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" /></svg>
);
const MenuIcon = () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="3" y1="6" x2="21" y2="6" /><line x1="3" y1="12" x2="21" y2="12" /><line x1="3" y1="18" x2="21" y2="18" /></svg>
);
const ChevronLeftIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="15 18 9 12 15 6" /></svg>
);
const ChevronRightIcon = () => (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="9 18 15 12 9 6" /></svg>
);

function IconFor({ label }: { label: string }) {
  const icons: Record<string, ReactNode> = {
    Dashboard: <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="3" y="3" width="7" height="7" rx="1" /><rect x="14" y="3" width="7" height="7" rx="1" /><rect x="3" y="14" width="7" height="7" rx="1" /><rect x="14" y="14" width="7" height="7" rx="1" /></svg>,
    Incidents: <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z" /><line x1="12" y1="9" x2="12" y2="13" /><line x1="12" y1="17" x2="12.01" y2="17" /></svg>,
    Bots: <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="3" y="11" width="18" height="10" rx="2" /><circle cx="12" cy="5" r="2" /><path d="M12 7v4" /></svg>,
  };
  return <span className="nav-link-icon">{icons[label] || <SunIcon />}</span>;
}

// --- Main Component ---

let navStatusCache: { ts: number; data: StatusSummary | null } = { ts: 0, data: null };
const WORKSPACES = ["marketdata", "execution", "incident"] as const;

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

  const currentSearch = useMemo(() =>
          typeof window !== "undefined" ? new URLSearchParams(window.location.search) : new URLSearchParams()
      , [pathname]);

  useEffect(() => {
    setPinned(JSON.parse(storage.get("pinned_hrefs") || "[]"));
    setHome(storage.get("home_href") || "/dashboard");
    setWorkspace(storage.get("workspace_template") || "marketdata");
  }, []);

  useEffect(() => {
    const readStatus = async () => {
      const now = Date.now();
      if (now - navStatusCache.ts < 8000 && navStatusCache.data) {
        applyStatus(navStatusCache.data);
        return;
      }
      const result = await fetchJson<StatusSummary>("/api/status/summary", 5000);
      if (result.ok && result.data) {
        navStatusCache = { ts: now, data: result.data };
        applyStatus(result.data);
      } else {
        setStatus("UNAVAILABLE");
      }
    };
    const applyStatus = (summary: StatusSummary) => {
      setStatus(summary.overall_status?.toUpperCase() ?? "UNAVAILABLE");
      setDegradedCount((summary.components ?? []).filter((c) => (c.status ?? "").toUpperCase() !== "OK").length);
    };
    readStatus();
  }, [pathname]);

  const navItems: NavItem[] = [
    { href: "/dashboard", label: "Dashboard" },
    { href: "/incidents", label: "Incidents" },
    { href: "/bots", label: "Bots" },
  ];

  const dashboardLink = useMemo(() => {
    if (pathname !== "/dashboard") return "/dashboard";
    return buildDashboardUrl(parseDashboardUrl(currentSearch));
  }, [pathname, currentSearch]);

  const togglePinned = (href: string) => {
    const next = pinned.includes(href) ? pinned.filter(h => h !== href) : [...pinned, href];
    setPinned(next);
    storage.set("pinned_hrefs", JSON.stringify(next));
  };

  return (
      <div className={`layout${collapsed ? " sidebar-collapsed" : ""}`}>
        {/* Mobile Menu Button */}
        <button className="mobile-menu-btn" onClick={() => setMobileOpen(true)} style={{ position: "fixed", top: 8, left: 8, zIndex: 50 }}>
          <MenuIcon />
        </button>

        {mobileOpen && <div className="mobile-backdrop" onClick={() => setMobileOpen(false)} style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.5)", zIndex: 40 }} />}

        <aside className={`sidebar${mobileOpen ? " mobile-open" : ""}`} style={{ width: collapsed ? 64 : 260, transition: "width 0.2s" }}>
          <div className="sidebar-header" style={{ padding: "16px", display: "flex", flexDirection: "column", gap: "12px" }}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              {!collapsed && <div><strong>Profinaut</strong></div>}
              <button onClick={() => setCollapsed(!collapsed)}>
                {collapsed ? <ChevronRightIcon /> : <ChevronLeftIcon />}
              </button>
            </div>

            {!collapsed && (
                <>
                  <div className={`badge ${status === "OK" ? "success" : "warning"}`}>Status: {status} ({degradedCount})</div>
                  <select value={workspace} onChange={(e) => {
                    const val = e.target.value;
                    setWorkspace(val);
                    storage.set("workspace_template", val);
                    router.push(`${pathname}?workspace=${val}`);
                  }}>
                    {WORKSPACES.map(w => <option key={w} value={w}>{w}</option>)}
                  </select>
                </>
            )}
          </div>

          <nav style={{ flex: 1, padding: "0 12px" }}>
            {navItems.map((item) => {
              const isActive = pathname === item.href;
              const isPinned = pinned.includes(item.href);
              return (
                  <div key={item.href} style={{ display: "flex", alignItems: "center", marginBottom: "4px" }}>
                    <Link href={item.href === "/dashboard" ? dashboardLink : item.href} className={`nav-link ${isActive ? "active" : ""}`} style={{ flex: 1, display: "flex", alignItems: "center", gap: "8px" }}>
                      <IconFor label={item.label} />
                      {!collapsed && <span>{item.label}</span>}
                    </Link>
                    {!collapsed && (
                        <button onClick={() => togglePinned(item.href)} style={{ opacity: isPinned ? 1 : 0.3 }}>
                          {isPinned ? "★" : "☆"}
                        </button>
                    )}
                  </div>
              );
            })}
          </nav>

          <div className="sidebar-footer" style={{ padding: "16px", borderTop: "1px solid #ccc" }}>
            <button onClick={toggle} style={{ display: "flex", alignItems: "center", gap: "8px" }}>
              {theme === "dark" ? <SunIcon /> : <MoonIcon />}
              {!collapsed && <span>{theme === "dark" ? "Light Mode" : "Dark Mode"}</span>}
            </button>
          </div>
        </aside>

        <main style={{ flex: 1, padding: "20px" }}>{children}</main>
      </div>
  );
}