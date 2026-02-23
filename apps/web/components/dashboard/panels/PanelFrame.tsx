"use client";

import type { ButtonHTMLAttributes, ReactNode } from "react";

// プロジェクトの構造に合わせてインポートパスは調整してください
import { formatAge, statusToBadgeClass } from "../ui/badges";
import type { WidgetQuality } from "../widgets/runtime";

type PanelFrameProps = {
  title: string;
  quality?: WidgetQuality;
  locked?: boolean;
  pinned?: boolean;
  protectedPanel?: boolean;
  canEdit: boolean;

  // ⚠️ 注意: これらの関数Propsを渡す親コンポーネントは、
  // 必ず先頭に "use client" を記述する必要があります。
  onDuplicate: () => void;
  onRemove: () => void;
  onToggleLock: () => void;
  onTogglePin: () => void;
  onToggleProtect: () => void;

  children: ReactNode;
  dragHandleProps?: ButtonHTMLAttributes<HTMLButtonElement>;
  resizeHandle?: ReactNode;
};

export function PanelFrame({
                             title,
                             quality,
                             locked = false,
                             pinned = false,
                             protectedPanel = false,
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
  // ステータスの鮮度を計算
  const lastAge = quality?.lastSuccessTs ? Date.now() - quality.lastSuccessTs : null;
  const isStale = typeof lastAge === "number" && lastAge > 30000;

  return (
      <div
          className="card panel-frame"
          style={{ height: "100%", display: "flex", flexDirection: "column" }}
      >
        {/* --- ヘッダー領域 --- */}
        <div className="card-header" style={{ gap: "12px", paddingBottom: "12px" }}>

          {/* タイトルとバッジ */}
          <div style={{ display: "flex", alignItems: "center", gap: "8px", flexWrap: "wrap" }}>
            <h3 className="card-title" style={{ margin: 0 }}>{title}</h3>

            {quality?.status && (
                <span className={statusToBadgeClass(quality.status)}>
              {quality.status}
            </span>
            )}

            {isStale && (
                <span className="badge badge-warning">
              stale {formatAge(lastAge)}
            </span>
            )}

            {locked && <span className="badge badge-info">locked</span>}
            {pinned && <span className="badge badge-accent">pinned</span>}
            {protectedPanel && <span className="badge">protected</span>}
          </div>

          {/* コントロールボタン群 */}
          {canEdit && (
              <div style={{ display: "flex", gap: "6px", flexWrap: "wrap", marginTop: "8px" }}>
                <button className="btn btn-sm btn-ghost" type="button" {...dragHandleProps}>
                  Move
                </button>
                <button className="btn btn-sm btn-secondary" type="button" onClick={onDuplicate}>
                  Duplicate
                </button>
                <button className="btn btn-sm btn-secondary" type="button" onClick={onToggleLock}>
                  {locked ? "Unlock" : "Lock"}
                </button>
                <button className="btn btn-sm btn-secondary" type="button" onClick={onTogglePin}>
                  {pinned ? "Unpin" : "Pin"}
                </button>
                <button className="btn btn-sm btn-secondary" type="button" onClick={onToggleProtect}>
                  {protectedPanel ? "Unprotect" : "Protect"}
                </button>
                <button className="btn btn-sm btn-danger" type="button" onClick={onRemove}>
                  Remove
                </button>
              </div>
          )}
        </div>

        {/* --- コンテンツ領域 --- */}
        <div className="panel-content" style={{ flex: 1, minHeight: 0, overflow: "auto" }}>
          {children}
        </div>

        {/* --- リサイズハンドル --- */}
        {canEdit && resizeHandle}
      </div>
  );
}