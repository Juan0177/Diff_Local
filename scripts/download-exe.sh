#!/usr/bin/env bash
# Scarica l'ultimo artifact Windows (setup.exe / Diff_Local.exe) da GitHub Actions.
#
# Uso:
#   ./scripts/download-exe.sh
#   ./scripts/download-exe.sh ./dist-bin
#
# Prerequisiti: gh autenticato.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$ROOT/dist-bin}"

command -v gh >/dev/null || { echo "Serve GitHub CLI (gh)" >&2; exit 1; }

mkdir -p "$OUT"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Scarico artifact Diff_Local-windows-x64…"
gh run download --name Diff_Local-windows-x64 --dir "$TMP" || {
  echo "Nessun artifact trovato. Avvia prima il workflow Build:" >&2
  echo "  gh workflow run build.yml" >&2
  exit 1
}

cp -v "$TMP"/* "$OUT"/
echo
echo "File in $OUT:"
ls -lah "$OUT"
