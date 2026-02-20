export type NavItem = {
  href: string;
  label: string;
  group: string;
  priority: number;
  requires?: { overall_status?: string[] };
};

export type NavGroup = { id: string; title: string };

type BuildNavInput = {
  overallStatus?: string | null;
  degradedCount?: number;
  pinned?: string[];
  homeHref?: string | null;
};

const INCIDENT_PRIORITIES: Record<string, number> = {
  "/dashboard": -100,
  "/commands": -90,
  "/bots": -80,
  "/dashboard?workspace=incident": -70
};

const BASE_ITEMS: NavItem[] = [
  { href: "/dashboard", label: "Dashboard", group: "core", priority: 10 },
  { href: "/dashboard?workspace=incident", label: "Incidents", group: "core", priority: 95 },
  { href: "/bots", label: "Bots", group: "core", priority: 20 },
  { href: "/commands", label: "Commands", group: "core", priority: 30 },
  { href: "/portfolio", label: "Portfolio", group: "core", priority: 40 },
  { href: "/markets", label: "Markets", group: "core", priority: 50 },
  { href: "/analytics", label: "Analytics", group: "intel", priority: 60 },
  { href: "/datasets", label: "Datasets", group: "intel", priority: 70 },
  { href: "/admin/modules", label: "Admin", group: "system", priority: 80 }
];

const GROUPS: NavGroup[] = [
  { id: "pinned", title: "Pinned" },
  { id: "home", title: "Home" },
  { id: "core", title: "Operations" },
  { id: "intel", title: "Intelligence" },
  { id: "system", title: "System" }
];

function normalizeStatus(status?: string | null, degradedCount?: number) {
  const value = (status ?? "").toUpperCase();
  if (value === "DEGRADED" || value === "DOWN" || value === "UNKNOWN") return value;
  if (typeof degradedCount === "number" && degradedCount > 0) return "DEGRADED";
  return "OK";
}

export function buildNav({ overallStatus, degradedCount, pinned = [], homeHref }: BuildNavInput): { groups: NavGroup[]; items: NavItem[] } {
  const status = normalizeStatus(overallStatus, degradedCount);
  const pinSet = new Set(pinned);

  const scored = BASE_ITEMS.map((item) => {
    let score = item.priority;

    if (status !== "OK" && INCIDENT_PRIORITIES[item.href] !== undefined) {
      score = INCIDENT_PRIORITIES[item.href];
    }

    if (pinSet.has(item.href)) {
      score -= 1000;
    }

    if (homeHref && homeHref === item.href) {
      score -= 500;
    }

    return { ...item, priority: score };
  });

  return {
    groups: GROUPS,
    items: scored.sort((a, b) => a.priority - b.priority)
  };
}
