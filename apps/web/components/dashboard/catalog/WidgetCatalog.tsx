"use client";

import { useMemo, useState } from "react";

import { listWidgets } from "../widgets/registry";

type WidgetCatalogProps = {
  onAdd: (widgetId: string) => void;
};

export function WidgetCatalog({ onAdd }: WidgetCatalogProps) {
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState<string>("all");
  const widgets = useMemo(() => {
    return listWidgets().filter((widget) => {
      if (category !== "all" && widget.category !== category) return false;
      if (!query.trim()) return true;
      return widget.title.toLowerCase().includes(query.toLowerCase());
    });
  }, [category, query]);

  return (
    <div className="card" style={{ marginBottom: 12 }}>
      <div className="card-header">
        <h3 className="card-title">Widget Catalog</h3>
      </div>
      <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginBottom: 10 }}>
        <input placeholder="Search widget" value={query} onChange={(e) => setQuery(e.target.value)} />
        <select value={category} onChange={(e) => setCategory(e.target.value)}>
          <option value="all">All categories</option>
          <option value="Status">Status</option>
          <option value="Ops">Ops</option>
          <option value="Bots">Bots</option>
          <option value="Markets">Markets</option>
          <option value="Utility">Utility</option>
        </select>
      </div>
      <div className="card-grid" style={{ gridTemplateColumns: "repeat(auto-fill,minmax(220px,1fr))" }}>
        {widgets.map((widget) => (
          <div key={widget.id} className="card" style={{ margin: 0 }}>
            <div className="text-sm"><strong>{widget.title}</strong></div>
            <div className="text-muted text-sm">{widget.description ?? widget.category}</div>
            <button className="btn btn-sm" style={{ marginTop: 8 }} onClick={() => onAdd(widget.id)}>Add</button>
          </div>
        ))}
      </div>
    </div>
  );
}
