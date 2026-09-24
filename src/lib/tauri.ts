import { invoke } from "@tauri-apps/api/core";
import type { DiffResult } from "./types";

export async function pickFile(): Promise<string | null> {
  return invoke<string | null>("pick_file");
}

export async function computeDiff(
  leftPath: string,
  rightPath: string,
): Promise<DiffResult> {
  return invoke<DiffResult>("compute_diff", {
    leftPath,
    rightPath,
  });
}
