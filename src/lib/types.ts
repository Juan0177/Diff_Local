export type RowKind = "equal" | "insert" | "delete" | "replace";

export interface InlineSpan {
  text: string;
  changed: boolean;
}

export interface DiffRow {
  kind: RowKind;
  left_no: number | null;
  right_no: number | null;
  left_text: string | null;
  right_text: string | null;
  left_spans: InlineSpan[] | null;
  right_spans: InlineSpan[] | null;
}

export interface DiffStats {
  equal: number;
  insert: number;
  delete: number;
  replace: number;
  total_rows: number;
}

export interface DiffResult {
  left_path: string;
  right_path: string;
  left_bytes: number;
  right_bytes: number;
  warning: string | null;
  stats: DiffStats;
  rows: DiffRow[];
  change_indices: number[];
}

export function basename(path: string): string {
  const parts = path.split(/[/\\]/);
  return parts[parts.length - 1] || path;
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
}
