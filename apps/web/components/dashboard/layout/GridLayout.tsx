"use client";

import { memo, useCallback, useEffect, useMemo, useRef, useState, type ButtonHTMLAttributes, type PointerEvent as ReactPointerEvent, type ReactNode } from "react";

type GridRect = { x: number; y: number; w: number; h: number };

type GridLayoutProps<T extends { id: string; frame: { grid?: GridRect }; locked?: boolean; pinned?: boolean }> = {
  panels: T[];
  columns: number;
  rowHeight: number;
  gap: number;
  editMode: boolean;
  onPanelFrameChange: (panelId: string, frame: GridRect) => void;
  renderPanel: (panel: T, bind: { dragHandleProps: ButtonHTMLAttributes<HTMLButtonElement>; resizeHandle: ReactNode }) => ReactNode;
};

type ActionState = {
  panelId: string;
  kind: "drag" | "resize";
  startClientX: number;
  startClientY: number;
  startFrame: GridRect;
} | null;

function clamp(value: number, min: number, max: number) {
  return Math.max(min, Math.min(max, value));
}

function GridLayoutInner<T extends { id: string; frame: { grid?: GridRect }; locked?: boolean; pinned?: boolean }>({
  panels,
  columns,
  rowHeight,
  gap,
  editMode,
  onPanelFrameChange,
  renderPanel
}: GridLayoutProps<T>) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [action, setAction] = useState<ActionState>(null);

  const normalized = useMemo(
    () => panels.map((panel) => ({ ...panel, grid: panel.frame.grid ?? { x: 0, y: 0, w: 3, h: 2 } })),
    [panels]
  );

  const startAction = useCallback((panelId: string, kind: "drag" | "resize", event: ReactPointerEvent<HTMLButtonElement>) => {
    const panel = normalized.find((item) => item.id === panelId);
    if (!panel || !editMode || panel.locked) return;
    event.preventDefault();
    setAction({
      panelId,
      kind,
      startClientX: event.clientX,
      startClientY: event.clientY,
      startFrame: { ...panel.grid }
    });
  }, [editMode, normalized]);

  useEffect(() => {
    if (!action) return;

    const onMove = (event: PointerEvent) => {
      const container = containerRef.current;
      if (!container) return;
      const width = container.clientWidth;
      const colWidth = (width - gap * (columns - 1)) / columns;
      const dx = event.clientX - action.startClientX;
      const dy = event.clientY - action.startClientY;
      const deltaX = Math.round(dx / (colWidth + gap));
      const deltaY = Math.round(dy / (rowHeight + gap));
      const next = { ...action.startFrame };

      if (action.kind === "drag") {
        next.x = clamp(action.startFrame.x + deltaX, 0, columns - action.startFrame.w);
        next.y = Math.max(0, action.startFrame.y + deltaY);
      } else {
        next.w = clamp(action.startFrame.w + deltaX, 1, columns - action.startFrame.x);
        next.h = Math.max(1, action.startFrame.h + deltaY);
      }
      onPanelFrameChange(action.panelId, next);
    };

    const onUp = () => setAction(null);
    const onEsc = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      onPanelFrameChange(action.panelId, action.startFrame);
      setAction(null);
    };

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp, { once: true });
    window.addEventListener("keydown", onEsc);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      window.removeEventListener("keydown", onEsc);
    };
  }, [action, columns, gap, onPanelFrameChange, rowHeight]);

  const maxY = normalized.reduce((acc, panel) => Math.max(acc, panel.grid.y + panel.grid.h), 0);
  const style = {
    position: "relative" as const,
    minHeight: maxY * (rowHeight + gap) + rowHeight,
    border: "1px dashed var(--border-default)",
    borderRadius: 12
  };

  return (
    <div ref={containerRef} style={style}>
      {normalized.map((panel) => {
        const widthPercent = (panel.grid.w / columns) * 100;
        const leftPercent = (panel.grid.x / columns) * 100;
        const panelStyle = {
          position: "absolute" as const,
          left: `calc(${leftPercent}% + ${panel.grid.x * gap}px)`,
          width: `calc(${widthPercent}% - ${(columns - panel.grid.w) * gap / columns}px)`,
          top: panel.grid.y * (rowHeight + gap),
          height: panel.grid.h * rowHeight + (panel.grid.h - 1) * gap,
          padding: 4,
          zIndex: panel.pinned ? 2 : 1
        };

        return (
          <div key={panel.id} style={panelStyle}>
            {renderPanel(panel, {
              dragHandleProps: {
                onPointerDown: (event) => startAction(panel.id, "drag", event),
                disabled: !editMode || panel.locked,
                title: "Drag panel"
              },
              resizeHandle: (
                <button
                  className="btn btn-sm"
                  style={{ alignSelf: "flex-end", marginTop: 8 }}
                  onPointerDown={(event) => startAction(panel.id, "resize", event)}
                  disabled={!editMode || panel.locked}
                >
                  Resize
                </button>
              )
            })}
          </div>
        );
      })}
    </div>
  );
}

export const GridLayout = memo(GridLayoutInner) as typeof GridLayoutInner;
