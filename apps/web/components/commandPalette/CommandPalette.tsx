"use client";

import { useEffect, useMemo, useState } from "react";
import { usePathname, useRouter } from "next/navigation";

import { buildDashboardUrl, parseDashboardUrl } from "../dashboard/core/router";
import { buildCommands } from "./commands";

export function CommandPalette() {
  const pathname = usePathname();
  const router = useRouter();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");

  const navigate = (href: string) => {
    router.push(href);
    setOpen(false);
    setQuery("");
  };

  const copyCurrentDashboardLink = async () => {
    if (pathname !== "/dashboard") return;
    const href = buildDashboardUrl(parseDashboardUrl(typeof window === "undefined" ? new URLSearchParams() : new URLSearchParams(window.location.search)));
    await navigator.clipboard.writeText(`${window.location.origin}${href}`);
    setOpen(false);
  };

  const commands = useMemo(
    () =>
      buildCommands({
        pathname,
        searchParams: typeof window === "undefined" ? new URLSearchParams() : new URLSearchParams(window.location.search),
        navigate,
        copyCurrentDashboardLink
      }),
    [pathname]
  );

  const filtered = useMemo(() => {
    const token = query.trim().toLowerCase();
    if (!token) return commands;
    return commands.filter((command) => `${command.label} ${command.keywords}`.toLowerCase().includes(token));
  }, [commands, query]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setOpen((value) => !value);
      }
      if (event.key === "Escape") {
        setOpen(false);
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  if (!open) return null;

  return (
    <div className="dialog-overlay" onClick={() => setOpen(false)}>
      <div className="dialog" style={{ width: "min(720px, 100%)" }} onClick={(event) => event.stopPropagation()}>
        <h3 className="dialog-title">Command Palette</h3>
        <input
          autoFocus
          className="search-input"
          placeholder="Type a command..."
          value={query}
          onChange={(event) => setQuery(event.target.value)}
        />
        <div style={{ display: "grid", gap: 8 }}>
          {filtered.map((command) => (
            <button key={command.id} className="btn" style={{ justifyContent: "flex-start" }} onClick={command.action}>
              {command.label}
            </button>
          ))}
          {filtered.length === 0 && <div className="text-sm text-muted">No matches.</div>}
        </div>
      </div>
    </div>
  );
}
