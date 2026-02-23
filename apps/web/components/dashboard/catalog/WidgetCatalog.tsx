"use client";

import { useMemo, useState } from "react";
// registryのパスや実装に合わせて適切にインポートしてください
import { listWidgets } from "../widgets/registry";

type WidgetCatalogProps = {
    /**
     * 親コンポーネントから渡されるコールバック関数。
     * このコンポーネントを呼び出す親ファイルも "use client" である必要があります。
     */
    onAdd: (widgetId: string) => void;
};

export function WidgetCatalog({ onAdd }: WidgetCatalogProps) {
    const [query, setQuery] = useState("");
    const [category, setCategory] = useState<string>("all");

    // メモ化されたウィジェットリストのフィルタリング
    const filteredWidgets = useMemo(() => {
        const allWidgets = listWidgets();
        const normalizedQuery = query.trim().toLowerCase();

        return allWidgets.filter((widget) => {
            // カテゴリフィルタリング
            const matchesCategory = category === "all" || widget.category === category;
            if (!matchesCategory) return false;

            // 検索クエリフィルタリング
            const matchesQuery = !normalizedQuery ||
                widget.title.toLowerCase().includes(normalizedQuery) ||
                (widget.description?.toLowerCase().includes(normalizedQuery) ?? false);

            return matchesQuery;
        });
    }, [category, query]);

    return (
        <div className="card" style={{ marginBottom: "12px" }}>
            <div className="card-header">
                <h3 className="card-title">Widget Catalog</h3>
            </div>

            <div style={{
                display: "flex",
                gap: "8px",
                flexWrap: "wrap",
                marginBottom: "16px",
                padding: "0 4px"
            }}>
                <input
                    type="text"
                    className="input-search"
                    placeholder="Search widget..."
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                    style={{ flex: 1, minWidth: "150px" }}
                />
                <select
                    value={category}
                    onChange={(e) => setCategory(e.target.value)}
                    style={{ padding: "4px 8px" }}
                >
                    <option value="all">All categories</option>
                    <option value="Status">Status</option>
                    <option value="Ops">Ops</option>
                    <option value="Bots">Bots</option>
                    <option value="Markets">Markets</option>
                    <option value="Utility">Utility</option>
                </select>
            </div>

            <div
                className="card-grid"
                style={{
                    display: "grid",
                    gap: "12px",
                    gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))"
                }}
            >
                {filteredWidgets.length > 0 ? (
                    filteredWidgets.map((widget) => (
                        <div
                            key={widget.id}
                            className="card widget-item"
                            style={{
                                margin: 0,
                                padding: "12px",
                                border: "1px solid #eee",
                                display: "flex",
                                flexDirection: "column",
                                justifyContent: "space-between"
                            }}
                        >
                            <div>
                                <div className="text-sm" style={{ fontWeight: "bold" }}>
                                    {widget.title}
                                </div>
                                <div
                                    className="text-muted text-sm"
                                    style={{ fontSize: "0.8rem", color: "#666", marginTop: "4px" }}
                                >
                                    {widget.description ?? widget.category}
                                </div>
                            </div>
                            <button
                                className="btn btn-sm btn-primary"
                                style={{ marginTop: "12px", width: "100%" }}
                                onClick={() => onAdd(widget.id)}
                            >
                                Add to Dashboard
                            </button>
                        </div>
                    ))
                ) : (
                    <div style={{ gridColumn: "1 / -1", textAlign: "center", padding: "20px", color: "#999" }}>
                        No widgets found matching your criteria.
                    </div>
                )}
            </div>
        </div>
    );
}