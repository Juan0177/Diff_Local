# Build locale di produzione (sullo stesso OS dove lanci lo script).
# Su Windows produce Diff_Local.exe e l'installer *-setup.exe.
#
# Uso (bash / Git Bash / WSL con toolchain Windows non supportato per .exe nativo):
#   ./scripts/build-local.sh
#
# Su Windows PowerShell preferisci:
#   npm ci
#   npm run tauri build

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

command -v npm >/dev/null || { echo "Serve Node/npm" >&2; exit 1; }
command -v cargo >/dev/null || { echo "Serve Rust/cargo" >&2; exit 1; }

if [[ ! -d node_modules ]]; then
  npm ci
fi

npm run tauri build

echo
echo "Build completata. Bundle in:"
echo "  src-tauri/target/release/bundle/"
find src-tauri/target/release/bundle -type f \( -name '*.exe' -o -name '*.msi' -o -name '*.AppImage' -o -name '*.deb' -o -name '*.dmg' \) 2>/dev/null | sed 's/^/  /' || true

if [[ -f src-tauri/target/release/Diff_Local.exe ]]; then
  echo "  src-tauri/target/release/Diff_Local.exe"
elif [[ -f src-tauri/target/release/diff-local ]]; then
  echo "  src-tauri/target/release/diff-local"
fi
