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

- [Node.js](https://nodejs.org/) 18+
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

## Build di produzione

```bash
npm run tauri build
```

Il frontend può essere compilato da solo con:

```bash
npm run build
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
