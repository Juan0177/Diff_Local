import { useCallback, useEffect, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { DiffResult, DiffRow, RowKind } from "../lib/types";
import type { ViewMode } from "./Toolbar";

const ROW_HEIGHT = 22;

interface DiffViewProps {
  result: DiffResult;
  viewMode: ViewMode;
  focusRowIndex: number | null;
}

function kindClass(kind: RowKind): string {
  switch (kind) {
    case "insert":
      return "row-insert";
    case "delete":
      return "row-delete";
    case "replace":
      return "row-replace";
    default:
      return "row-equal";
  }
}

function SideBySideRow({ row }: { row: DiffRow }) {
  return (
    <div className={`diff-row side ${kindClass(row.kind)}`}>
      <div className="pane left">
        <span className="gutter">{row.left_no ?? ""}</span>
        <pre className="code">{row.left_text ?? ""}</pre>
      </div>
      <div className="pane right">
        <span className="gutter">{row.right_no ?? ""}</span>
        <pre className="code">{row.right_text ?? ""}</pre>
      </div>
    </div>
  );
}

function UnifiedRow({ row }: { row: DiffRow }) {
  if (row.kind === "equal") {
    return (
      <div className={`diff-row unified ${kindClass(row.kind)}`}>
        <span className="gutter">{row.left_no ?? ""}</span>
        <span className="gutter">{row.right_no ?? ""}</span>
        <span className="marker"> </span>
        <pre className="code">{row.left_text ?? ""}</pre>
      </div>
    );
  }

  if (row.kind === "delete" || (row.kind === "replace" && row.left_text != null)) {
    return (
      <>
        <div className={`diff-row unified row-delete`}>
          <span className="gutter">{row.left_no ?? ""}</span>
          <span className="gutter" />
          <span className="marker">−</span>
          <pre className="code">{row.left_text ?? ""}</pre>
        </div>
        {row.kind === "replace" && row.right_text != null ? (
          <div className={`diff-row unified row-insert`}>
            <span className="gutter" />
            <span className="gutter">{row.right_no ?? ""}</span>
            <span className="marker">+</span>
            <pre className="code">{row.right_text ?? ""}</pre>
          </div>
        ) : null}
      </>
    );
  }

  return (
    <div className={`diff-row unified row-insert`}>
      <span className="gutter" />
      <span className="gutter">{row.right_no ?? ""}</span>
      <span className="marker">+</span>
      <pre className="code">{row.right_text ?? ""}</pre>
    </div>
  );
}

export function DiffView({ result, viewMode, focusRowIndex }: DiffViewProps) {
  const parentRef = useRef<HTMLDivElement>(null);
  const rows = result.rows;

  // Unified replace rows expand to 2 visual lines — approximate with fixed height
  // for virtualization; replace still uses one virtual index (side-by-side).
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: useCallback(
      (index: number) => {
        if (viewMode === "unified" && rows[index]?.kind === "replace") {
          return ROW_HEIGHT * 2;
        }
        return ROW_HEIGHT;
      },
      [viewMode, rows],
    ),
    overscan: 24,
  });

  useEffect(() => {
    if (focusRowIndex == null) return;
    virtualizer.scrollToIndex(focusRowIndex, { align: "center" });
  }, [focusRowIndex, virtualizer]);

  useEffect(() => {
    virtualizer.measure();
  }, [viewMode, virtualizer]);

  return (
    <div className="diff-shell">
      {viewMode === "side-by-side" ? (
        <div className="diff-header side">
          <div className="pane left">
            <span className="gutter">#</span>
            <span className="header-path" title={result.left_path}>
              {result.left_path}
            </span>
          </div>
          <div className="pane right">
            <span className="gutter">#</span>
            <span className="header-path" title={result.right_path}>
              {result.right_path}
            </span>
          </div>
        </div>
      ) : (
        <div className="diff-header unified">
          <span className="header-path" title={result.left_path}>
            {result.left_path}
          </span>
          <span className="vs">→</span>
          <span className="header-path" title={result.right_path}>
            {result.right_path}
          </span>
        </div>
      )}

      <div className="diff-scroll" ref={parentRef}>
        <div
          className="diff-virtual"
          style={{ height: `${virtualizer.getTotalSize()}px` }}
        >
          {virtualizer.getVirtualItems().map((item) => {
            const row = rows[item.index];
            return (
              <div
                key={item.key}
                data-index={item.index}
                ref={virtualizer.measureElement}
                className={
                  focusRowIndex === item.index ? "virt-row focused" : "virt-row"
                }
                style={{
                  position: "absolute",
                  top: 0,
                  left: 0,
                  width: "100%",
                  transform: `translateY(${item.start}px)`,
                }}
              >
                {viewMode === "side-by-side" ? (
                  <SideBySideRow row={row} />
                ) : (
                  <UnifiedRow row={row} />
                )}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
