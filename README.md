# RustCOM — COM Port Analyzer

A serial-port analyzer built with Tauri 2 + Svelte 5. Cross-platform desktop binary, native serial I/O, modern UI.

## Status

- **Plan 1 (current)** — single-port end-to-end: connect, RX streaming, ASCII/Hex/Both view, ANSI strip, ASCII/Hex send with line endings, DTR/RTS, command history.
- **Plan 2 (next)** — multi-port tabs, send macros, regex filter, logging, in-buffer search.
- **Plan 3 (after)** — Lua scripting (mlua + Monaco editor), polish, packaging, plot view roadmap.

## Download

Pre-built executables ship from the [Releases](../../releases) page.

## Features (current)

- One serial port at a time (multi-port arrives in Plan 2).
- ASCII / Hex / Both views with ANSI escape stripping.
- ASCII send with selectable line endings (None, `\r`, `\n`, `\r\n`).
- Hex send (`AA BB 0D 0A` style).
- DTR / RTS line control.
- Command history (Up/Down in the send box).
- Auto-scrolling terminal with manual clear.
- Live RX/TX byte counters.

## Connection settings

- **Baud rate**: 300 to 921600
- **Data bits**: 5, 6, 7, 8
- **Stop bits**: 1, 2
- **Parity**: None, Even, Odd
- **Flow control**: None, Software (XON/XOFF), Hardware (RTS/CTS)

## Building from source

### Prerequisites

- [Rust](https://rustup.rs/) 1.77+
- [Node.js](https://nodejs.org/) 20+
- Tauri CLI 2: `cargo install tauri-cli --version "^2.0.0" --locked`

Per-platform:

- **Windows**: [Build Tools for Visual Studio 2022](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022) with "Desktop development with C++". WebView2 is preinstalled on Windows 11.
- **Linux**: `webkit2gtk-4.1`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`. On Ubuntu/Debian:
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
  ```
- **macOS**: Xcode Command Line Tools.

### Run in dev

```bash
npm install
cargo tauri dev
```

### Build a release binary

```bash
cargo tauri build
```

Artifacts land in `src-tauri/target/release/bundle/`.

## Project layout

```
RustCOM/
├── src-tauri/        Rust backend (Tauri commands, serial workers)
├── src/              Svelte 5 frontend
├── docs/             Specs and plans
└── legacy-egui/      Previous egui implementation, kept until Plan 3 finishes
```

## License

See LICENSE.
