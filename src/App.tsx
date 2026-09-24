import { useCallback, useEffect, useState } from "react";
import { DiffView } from "./components/DiffView";
import { StatusBar, Toolbar, type ViewMode } from "./components/Toolbar";
import { computeDiff, pickFile } from "./lib/tauri";
import type { DiffResult } from "./lib/types";
import "./App.css";

function App() {
  const [leftPath, setLeftPath] = useState<string | null>(null);
  const [rightPath, setRightPath] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>("side-by-side");
  const [result, setResult] = useState<DiffResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [changeCursor, setChangeCursor] = useState(0);
  const [focusRowIndex, setFocusRowIndex] = useState<number | null>(null);

  const changeCount = result?.change_indices.length ?? 0;

  const onPickLeft = useCallback(async () => {
    try {
      const path = await pickFile();
      if (path) {
        setLeftPath(path);
        setResult(null);
        setError(null);
      }
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const onPickRight = useCallback(async () => {
    try {
      const path = await pickFile();
      if (path) {
        setRightPath(path);
        setResult(null);
        setError(null);
      }
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const onCompare = useCallback(async () => {
    if (!leftPath || !rightPath) return;
    setBusy(true);
    setError(null);
    try {
      const diff = await computeDiff(leftPath, rightPath);
      setResult(diff);
      setChangeCursor(0);
      setFocusRowIndex(
        diff.change_indices.length > 0 ? diff.change_indices[0] : null,
      );
    } catch (e) {
      setResult(null);
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }, [leftPath, rightPath]);

  const onPrevChange = useCallback(() => {
    if (!result || result.change_indices.length === 0) return;
    const next =
      (changeCursor - 1 + result.change_indices.length) %
      result.change_indices.length;
    setChangeCursor(next);
    setFocusRowIndex(result.change_indices[next]);
  }, [result, changeCursor]);

  const onNextChange = useCallback(() => {
    if (!result || result.change_indices.length === 0) return;
    const next = (changeCursor + 1) % result.change_indices.length;
    setChangeCursor(next);
    setFocusRowIndex(result.change_indices[next]);
  }, [result, changeCursor]);

  useEffect(() => {
    const onKey = (ev: KeyboardEvent) => {
      const target = ev.target as HTMLElement | null;
      if (
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.isContentEditable)
      ) {
        return;
      }
      if (ev.key === "n" || ev.key === "N" || ev.key === "]") {
        ev.preventDefault();
        onNextChange();
      } else if (ev.key === "p" || ev.key === "P" || ev.key === "[") {
        ev.preventDefault();
        onPrevChange();
      } else if (ev.key === "Enter" && (ev.ctrlKey || ev.metaKey)) {
        ev.preventDefault();
        void onCompare();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onNextChange, onPrevChange, onCompare]);

  return (
    <div className="app">
      <div className="atmosphere" aria-hidden />
      <Toolbar
        leftPath={leftPath}
        rightPath={rightPath}
        viewMode={viewMode}
        busy={busy}
        hasDiff={Boolean(result)}
        changeCount={changeCount}
        currentChange={changeCount === 0 ? 0 : changeCursor}
        onPickLeft={() => void onPickLeft()}
        onPickRight={() => void onPickRight()}
        onCompare={() => void onCompare()}
        onViewMode={setViewMode}
        onPrevChange={onPrevChange}
        onNextChange={onNextChange}
      />

      <main className="main">
        {result ? (
          <DiffView
            result={result}
            viewMode={viewMode}
            focusRowIndex={focusRowIndex}
          />
        ) : (
          <div className="empty-state">
            <h2>Apri due file da confrontare</h2>
            <p>
              Diff_Local calcola un diff allineato lato per lato, con scroll
              virtualizzato pensato per file di decine di migliaia di righe.
            </p>
            <p className="hint">
              Scorciatoie: <kbd>N</kbd> / <kbd>P</kbd> o <kbd>]</kbd> /{" "}
              <kbd>[</kbd> per le modifiche · <kbd>Ctrl</kbd>+<kbd>Enter</kbd>{" "}
              confronta
            </p>
          </div>
        )}
      </main>

      <StatusBar result={result} error={error} />
    </div>
  );
}

export default App;
