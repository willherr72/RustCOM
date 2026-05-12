# RustCOM — COM Port Analyzer

A multi-port serial-port analyzer with embedded Lua scripting, built with Tauri 2 and Svelte 5.

## Features

- **Multi-port tabs** — open many COM ports at once, each with its own buffer, settings, and history.
- **Three views** — ASCII, Hex (16-byte rows with offset + ASCII gutter), or Both. ANSI escape stripping.
- **Send modes** — ASCII with selectable line endings (`None`, `\r`, `\n`, `\r\n`) or raw hex (`AA BB 0D 0A`).
- **Send macros** — named one-shot snippets, optional F1–F12 hotkeys, persisted to disk.
- **Lua scripting** — write scripts in a Monaco editor, run them against the active tab; API includes `serial.send`, `serial.send_text`, `serial.send_hex`, `on_recv`, `log`, `delay`, `ui.toast`.
- **Live plot view** — define a regex with named capture groups (e.g. `temp:(?<temp>\d+\.\d+)`) and watch RX values render as a streaming line chart. Pause / Resume / Save CSV.
- **Multi-tab Lua** — `tabs.send(id, ...)`, `tabs.send_text(id, "AT")`, `tabs.on_recv(id, fn)` so one script can drive several ports.
- **Session restore** — opt-in: open tabs are remembered across restarts (disconnected on relaunch).
- **Display-time regex filter** per tab (preserves the underlying buffer).
- **Per-tab logging** with native save dialog.
- **Ctrl+F search** with regex toggle and inline highlights.
- **Auto-reconnect** per tab.
- **Connection settings** — baud 300–921600, data bits 5/6/7/8, stop 1/2, parity None/Even/Odd, flow None/SW/HW, DTR/RTS toggles.

## Download

Pre-built executables ship from the [Releases](../../releases) page.

## Building from source

### Prerequisites

- [Rust](https://rustup.rs/) 1.77+
- [Node.js](https://nodejs.org/) 20+
- Tauri CLI 2: `cargo install tauri-cli --version "^2.0.0" --locked`

Per-platform:

- **Windows** — Build Tools for Visual Studio 2022 with "Desktop development with C++". WebView2 is preinstalled on Windows 11.
- **Linux** — `sudo apt install libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`
- **macOS** — Xcode Command Line Tools.

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
├── src-tauri/        Rust backend (Tauri commands, serial workers, Lua engine)
├── src/              Svelte 5 frontend
└── docs/             Specs and plans
```

## Persistence

Settings, macros, and scripts live in your OS user-data dir:

- Windows: `%LOCALAPPDATA%\rustcom\`
- macOS: `~/Library/Application Support/rustcom/`
- Linux: `~/.local/share/rustcom/`

## License

MIT — see LICENSE.
