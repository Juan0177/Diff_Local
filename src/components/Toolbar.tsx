import type { DiffResult } from "../lib/types";
import { basename, formatBytes } from "../lib/types";

export type ViewMode = "side-by-side" | "unified";

interface ToolbarProps {
  leftPath: string | null;
  rightPath: string | null;
  viewMode: ViewMode;
  busy: boolean;
  hasDiff: boolean;
  changeCount: number;
  currentChange: number;
  onPickLeft: () => void;
  onPickRight: () => void;
  onCompare: () => void;
  onViewMode: (mode: ViewMode) => void;
  onPrevChange: () => void;
  onNextChange: () => void;
}

export function Toolbar({
  leftPath,
  rightPath,
  viewMode,
  busy,
  hasDiff,
  changeCount,
  currentChange,
  onPickLeft,
  onPickRight,
  onCompare,
  onViewMode,
  onPrevChange,
  onNextChange,
}: ToolbarProps) {
  const canCompare = Boolean(leftPath && rightPath) && !busy;

  return (
    <header className="toolbar">
      <div className="brand-block">
        <h1 className="brand">Diff_Local</h1>
        <p className="tagline">Confronto di file di testo, anche grandi</p>
      </div>

      <div className="toolbar-actions">
        <div className="file-pickers">
          <button type="button" className="btn ghost" onClick={onPickLeft} disabled={busy}>
            <span className="btn-label">Sinistra</span>
            <span className="btn-path" title={leftPath ?? undefined}>
              {leftPath ? basename(leftPath) : "Scegli file…"}
            </span>
          </button>
          <button type="button" className="btn ghost" onClick={onPickRight} disabled={busy}>
            <span className="btn-label">Destra</span>
            <span className="btn-path" title={rightPath ?? undefined}>
              {rightPath ? basename(rightPath) : "Scegli file…"}
            </span>
          </button>
        </div>

        <button
          type="button"
          className="btn primary"
          onClick={onCompare}
          disabled={!canCompare}
        >
          {busy ? "Calcolo…" : "Confronta"}
        </button>

        <div className="mode-toggle" role="group" aria-label="Modalità vista">
          <button
            type="button"
            className={viewMode === "side-by-side" ? "btn toggle active" : "btn toggle"}
            onClick={() => onViewMode("side-by-side")}
          >
            Affiancata
          </button>
          <button
            type="button"
            className={viewMode === "unified" ? "btn toggle active" : "btn toggle"}
            onClick={() => onViewMode("unified")}
          >
            Unificata
          </button>
        </div>

        <div className="nav-changes">
          <button
            type="button"
            className="btn icon"
            onClick={onPrevChange}
            disabled={!hasDiff || changeCount === 0}
            title="Modifica precedente (P / [)"
            aria-label="Modifica precedente"
          >
            ↑
          </button>
          <span className="change-counter">
            {changeCount === 0
              ? "0 modifiche"
              : `${currentChange + 1} / ${changeCount}`}
          </span>
          <button
            type="button"
            className="btn icon"
            onClick={onNextChange}
            disabled={!hasDiff || changeCount === 0}
            title="Modifica successiva (N / ])"
            aria-label="Modifica successiva"
          >
            ↓
          </button>
        </div>
      </div>
    </header>
  );
}

interface StatusBarProps {
  result: DiffResult | null;
  error: string | null;
}

export function StatusBar({ result, error }: StatusBarProps) {
  if (error) {
    return (
      <footer className="status-bar error">
        <span>{error}</span>
      </footer>
    );
  }

  if (!result) {
    return (
      <footer className="status-bar muted">
        <span>Seleziona due file e premi Confronta</span>
      </footer>
    );
  }

  const { stats, left_bytes, right_bytes, warning } = result;

  return (
    <footer className="status-bar">
      <span className="stat insert">+{stats.insert}</span>
      <span className="stat delete">−{stats.delete}</span>
      <span className="stat replace">~{stats.replace}</span>
      <span className="stat equal">={stats.equal}</span>
      <span className="sep" />
      <span>
        {formatBytes(left_bytes)} · {formatBytes(right_bytes)} · {stats.total_rows}{" "}
        righe
      </span>
      {warning ? <span className="warn">{warning}</span> : null}
    </footer>
  );
}
