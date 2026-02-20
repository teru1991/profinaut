import { createElement, type JSX } from "react";

import type { WidgetProps } from "./runtime";
import { BotsKpiWidget } from "./BotsKpiWidget";
import { DegradedComponentsWidget } from "./DegradedComponentsWidget";
import { QuickLinksWidget } from "./QuickLinksWidget";
import { SystemStatusWidget } from "./SystemStatusWidget";

export type WidgetDefinition = {
  id: string;
  title: string;
  category: "Status" | "Ops" | "Bots" | "Markets" | "Utility";
  defaultGrid: { w: number; h: number };
  minGrid?: { w: number; h: number };
  description?: string;
  requires?: { status?: string[]; capabilities?: string[] };
  render: (props: WidgetProps) => JSX.Element;
};

export const WIDGETS: Record<string, WidgetDefinition> = {
  "system-status": {
    id: "system-status",
    title: "System Status",
    category: "Status",
    defaultGrid: { w: 3, h: 2 },
    minGrid: { w: 2, h: 1 },
    description: "Overall dashboard and component status.",
    render: (props) => createElement(SystemStatusWidget, props)
  },
  "degraded-components": {
    id: "degraded-components",
    title: "Degraded Components",
    category: "Status",
    defaultGrid: { w: 6, h: 2 },
    minGrid: { w: 3, h: 1 },
    description: "Lists non-OK components with reasons.",
    render: (props) => createElement(DegradedComponentsWidget, props)
  },
  "bots-overview": {
    id: "bots-overview",
    title: "Bots KPI",
    category: "Bots",
    defaultGrid: { w: 3, h: 2 },
    minGrid: { w: 2, h: 1 },
    description: "Summary of bot counts and degraded state.",
    render: (props) => createElement(BotsKpiWidget, props)
  },
  "quick-nav": {
    id: "quick-nav",
    title: "Quick Links",
    category: "Utility",
    defaultGrid: { w: 4, h: 1 },
    minGrid: { w: 2, h: 1 },
    description: "One-click navigation links.",
    render: (props) => createElement(QuickLinksWidget, props)
  }
};

export function listWidgets(): WidgetDefinition[] {
  return Object.values(WIDGETS).sort((a, b) => a.title.localeCompare(b.title));
}
