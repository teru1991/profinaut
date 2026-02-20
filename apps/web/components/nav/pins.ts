const PINS_KEY = "profinaut.nav.pins.v1";
const HOME_KEY = "profinaut.nav.home.v1";
const WORKSPACE_KEY = "profinaut.nav.workspace.v1";

function canUseStorage() {
  return typeof window !== "undefined";
}

export function loadPinnedHrefs(): string[] {
  if (!canUseStorage()) return [];
  try {
    const raw = window.localStorage.getItem(PINS_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed.filter((v): v is string => typeof v === "string") : [];
  } catch {
    return [];
  }
}

export function savePinnedHrefs(hrefs: string[]) {
  if (!canUseStorage()) return;
  window.localStorage.setItem(PINS_KEY, JSON.stringify(Array.from(new Set(hrefs))));
}

export function togglePinnedHref(href: string): string[] {
  const current = new Set(loadPinnedHrefs());
  if (current.has(href)) current.delete(href);
  else current.add(href);
  const next = Array.from(current);
  savePinnedHrefs(next);
  return next;
}

export function loadHomeHref(): string | null {
  if (!canUseStorage()) return null;
  return window.localStorage.getItem(HOME_KEY);
}

export function saveHomeHref(href: string) {
  if (!canUseStorage()) return;
  window.localStorage.setItem(HOME_KEY, href);
}

export function loadWorkspaceTemplate(): string | null {
  if (!canUseStorage()) return null;
  return window.localStorage.getItem(WORKSPACE_KEY);
}

export function saveWorkspaceTemplate(templateId: string) {
  if (!canUseStorage()) return;
  window.localStorage.setItem(WORKSPACE_KEY, templateId);
}
