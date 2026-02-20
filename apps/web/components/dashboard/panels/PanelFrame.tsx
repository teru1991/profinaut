"use client";

import type { ButtonHTMLAttributes, ReactNode } from "react";

import { formatAge, statusToBadgeClass } from "../ui/badges";
import type { WidgetQuality } from "../widgets/runtime";

type PanelFrameProps = {
  title: string;
  quality?: WidgetQuality;
  locked?: boolean;
  pinned?: boolean;
  protectedPanel?: boolean;
  canEdit: boolean;
  onDuplicate: () => void;
  onRemove: () => void;
  onToggleLock: () => void;
  onTogglePin: () => void;
  onToggleProtect: () => void;
  children: ReactNode;
  dragHandleProps?: ButtonHTMLAttributes<HTMLButtonElement>;
  resizeHandle?: React.ReactNode;
};

export function PanelFrame({
  title,
  quality,
  locked,
  pinned,
  protectedPanel,
  canEdit,
  onDuplicate,
  onRemove,
  onToggleLock,
  onTogglePin,
  onToggleProtect,
  children,
  dragHandleProps,
  resizeHandle
}: PanelFrameProps) {
  const lastAge = quality?.lastSuccessTs ? Date.now() - quality.lastSuccessTs : null;
  return (
    <div className="card" style={{ height: "100%", display: "flex", flexDirection: "column" }}>
      <div className="card-header" style={{ gap: 8 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}>
          <h3 className="card-title">{title}</h3>
          {quality?.status && <span className={statusToBadgeClass(quality.status)}>{quality.status}</span>}
          {typeof lastAge === "number" && lastAge > 30000 && <span className="badge badge-warning">stale {formatAge(lastAge)}</span>}
          {locked && <span className="badge badge-info">locked</span>}
          {pinned && <span className="badge badge-accent">pinned</span>}
          {protectedPanel && <span className="badge">protected</span>}
        </div>
        {canEdit && (
          <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
            <button className="btn btn-sm" {...dragHandleProps}>Move</button>
            <button className="btn btn-sm" onClick={onDuplicate}>Duplicate</button>
            <button className="btn btn-sm" onClick={onToggleLock}>{locked ? "Unlock" : "Lock"}</button>
            <button className="btn btn-sm" onClick={onTogglePin}>{pinned ? "Unpin" : "Pin"}</button>
            <button className="btn btn-sm" onClick={onToggleProtect}>{protectedPanel ? "Unprotect" : "Protect"}</button>
            <button className="btn btn-sm btn-danger" onClick={onRemove}>Remove</button>
          </div>
        )}
      </div>
      <div style={{ flex: 1, minHeight: 0 }}>{children}</div>
      {canEdit && resizeHandle}
    </div>
  );
}
