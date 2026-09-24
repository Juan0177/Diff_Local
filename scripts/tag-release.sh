#!/usr/bin/env bash
# Pubblica una release con l'exe Windows (e gli altri binari) su GitHub.
#
# Uso:
#   ./scripts/tag-release.sh              # usa la versione in tauri.conf.json
#   ./scripts/tag-release.sh 0.1.1        # imposta versione e tagga
#   ./scripts/tag-release.sh --dispatch   # avvia solo il workflow senza nuovo tag
#
# Prerequisiti: git, gh (autenticato), working tree pulito.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CONF="src-tauri/tauri.conf.json"
PKG="package.json"
CARGO="src-tauri/Cargo.toml"

die() { echo "Errore: $*" >&2; exit 1; }

command -v git >/dev/null || die "serve git"
command -v gh >/dev/null || die "serve GitHub CLI (gh)"

if [[ "${1:-}" == "--dispatch" ]]; then
  echo "Avvio workflow Release (manual)…"
  gh workflow run release.yml --ref "$(git branch --show-current)"
  echo "Apri: $(gh repo view --json url -q .url)/actions"
  exit 0
fi

[[ -z "$(git status --porcelain)" ]] || die "working tree non pulito; committa o stash prima"

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
  VERSION="$(python3 - <<'PY'
import json
print(json.load(open("src-tauri/tauri.conf.json"))["version"])
PY
)"
fi

[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-].+)?$ ]] || die "versione non valida: $VERSION"

TAG="v${VERSION}"
if git rev-parse "$TAG" >/dev/null 2>&1; then
  die "il tag $TAG esiste già"
fi

echo "Aggiorno versione a $VERSION…"
python3 - <<PY
import json
from pathlib import Path

version = "$VERSION"

conf_path = Path("$CONF")
conf = json.loads(conf_path.read_text())
conf["version"] = version
conf_path.write_text(json.dumps(conf, indent=2) + "\n")

pkg_path = Path("$PKG")
pkg = json.loads(pkg_path.read_text())
pkg["version"] = version
pkg_path.write_text(json.dumps(pkg, indent=2) + "\n")

cargo = Path("$CARGO").read_text().splitlines()
out = []
replaced = False
for line in cargo:
    if line.startswith("version = ") and not replaced:
        out.append(f'version = "{version}"')
        replaced = True
    else:
        out.append(line)
Path("$CARGO").write_text("\n".join(out) + "\n")
PY

git add "$CONF" "$PKG" "$CARGO"
if ! git diff --cached --quiet; then
  git commit -m "chore: bump version to ${VERSION}"
fi

BRANCH="$(git branch --show-current)"
echo "Push branch $BRANCH…"
git push -u origin "$BRANCH"

echo "Creo e pusho tag $TAG (avvia il workflow Release)…"
git tag -a "$TAG" -m "Diff_Local ${VERSION}"
git push origin "$TAG"

echo
echo "OK. Workflow in corso:"
echo "  $(gh repo view --json url -q .url)/actions"
echo "Quando finisce, l'exe sarà in:"
echo "  $(gh repo view --json url -q .url)/releases/tag/${TAG}"
