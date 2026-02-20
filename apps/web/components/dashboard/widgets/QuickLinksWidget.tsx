"use client";

import Link from "next/link";

import { type WidgetProps, useReportQuality } from "./runtime";

const links = [
  ["Bots", "/bots"],
  ["Markets", "/markets"],
  ["Portfolio", "/portfolio"],
  ["Commands", "/commands"],
  ["Analytics", "/analytics"],
  ["Modules", "/admin/modules"]
] as const;

export function QuickLinksWidget({ reportQuality }: WidgetProps) {
  useReportQuality(reportQuality, { status: "OK", lastSuccessTs: Date.now() });
  return (
    <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
      {links.map(([label, href]) => (
        <Link key={href} className="btn btn-sm" href={href}>{label}</Link>
      ))}
    </div>
  );
}
