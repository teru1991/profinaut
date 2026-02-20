"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import type { GlobalFilters } from "../core/filters";
import { normalizeStatus, type HealthStatus } from "../ui/badges";

export type FetchResult<T> = { ok: boolean; data: T | null; error: string | null; ts: number };
export type WidgetQuality = { status: HealthStatus; lastSuccessTs?: number | null };
export type WidgetCapabilities = { status: HealthStatus; components: string[] };

export type WidgetProps = {
  globalFilters: GlobalFilters;
  refreshNow: () => void;
  capabilities: WidgetCapabilities;
  reportQuality: (quality: WidgetQuality) => void;
};

export async function fetchJson<T>(url: string, timeoutMs = 6000): Promise<FetchResult<T>> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const response = await fetch(url, { cache: "no-store", signal: controller.signal });
    if (!response.ok) {
      return { ok: false, data: null, error: `HTTP ${response.status}`, ts: Date.now() };
    }
    const data = (await response.json()) as T;
    return { ok: true, data, error: null, ts: Date.now() };
  } catch (error) {
    return {
      ok: false,
      data: null,
      error: error instanceof Error ? error.message : "Request failed",
      ts: Date.now()
    };
  } finally {
    clearTimeout(timer);
  }
}

export function useWidgetQuery<T>(url: string, refreshMs = 10000) {
  const [result, setResult] = useState<FetchResult<T>>({ ok: false, data: null, error: null, ts: Date.now() });
  const [lastSuccess, setLastSuccess] = useState<{ data: T; ts: number } | null>(null);

  const load = useCallback(async () => {
    const next = await fetchJson<T>(url, 7000);
    setResult(next);
    if (next.ok && next.data) setLastSuccess({ data: next.data, ts: next.ts });
  }, [url]);

  useEffect(() => {
    load();
    const id = setInterval(load, refreshMs);
    return () => clearInterval(id);
  }, [load, refreshMs]);

  return { result, lastSuccess, refreshNow: load };
}

type StatusSummary = { overall_status?: string; components?: { name?: string }[] } | null;

export function useWidgetCapabilities(status: StatusSummary): WidgetCapabilities {
  return useMemo(
    () => ({
      status: normalizeStatus(status?.overall_status),
      components: (status?.components ?? []).map((c) => c.name ?? "").filter(Boolean)
    }),
    [status]
  );
}

export function useReportQuality(reportQuality: (quality: WidgetQuality) => void, quality: WidgetQuality) {
  const prev = useRef<string>("");
  useEffect(() => {
    const token = `${quality.status}-${quality.lastSuccessTs ?? "none"}`;
    if (token !== prev.current) {
      reportQuality(quality);
      prev.current = token;
    }
  }, [quality, reportQuality]);
}
