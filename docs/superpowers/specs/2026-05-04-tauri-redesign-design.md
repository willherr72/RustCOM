# RustCOM — Tauri Redesign Spec

**Date:** 2026-05-04
**Status:** Approved (brainstorming complete, awaiting plan)
**Scope:** Full rewrite of the desktop UI from egui to Tauri 2 + Svelte 5, with feature changes.

---

## 1. Goals

- Replace the egui UI with a Tauri-hosted Svelte 5 frontend so the app can have a polished, modern look that egui cannot deliver.
- Preserve the existing serial-port functionality that already works.
- Add multi-port tabs, embedded Lua scripting, send macros, in-buffer search, and command history.
- Drop the virtual-COM feature (unused, untested).
- Ship on Windows, Linux, and macOS.

## 2. Non-goals (v1)

- **Multi-tab Lua scripting** (`tabs[id]:send(...)`). The Lua API will be stubbed for it but unimplemented; v2 will fill it in.
- **Session restore** — open tabs at the time of close are not remembered. App opens to a blank slate every launch.
- **Persisted buffer contents** — buffers are in-memory only.
- **Persisted command history** — also in-memory only, cleared on close.
- **A new feature unrelated to the above** (e.g., logic analyzer, plotting). Out of scope.

## 3. Tech stack

| Layer | Choice | Reason |
|---|---|---|
| Backend | Rust + Tauri 2 | Keep all serial logic in Rust; Tauri gives us a webview without bundling a browser. |
| Serial | `serialport` 4.x | Already battle-tested in current code. |
| Scripting | Lua via `mlua` | De-facto language for instrument tooling; runs in a dedicated Rust thread. |
| Frontend framework | Svelte 5 + TypeScript + Vite (no SvelteKit) | Small bundle, ergonomic stores for streaming serial data, no SSR/routing needed. |
| Editor | Monaco | Lua syntax highlighting + good keybindings out of the box. |
| Theme | Custom "Zed Cool" — `#161719` near-black, lavender-blue accent (`#7a86c8`), Inter chrome, JetBrains Mono terminal | Approved during brainstorming. |

## 4. Layout

Activity-bar layout (Zed/VS Code style):

- **Title bar** (top, full-width): app name, connection-state badge, window controls.
- **Activity bar** (left, ~44 px): icon-only switcher with five entries — Connection, Macros, Scripts, Filter, Logs. Active entry shows the side panel; clicking the active icon collapses the panel for max terminal real estate.
- **Side panel** (~200 px when open): contents depend on active activity-bar entry.
- **Main area**: tab strip (one tab per open port, plus `+`), terminal view, send row (mode selector + line-ending + input + send).
- **Status bar** (bottom, full-width): connection pill, RX/TX byte counters, current settings summary.

## 5. Architecture

### 5.1 Backend modules (`src-tauri/src/`)

- `main.rs` — Tauri builder, command/event registration.
- `error.rs` — `AppError`, `pub type Result<T> = std::result::Result<T, AppError>`. All commands return this; serialized to the frontend.
- `port/`
  - `mod.rs` — `PortManager` owning `HashMap<TabId, PortHandle>`. `TabId` is a `u32` allocated by the frontend on tab creation.
  - `worker.rs` — `PortWorker`: spawned thread per port; runs the read loop, owns the `Box<dyn SerialPort>`, communicates over `mpsc` (Tx: write/set_dtr/set_rts/close commands; Rx is via Tauri events).
  - `config.rs` — `DataBits`, `StopBits`, `Parity`, `FlowControl`, `LineEnding`, `SendMode` (ported from current `serial.rs`), and a `PortConfig` struct that bundles them with `baud_rate`, `auto_reconnect: bool`, and `reconnect_delay_ms: u64`. Per-tab — every tab has its own `PortConfig`.
- `commands/`
  - `ports.rs` — `list_ports`, `open_port`, `close_port`, `send`, `send_hex`, `set_dtr`, `set_rts`.
  - `macros.rs` — `list_macros`, `save_macro`, `delete_macro`, `run_macro`.
  - `scripts.rs` — `list_scripts`, `save_script`, `delete_script`, `script_run`, `script_stop`.
  - `settings.rs` — `load_settings`, `save_settings`.
- `script/`
  - `mod.rs` — `ScriptEngine`: owns the Lua VM in a dedicated thread; commands via `mpsc`.
  - `api.rs` — Lua bindings: `serial.send(bytes)`, `serial.send_hex(str)`, `serial.send_text(str)`, `on_recv(fn)`, `on_disconnect(fn)`, `log(msg)`, `delay(ms)`, `ui.toast(msg, level?)`. Scripts run scoped to the **active tab at the time `script_run` was invoked**.
- `storage.rs` — Resolves OS-specific user-data dir (`%APPDATA%\rustcom\` on Windows, `~/.config/rustcom/` on Linux, `~/Library/Application Support/rustcom/` on macOS). Atomic JSON read/write.
- `hex.rs` — `format_hex`, `parse_hex_input`, `strip_ansi_codes` — ported verbatim from current code.

### 5.2 Frontend modules (`src/lib/`)

- `ipc.ts` — Typed wrappers over `@tauri-apps/api/core::invoke` and `event::listen`. One function per Tauri command.
- `stores/`
  - `tabs.ts` — `tabs: Writable<Tab[]>`, `activeTabId`, helpers to add/close/switch.
  - `ports.ts` — `availablePorts: Writable<PortInfo[]>`, polled every ~3 s via `list_ports`.
  - `macros.ts`, `scripts.ts`, `settings.ts` — load on app start, mutate via commands, mirror locally.
- `components/`
  - `ActivityBar.svelte`, `TitleBar.svelte`, `StatusBar.svelte`, `TabStrip.svelte`.
  - `Terminal.svelte` — virtualized list of lines (use a small windowing helper or a vetted lib). Auto-scroll-on-tail unless user scrolls up. Renders ASCII/Hex/Both per the active tab's `viewMode`.
  - `SendRow.svelte` — mode selector (ASCII/Hex), line-ending picker (ASCII only), command-history walk on Up/Down, Enter to send.
  - `SearchOverlay.svelte` — Ctrl+F triggered, regex toggle, next/prev match, highlights matches inline.
  - `panels/` — `ConnectionPanel`, `MacrosPanel`, `ScriptsPanel` (Monaco editor + run/stop), `FilterPanel`, `LogsPanel`.
- `theme.css` — CSS custom properties for the Zed Cool palette so they can be referenced everywhere.

### 5.3 Threading model

- One thread per open port (the `PortWorker`). Reads with a 10 ms timeout (matches current behavior); pushes bytes to the frontend via `app_handle.emit("rx://<tab_id>", payload)`.
- One thread for the Lua engine. The frontend calls `script_run` and `script_stop`; `on_recv` callbacks fire on this thread, which then dispatches `serial.send(...)` calls to the relevant `PortWorker` over its mpsc.
- The Tauri main thread / UI runtime owns no serial I/O — it only routes commands.

## 6. Data flow (key paths)

**Open a port:**
1. User clicks `+` in the tab strip → frontend allocates a new `TabId` and pushes a transient tab in `disconnected` state.
2. User fills connection settings in the panel → clicks Connect.
3. Frontend calls `invoke('open_port', { tabId, config })`.
4. `PortManager.open` spawns a `PortWorker` and stores the handle.
5. Worker enters its read loop. On each non-empty read, it emits `rx://<tabId>` with `{ bytes, timestamp }`.
6. `Terminal.svelte` for the active tab listens, appends to that tab's display buffer.

**Send data:**
- `invoke('send' | 'send_hex', { tabId, payload, lineEnding? })` → `PortManager` routes to the right worker over mpsc → worker writes → backend emits `tx://<tabId>` so the terminal can echo. (Echo is a UI affordance, not a real loopback.)

**Streaming buffer cap:**
- Each tab's display buffer in the Svelte store keeps the same drain rule as today: `MAX_BUFFER_SIZE = 100_000`, drain `BUFFER_DRAIN_SIZE = 10_000` from the head when exceeded. (Counters for total RX/TX bytes are independent and never reset by the drain.)

**Run a Lua script:**
- `invoke('script_run', { tabId, source })` → engine compiles and executes; any `on_recv` callback registered fires on subsequent `rx://<tabId>` events for *this* tab.
- The bound `tabId` is captured at `script_run` time; switching tabs in the UI afterwards does **not** retarget the script. Closing the bound tab implicitly stops the script.
- A second `script_run` replaces the previous registration; `script_stop` clears it.
- `delay(ms)` sleeps the script thread (the UI is unaffected).
- `ui.toast(msg)` emits a `toast` event back to the frontend.

**Command history:**
- Per-tab ring buffer (~200 entries) held in `tabs.ts`. Up/Down in the send input walks it. Cleared when the tab is closed; not persisted.

**Search:**
- Ctrl+F overlay scoped to the active tab. Regex toggle. Next/prev cycles matches; matches are highlighted in the rendered terminal.

## 7. Persistence

User-data dir (`storage.rs` resolves it):
- `settings.json` — last-used connection defaults, theme prefs, panel pin state.
- `macros.json` — `[ { id, name, mode: 'ascii'|'hex', payload, line_ending, hotkey? } ]`.
- `scripts/<name>.lua` — one file per saved script.

Not persisted: open tabs, buffers, command history.

## 8. Errors & UX

- All Tauri commands return `Result<T, AppError>`. On error, frontend shows a non-modal toast in the title-bar/status-bar region. No `unwrap` or `expect` in the backend.
- Connection loss continues to surface as a toast and an inline log line in the affected tab; the auto-reconnect option is preserved (per-tab setting).
- Lua runtime errors land in the Logs panel for the active tab, with line/column info.

## 9. Migration plan

Done as one large PR (the old build remains on `main` until merged):

1. Move existing `src/*.rs` → `src-tauri/src/legacy/` (kept as reference, excluded from `Cargo.toml`).
2. Scaffold Tauri 2 around the existing repo — preserve git history, `README.md`, `icon.ico`, `build.rs`.
3. Reimplement serial logic in the new structure (mostly a verbatim port — the I/O code was solid; the UI was the problem).
4. Drop `virtual_com.rs`.
5. Build the Svelte frontend matching the approved mockups and theme.
6. Once green on all three platforms, delete `src-tauri/src/legacy/`.

## 10. Acceptance criteria

A v1 release is done when:

- The app builds and runs on Windows, Linux, and macOS.
- Two or more ports can be opened in tabs simultaneously and operate independently.
- ASCII/Hex/Both views render correctly; the new hex view matches the current formatter byte-for-byte.
- Macros can be created, edited, deleted, hot-keyed (F1–F12 by default), and triggered against the active tab.
- A Lua script using `serial.send`, `on_recv`, `log`, `delay`, and `ui.toast` runs against the active tab without UI stutter.
- Ctrl+F search finds and highlights regex matches in the active terminal.
- Up/Down arrows in the send input walk per-tab command history.
- Connection settings, macros, and scripts persist across restarts; tabs and buffers do not.
- The `virtual_com` module is gone; no reference to it in the code or README.
- The README is updated to document Tauri build prerequisites and the new feature set.

## 11. Open questions (intentionally deferred)

- Plot/graph view of numeric RX data — out of scope for v1.
- File transfer (XMODEM/YMODEM) — out of scope.
- Multi-tab Lua scripting — explicitly v2.
- Session restore — explicitly v2 if requested later.
