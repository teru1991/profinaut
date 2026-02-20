import { type GlobalFilters, normalizeFilters } from "./filters";

export type DashboardRouteState = {
  pageId?: string;
  focus?: boolean;
  filters?: GlobalFilters;
};

export function buildDashboardUrl(state: DashboardRouteState): string {
  const params = new URLSearchParams();

  if (state.pageId) {
    params.set("pageId", state.pageId);
  }
  if (state.focus) {
    params.set("focus", "1");
  }

  const filters = normalizeFilters(state.filters ?? {});
  if (filters.venue) params.set("venue", filters.venue);
  if (filters.bot) params.set("bot", filters.bot);
  if (filters.symbol) params.set("symbol", filters.symbol);
  if (filters.timeRange) params.set("range", filters.timeRange);

  const query = params.toString();
  return `/dashboard${query ? `?${query}` : ""}`;
}

export function parseDashboardUrl(searchParams?: URLSearchParams): DashboardRouteState {
  const params =
    searchParams ??
    (typeof window !== "undefined"
      ? new URL(window.location.href).searchParams
      : new URLSearchParams());

  return {
    pageId: params.get("pageId") ?? undefined,
    focus: params.get("focus") === "1",
    filters: normalizeFilters({
      venue: params.get("venue") ?? undefined,
      bot: params.get("bot") ?? undefined,
      symbol: params.get("symbol") ?? undefined,
      timeRange: (params.get("range") as GlobalFilters["timeRange"]) ?? undefined
    })
  };
}
