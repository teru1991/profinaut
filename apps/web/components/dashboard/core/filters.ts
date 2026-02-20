export type DashboardTimeRange = "15m" | "1h" | "1d" | "7d";

export type GlobalFilters = {
  venue?: string;
  bot?: string;
  symbol?: string;
  timeRange?: DashboardTimeRange;
};

export function normalizeFilters(input: Partial<GlobalFilters>): GlobalFilters {
  const timeRange = input.timeRange && ["15m", "1h", "1d", "7d"].includes(input.timeRange)
    ? input.timeRange
    : undefined;

  return {
    venue: input.venue?.trim() || undefined,
    bot: input.bot?.trim() || undefined,
    symbol: input.symbol?.trim() || undefined,
    timeRange
  };
}

export function serializeFilters(filters: GlobalFilters): string {
  return JSON.stringify({
    venue: filters.venue ?? "",
    bot: filters.bot ?? "",
    symbol: filters.symbol ?? "",
    timeRange: filters.timeRange ?? ""
  });
}

export function deserializeFilters(raw: string | null): GlobalFilters {
  if (!raw) {
    return {};
  }
  try {
    const parsed = JSON.parse(raw) as Partial<GlobalFilters>;
    return normalizeFilters(parsed);
  } catch {
    return {};
  }
}
