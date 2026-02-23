import { buildDashboardUrl, parseDashboardUrl } from "../dashboard/core/router";

export type PaletteCommand = {
  id: string;
  label: string;
  keywords: string;
  action: () => void;
};

type BuildCommandsInput = {
  pathname: string;
  searchParams: URLSearchParams;
  navigate: (href: string) => void;
  copyCurrentDashboardLink: () => void;
};

const PAGE_ROUTES = [
  ["Dashboard", "/dashboard"],
  ["Bots", "/bots"],
  ["Markets", "/markets"],
  ["Portfolio", "/portfolio"],
  ["Commands", "/commands"],
  ["Analytics", "/analytics"],
  ["Admin", "/admin/modules"]
] as const;

export function buildCommands({ pathname, searchParams, navigate, copyCurrentDashboardLink }: BuildCommandsInput): PaletteCommand[] {
  const routeState = parseDashboardUrl(searchParams);
  const dashboardBase = buildDashboardUrl({ filters: routeState.filters, pageId: routeState.pageId });

  const pageCommands = PAGE_ROUTES.map(([label, href]) => ({
    id: `page-${label.toLowerCase()}`,
    label: `Go to ${label}`,
    keywords: `nav page ${label.toLowerCase()}`,
    action: () => navigate(label === "Dashboard" ? dashboardBase : href)
  }));

  return [
    ...pageCommands,
    {
      id: "dashboard-add-widget",
      label: "Add Widget",
      keywords: "dashboard widget catalog add",
      action: () => {
        const href = `${dashboardBase}${dashboardBase.includes("?") ? "&" : "?"}mode=edit&catalog=1`;
        navigate(href);
      }
    },
    {
      id: "dashboard-focus",
      label: "Toggle Dashboard Focus/Kiosk",
      keywords: "dashboard focus kiosk",
      action: () => {
        const next = buildDashboardUrl({
          ...routeState,
          focus: !routeState.focus
        });
        navigate(next);
      }
    },
    {
      id: "filter-venue-binance",
      label: "Set Venue filter: binance",
      keywords: "filter venue quick set binance",
      action: () => navigate(buildDashboardUrl({ ...routeState, filters: { ...routeState.filters, venue: "binance" } }))
    },
    {
      id: "filter-symbol-btcusdt",
      label: "Set Symbol filter: BTCUSDT",
      keywords: "filter symbol quick set btcusdt",
      action: () => navigate(buildDashboardUrl({ ...routeState, filters: { ...routeState.filters, symbol: "BTCUSDT" } }))
    },
    {
      id: "dashboard-copy-link",
      label: "Copy dashboard deep link",
      keywords: "copy link deep link dashboard",
      action: copyCurrentDashboardLink
    }
  ].filter((command) => pathname === "/dashboard" || !command.id.startsWith("dashboard-copy-link"));
}
