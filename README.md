# Diff_Local

Utility desktop per visualizzare il **diff tra due file di testo**, anche mediamente grandi (decine di migliaia di righe / fino a ~100 MiB).

Stack: **Tauri 2** + **React** + **TypeScript**. Il motore di diff (Myers, crate `similar`) gira in Rust; la UI usa scroll virtualizzato.

## Funzionalità

- Apertura file tramite dialog nativo
- Vista **affiancata** (side-by-side) o **unificata**
- Scroll virtualizzato e navigazione tra modifiche (`N` / `P`, oppure `]` / `[`)
- Conteggi insert / delete / replace
- Limite soft a 50 MiB (avviso) e hard a 100 MiB

## Prerequisiti

- [Node.js](https://nodejs.org/) 20+ (consigliato 20.19+ o 22 LTS)
- [Rust](https://www.rust-lang.org/tools/install) (stable, consigliato ≥ 1.85)
- Dipendenze di sistema Tauri per il tuo OS: [prerequisites](https://tauri.app/start/prerequisites/)

Su Debian/Ubuntu tipicamente:

```bash
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf libgtk-3-dev
```

## Avvio in sviluppo

```bash
npm install
npm run tauri dev
```

## Build di produzione (locale)

Sullo stesso OS della macchina:

```bash
npm ci
npm run tauri:build
# oppure
./scripts/build-local.sh
```

Su **Windows** ottieni:
- `src-tauri/target/release/Diff_Local.exe`
- installer NSIS `src-tauri/target/release/bundle/nsis/*-setup.exe`

Il frontend da solo: `npm run build`.

## Scaricare / pubblicare l'exe da GitHub

L'exe Windows **non** viene committato nel tree (è troppo grande): viene prodotto da GitHub Actions e pubblicato nelle **Releases** / **Artifacts**.

### Build automatica (ogni push)

Il workflow [`.github/workflows/build.yml`](.github/workflows/build.yml) compila Windows + Linux e carica gli artifact.

1. Apri **Actions → Build** (o aspetta il run sul push)
2. Apri il run → **Artifacts** → `Diff_Local-windows-x64`
3. Oppure da CLI: `npm run release:download`

### Pubblicare una release con l'exe

```bash
# bump versione (opzionale), commit, tag vX.Y.Z e push → avvia Release
./scripts/tag-release.sh 0.1.0

# oppure avvia solo il workflow senza nuovo tag
npm run release:dispatch
```

Il workflow [`.github/workflows/release.yml`](.github/workflows/release.yml) crea una GitHub Release con `Diff_Local_*_x64-setup.exe` (e gli altri OS).

> In **Settings → Actions → General → Workflow permissions** abilita *Read and write permissions* (serve a `tauri-action` per creare la release).

### Branch `binaries` (exe dentro Git)

Il workflow [`.github/workflows/publish-binaries.yml`](.github/workflows/publish-binaries.yml) committa l'installer sul branch dedicato `binaries` (non su `main`):

```bash
gh workflow run publish-binaries.yml
# oppure dopo un tag v*
git fetch origin binaries
git checkout binaries
ls windows/
```

## Test

```bash
npm run test:rust
```

## Uso

1. Scegli il file **Sinistra** e il file **Destra**
2. Premi **Confronta** (oppure `Ctrl`+`Enter`)
3. Alterna Affiancata / Unificata e scorri le modifiche con i pulsanti o le scorciatoie

## Limiti (v1)

- Solo file di testo (non cartelle, binari o immagini)
- Nessun editing / merge a tre vie / integrazione Git
