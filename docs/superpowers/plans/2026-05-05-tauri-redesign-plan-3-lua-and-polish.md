# RustCOM Tauri Redesign — Plan 3: Lua Scripting + Settings Persistence + Polish

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land Lua scripting (mlua + Monaco editor + Scripts panel), persist user settings/auto-reconnect, clean up the polish backlog from Plans 1 & 2 reviews, and ship a clean v1 release.

**Architecture:** A `ScriptEngine` thread on the backend owns the Lua VM and exposes `serial.send` / `on_recv` / `delay` / `log` / `ui.toast` to scripts; communication with `PortManager` and the frontend is via `mpsc` and Tauri events. Settings persist in `settings.json` next to `macros.json` using the existing `storage` module. Auto-reconnect runs as a small per-tab state machine inside `PortWorker`. Polish: rename crate `app` → `rustcom`, wire `tauri-plugin-log`, tighten CSP, fix a11y warnings, reformat MacrosPanel, drop `legacy-egui/`.

**Tech Stack:** Rust + Tauri 2 + `mlua` (vendored Lua 5.4) for scripting; Svelte 5 + TypeScript + Vite + Monaco editor (`monaco-editor` + `@monaco-editor/loader` or `vite-plugin-monaco-editor`).

**Spec:** `docs/superpowers/specs/2026-05-04-tauri-redesign-design.md`
**Predecessor plans:** Plan 1 (`...plan-1-foundations.md`), Plan 2 (`...plan-2-multitab-and-panels.md`).

**Working branch:** continue on `tauri-redesign`. Plan 3 ends with merging `tauri-redesign` → `main`, tagging `v0.2.0`, and producing release artifacts.

**Working directory:** `C:\Users\WilliamHerr\Desktop\Code\RustCOM`.

**Testing philosophy:** Same as Plans 1–2. Pure Rust logic (Lua bindings dispatch, settings serde) gets unit tests. The Lua VM itself we exercise with small `cargo test` fixtures (no real serial port, just verify `serial.send(bytes)` ends up forwarded to a mock). UI is verified by running.

---

## Plan 3 task overview

| # | Task | Verification |
|---|---|---|
| 1 | Cleanup: rename Cargo crate `app` → `rustcom`, fix `tauri.conf.json` metadata | `cargo build` clean, lib name updated |
| 2 | Cleanup: minimal CSP in `tauri.conf.json` | App still loads |
| 3 | Cleanup: wire `tauri-plugin-log` + capability | Log lines appear in webview console |
| 4 | Cleanup: a11y label associations on ConnectionPanel + FilterPanel + LogsPanel | `npm run check` warnings drop to 0 |
| 5 | Cleanup: extract `MacroEditor.svelte` snippet from MacrosPanel duplication | Same UX, half the markup |
| 6 | Backend `settings.rs` (settings.json schema + read/save commands) | `cargo test` |
| 7 | Frontend `stores/settings.ts` + load on app start | Defaults apply on first run, persist after edit |
| 8 | Auto-reconnect state machine in `PortWorker` | Manual: pull USB, port reopens after delay |
| 9 | Backend `mlua` dependency + `script/api.rs` skeleton + tests | `cargo test --lib script` |
| 10 | Backend `script/mod.rs` `ScriptEngine` thread + `commands/scripts.rs` | `cargo test`, commands callable |
| 11 | Frontend `stores/scripts.ts` + ipc wrappers | Vitest |
| 12 | Monaco package + `lib/monaco.ts` loader (with worker config) | `npm run build` clean |
| 13 | `ScriptsPanel.svelte` (script list + Monaco editor + Run/Stop/Save/Delete) | Manual: write Lua, run it |
| 14 | Toast surface (`stores/toasts.ts` + `ToastStack.svelte`) for `ui.toast` | Manual: `ui.toast("hi")` shows |
| 15 | `script_log` event surfaced in Logs panel for the active tab | Manual: `log("X")` appears in panel |
| 16 | Drop `legacy-egui/` and update README to v0.2.0 + features list | `git status` clean, README current |
| 17 | Bump version → 0.2.0 in `Cargo.toml`, `tauri.conf.json`, `package.json` | `cargo tauri build` produces v0.2.0 artifacts |
| 18 | Final smoke + tag `v0.2.0` + merge `tauri-redesign` → `main` | Tag pushed locally; clean git log |

---

## Task 1: Rename crate `app` → `rustcom`

**Files:**
- Modify: `src-tauri/Cargo.toml`

The crate-init left placeholders that surface in built binaries (`app.exe`, "A Tauri App").

- [ ] **Step 1: Edit `src-tauri/Cargo.toml`**

Change the `[package]` and `[lib]` blocks at the top:

```toml
[package]
name = "rustcom"
version = "0.1.0"
description = "RustCOM serial-port analyzer"
authors = ["RustCOM"]
license = "MIT"
repository = ""
edition = "2021"
rust-version = "1.77.2"

[lib]
name = "rustcom_lib"
crate-type = ["staticlib", "cdylib", "rlib"]
```

Leave the `[build-dependencies]` and `[dependencies]` blocks unchanged.

- [ ] **Step 2: Update `main.rs` to call the new lib name**

Open `src-tauri/src/main.rs` — it currently calls `app_lib::run()`. Change to:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    rustcom_lib::run();
}
```

- [ ] **Step 3: Build**

```bash
cd src-tauri
cargo build
```

Expected: clean, no errors. Built binary will be at `target/debug/rustcom.exe` instead of `app.exe`.

- [ ] **Step 4: Commit**

```bash
cd ..
git add src-tauri/Cargo.toml src-tauri/src/main.rs
git commit -m "chore(backend): rename crate app -> rustcom"
```

---

## Task 2: Minimal CSP

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Replace `"csp": null`**

In the `app.security` block:

```json
"security": {
  "csp": "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; font-src 'self' data:; connect-src 'self' ipc: http://ipc.localhost"
}
```

The relaxations:
- `'unsafe-inline'` for styles is required for Svelte's component-level CSS injection.
- `data:` for images and fonts is needed because Monaco's bundled fonts/icons load as data URIs.
- `connect-src` includes `ipc:` and `http://ipc.localhost` (Tauri 2's IPC origins on Windows).

Future Lua/script work won't loosen this — Lua runs server-side, not in the webview.

- [ ] **Step 2: Run dev**

```bash
cargo tauri dev
```

App should boot normally. Open the WebView2 devtools (F12) and confirm no CSP violations are logged. If any appear, copy the missing source into the relevant CSP directive.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "chore(security): add minimal CSP"
```

---

## Task 3: Wire `tauri-plugin-log`

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/capabilities/default.json`

The `tauri-plugin-log` crate is in `Cargo.toml` but never registered. Wire it so backend `log::info!` / `error!` messages can flow to the webview console (and a rolling log file).

- [ ] **Step 1: Register the plugin in lib.rs**

In `src-tauri/src/lib.rs`, find the `tauri::Builder::default()` block and add `.plugin(tauri_plugin_log::Builder::new().build())` right before `.plugin(tauri_plugin_dialog::init())`:

```rust
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(PortManager::new())
        .invoke_handler(tauri::generate_handler![
            // ... unchanged
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
```

- [ ] **Step 2: Allow the log plugin in capabilities**

Edit `src-tauri/capabilities/default.json`. Add `"log:default"` to the `permissions` list:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:event:allow-listen",
    "core:event:allow-unlisten",
    "dialog:default",
    "log:default"
  ]
}
```

- [ ] **Step 3: Add a smoke log to verify wiring**

In `src-tauri/src/lib.rs`, just inside `pub fn run()` (before `tauri::Builder`):

```rust
pub fn run() {
    log::info!("RustCOM starting");
    tauri::Builder::default()
```

- [ ] **Step 4: Build and run**

```bash
cd src-tauri
cargo build
cd ..
cargo tauri dev
```

In the dev window, open devtools (F12). The console should show the `RustCOM starting` line.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/capabilities/default.json
git commit -m "chore(backend): wire tauri-plugin-log"
```

---

## Task 4: Fix a11y label associations

**Files:**
- Modify: `src/lib/components/panels/ConnectionPanel.svelte`
- Modify: `src/lib/components/panels/FilterPanel.svelte`
- Modify: `src/lib/components/panels/LogsPanel.svelte` (if any warnings remain)

Wrap the decorative `<label class="lbl">` elements around their selects/inputs so screen readers and svelte-check both stop complaining.

- [ ] **Step 1: ConnectionPanel — wrap each label+select pair**

For each of the six occurrences in `src/lib/components/panels/ConnectionPanel.svelte`:

Old shape:
```svelte
<label class="lbl">Port</label>
<select ...>...</select>
```

New shape (one label wraps both the text and the control):
```svelte
<label class="lbl">
  Port
  <select ...>...</select>
</label>
```

Apply this transform to all six fields: Port, Baud, Data, Stop, Parity, Flow.

The Data and Stop selects are inside `.row2 > div`. Wrap them like:

```svelte
<div>
  <label class="lbl">
    Data
    <select ...>...</select>
  </label>
</div>
```

- [ ] **Step 2: Update CSS to keep visual layout**

Add to the existing `.lbl` style in ConnectionPanel.svelte:

```css
.lbl {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
```

Remove the `margin-top: 4px` from `.lbl` if present (the parent `gap: 6px` handles spacing now).

- [ ] **Step 3: Same fix in FilterPanel.svelte**

The `<label class="lbl">Pattern</label>` followed by `<input ...>` becomes:

```svelte
<label class="lbl">
  Pattern
  <input
    placeholder="e.g. ^READY|^ERROR"
    value={$activeTab.filterPattern}
    oninput={(e) => patchActiveTab({ filterPattern: (e.currentTarget as HTMLInputElement).value })}
    disabled={!$activeTab.filterEnabled}
  />
</label>
```

Same `.lbl { display: flex; flex-direction: column; gap: 4px; }` rule.

Also remove the unused `input[type="text"]` selector from FilterPanel's `<style>` block (svelte-check warned about it):

Replace:
```css
input[type="text"], input:not([type]) { width: 100%; font-family: var(--font-mono); }
```

With:
```css
input { width: 100%; font-family: var(--font-mono); }
```

- [ ] **Step 4: Run `npm run check`**

```bash
npm run check
```

Expected: 0 errors, 0 warnings. If a `SearchOverlay.svelte` `autofocus` warning remains, suppress it inline:

```svelte
<!-- svelte-ignore a11y_autofocus -->
<input
  autofocus
  ...
/>
```

(The warning is intentional — Ctrl+F overlay should grab focus immediately.)

- [ ] **Step 5: Build**

```bash
npm run build
```

Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add src/lib/components/panels/ConnectionPanel.svelte src/lib/components/panels/FilterPanel.svelte src/lib/components/SearchOverlay.svelte
git commit -m "chore(ui): associate decorative labels with their controls"
```

---

## Task 5: Extract `MacroEditor.svelte` snippet

**Files:**
- Create: `src/lib/components/panels/MacroEditor.svelte`
- Modify: `src/lib/components/panels/MacrosPanel.svelte`

The macros panel renders the same edit form twice (existing edit + new draft). Extract to one component.

- [ ] **Step 1: Write MacroEditor.svelte**

`src/lib/components/panels/MacroEditor.svelte`:

```svelte
<script lang="ts">
  import { type LineEnding, type Macro } from "$lib/ipc";

  let {
    draft = $bindable<Macro>(),
    saving = false,
    panelError = null,
    onSave,
    onCancel,
    onDelete = null,
  }: {
    draft: Macro;
    saving?: boolean;
    panelError?: string | null;
    onSave: () => void;
    onCancel: () => void;
    onDelete?: (() => void) | null;
  } = $props();

  const HOTKEYS = ["", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12"];
</script>

<div class="edit">
  <input placeholder="Name" bind:value={draft.name} />
  <div class="row">
    <select bind:value={draft.mode}>
      <option value="ascii">ASCII</option>
      <option value="hex">Hex</option>
    </select>
    {#if draft.mode === "ascii"}
      <select
        value={draft.line_ending ?? "none"}
        onchange={(e) => (draft.line_ending = (e.currentTarget as HTMLSelectElement).value as LineEnding)}
      >
        <option value="none">None</option>
        <option value="cr">\r</option>
        <option value="lf">\n</option>
        <option value="crlf">\r\n</option>
      </select>
    {/if}
    <select
      value={draft.hotkey ?? ""}
      onchange={(e) => (draft.hotkey = (e.currentTarget as HTMLSelectElement).value || null)}
    >
      {#each HOTKEYS as k}
        <option value={k}>{k || "no hotkey"}</option>
      {/each}
    </select>
  </div>
  <textarea
    rows="2"
    bind:value={draft.payload}
    placeholder={draft.mode === "ascii" ? "Message…" : "AA BB 0D 0A"}
  ></textarea>
  {#if panelError}<p class="err">{panelError}</p>{/if}
  <div class="row gap">
    <button class="primary" onclick={onSave} disabled={saving}>{saving ? "Saving…" : "Save"}</button>
    <button onclick={onCancel}>Cancel</button>
    {#if onDelete}<button class="danger" onclick={onDelete}>Delete</button>{/if}
  </div>
</div>

<style>
  .edit { display: flex; flex-direction: column; gap: 6px; padding: 8px; background: var(--bg-input); border-radius: var(--radius-md); border: 1px solid var(--accent); }
  .edit input, .edit select, .edit textarea { font-family: var(--font-mono); font-size: 11px; }
  .edit textarea { resize: vertical; min-height: 36px; padding: 6px 8px; border-radius: var(--radius-md); background: var(--bg); color: var(--fg); border: 1px solid transparent; }
  .edit textarea:focus { border-color: var(--accent); outline: none; }
  .row { display: flex; gap: 4px; min-width: 0; }
  .row.gap { gap: 6px; flex-wrap: wrap; }
  .row select { flex: 1 1 0; min-width: 0; }
  .edit input, .edit textarea { box-sizing: border-box; width: 100%; }
  .err { color: var(--err); font-size: 11px; margin: 0; }
</style>
```

- [ ] **Step 2: Refactor MacrosPanel.svelte to use it**

Replace the entire `{#each $macros}` block AND the trailing new-draft block with this single block. Inside `MacrosPanel.svelte`, after the script tag, the markup becomes:

```svelte
<div class="panel">
  <div class="header">
    <h4>Macros</h4>
    <button class="add" onclick={startNew}>+ New</button>
  </div>

  {#if $macros.length === 0 && !editingId}
    <p class="empty">No macros yet. Click <em>+ New</em> to add one.</p>
  {/if}

  {#each $macros as m (m.id)}
    {#if editingId === m.id}
      <MacroEditor
        bind:draft
        {saving}
        {panelError}
        onSave={save}
        onCancel={() => { editingId = null; panelError = null; }}
        onDelete={() => del(m.id)}
      />
    {:else}
      <div class="item">
        <div class="meta">
          <strong>{m.name || "(unnamed)"}</strong>
          <span class="kind">{m.mode}{m.hotkey ? ` · ${m.hotkey}` : ""}</span>
        </div>
        <div class="actions">
          <button onclick={() => run(m)}>Run</button>
          <button onclick={() => startEdit(m)}>Edit</button>
        </div>
      </div>
    {/if}
  {/each}

  {#if editingId && !$macros.some((m) => m.id === editingId)}
    <MacroEditor
      bind:draft
      {saving}
      {panelError}
      onSave={save}
      onCancel={() => { editingId = null; panelError = null; }}
    />
  {/if}
</div>
```

Add the import at the top of the script block:

```typescript
import MacroEditor from "$lib/components/panels/MacroEditor.svelte";
```

Remove the `HOTKEYS` const from MacrosPanel.svelte (now it lives in MacroEditor only).

Remove the now-unused styles from MacrosPanel's `<style>` block (everything that was scoped to `.edit`, `.row`, `.err`, plus the `.edit input/textarea/select` rules — they all live in MacroEditor now). Keep:

```css
.panel { padding: 12px; height: 100%; box-sizing: border-box; display: flex; flex-direction: column; gap: 8px; overflow: auto; }
.header { display: flex; align-items: center; justify-content: space-between; }
h4 { font-size: 10px; letter-spacing: 0.1em; text-transform: uppercase; color: var(--fg-subtle); margin: 0; font-weight: 600; }
.add { padding: 4px 8px; font-size: 10px; }
.empty { color: var(--fg-subtle); font-size: 11px; }

.item { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 6px 8px; background: var(--bg-input); border-radius: var(--radius-md); }
.meta { display: flex; flex-direction: column; min-width: 0; font-size: 11px; }
.meta strong { color: var(--fg); font-weight: 500; }
.kind { color: var(--fg-subtle); font-size: 9px; text-transform: uppercase; letter-spacing: 0.06em; }
.actions { display: flex; gap: 4px; }
.actions button { padding: 4px 8px; font-size: 10px; }
```

- [ ] **Step 3: Type check + build**

```bash
npm run check
npm run build
```

Both pass.

- [ ] **Step 4: Smoke**

```bash
cargo tauri dev
```

Open Macros panel. Create a macro, save, edit, delete — same UX as before.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/panels/MacroEditor.svelte src/lib/components/panels/MacrosPanel.svelte
git commit -m "refactor(ui): extract MacroEditor from MacrosPanel duplication"
```

---

## Task 6: Backend `settings.rs` (settings.json)

**Files:**
- Create: `src-tauri/src/commands/settings.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

`settings.json` holds last-used connection defaults so a fresh tab opens with sensible values instead of `9600 8-N-1`.

- [ ] **Step 1: Write the module**

`src-tauri/src/commands/settings.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::port::config::{DataBits, FlowControl, LineEnding, Parity, SendMode, StopBits};
use crate::storage;

const FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    pub baud_rate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
    pub flow_control: FlowControl,
    pub line_ending: LineEnding,
    pub send_mode: SendMode,
    pub auto_reconnect: bool,
    pub reconnect_delay_ms: u64,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            baud_rate: 9600,
            data_bits: DataBits::Eight,
            stop_bits: StopBits::One,
            parity: Parity::None,
            flow_control: FlowControl::None,
            line_ending: LineEnding::CrLf,
            send_mode: SendMode::Ascii,
            auto_reconnect: false,
            reconnect_delay_ms: 2000,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub defaults: DefaultsConfig,
}

impl Default for Settings {
    fn default() -> Self {
        Self { defaults: DefaultsConfig::default() }
    }
}

#[tauri::command]
pub fn load_settings() -> AppResult<Settings> {
    storage::read_json(FILE)
}

#[tauri::command]
pub fn save_settings(settings: Settings) -> AppResult<Settings> {
    storage::write_json(FILE, &settings)?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_have_sensible_values() {
        let d = DefaultsConfig::default();
        assert_eq!(d.baud_rate, 9600);
        assert_eq!(d.data_bits, DataBits::Eight);
        assert_eq!(d.line_ending, LineEnding::CrLf);
    }

    #[test]
    fn settings_serde_roundtrip() {
        let s = Settings::default();
        let json = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.defaults.baud_rate, s.defaults.baud_rate);
    }
}
```

Note `DefaultsConfig` doesn't derive `Default` automatically because the field types don't all derive Default. We implement it manually to control the values.

- [ ] **Step 2: Register the module**

Edit `src-tauri/src/commands/mod.rs`:

```rust
pub mod logs;
pub mod macros;
pub mod ports;
pub mod settings;
```

Edit `src-tauri/src/lib.rs`. Add to the invoke_handler list:

```rust
            commands::settings::load_settings,
            commands::settings::save_settings,
```

- [ ] **Step 3: Build + test**

```bash
cd src-tauri
cargo test --lib settings
```

Expected: 2 tests pass.

```bash
cargo build
```

Clean.

- [ ] **Step 4: Commit**

```bash
cd ..
git add src-tauri/src/commands/settings.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(backend): settings.json with connection defaults"
```

---

## Task 7: Frontend `stores/settings.ts` + load on app start

**Files:**
- Create: `src/lib/stores/settings.ts`
- Modify: `src/lib/ipc.ts`
- Modify: `src/lib/stores/tabs.ts` (use settings as defaults when creating tabs)
- Modify: `src/App.svelte` (load settings on mount)

- [ ] **Step 1: Add Settings types + ipc methods**

Edit `src/lib/ipc.ts`. Add types:

```typescript
export interface DefaultsConfig {
  baud_rate: number;
  data_bits: DataBits;
  stop_bits: StopBits;
  parity: Parity;
  flow_control: FlowControl;
  line_ending: LineEnding;
  send_mode: SendMode;
  auto_reconnect: boolean;
  reconnect_delay_ms: number;
}

export interface Settings {
  defaults: DefaultsConfig;
}
```

Add to the `ipc` const:

```typescript
  loadSettings: () => invoke<Settings>("load_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
```

- [ ] **Step 2: Write the settings store**

`src/lib/stores/settings.ts`:

```typescript
import { writable, type Writable, get } from "svelte/store";
import { ipc, type Settings, type DefaultsConfig } from "$lib/ipc";

const initial: Settings = {
  defaults: {
    baud_rate: 9600,
    data_bits: "eight",
    stop_bits: "one",
    parity: "none",
    flow_control: "none",
    line_ending: "crlf",
    send_mode: "ascii",
    auto_reconnect: false,
    reconnect_delay_ms: 2000,
  },
};

export const settings: Writable<Settings> = writable(initial);
export const settingsLoaded: Writable<boolean> = writable(false);

export async function loadSettings() {
  try {
    const s = await ipc.loadSettings();
    settings.set(s);
  } catch {
    settings.set(initial);
  } finally {
    settingsLoaded.set(true);
  }
}

export async function persistDefaults(defaults: DefaultsConfig) {
  const next: Settings = { ...get(settings), defaults };
  const saved = await ipc.saveSettings(next);
  settings.set(saved);
}

export function snapshotDefaults(): DefaultsConfig {
  return get(settings).defaults;
}
```

- [ ] **Step 3: Update tabs.ts to use settings on tab creation**

Edit `src/lib/stores/tabs.ts`. Add the import:

```typescript
import { snapshotDefaults } from "$lib/stores/settings";
```

Update `defaultConfig()` to read from settings:

```typescript
function defaultConfig(): PortConfig {
  const d = snapshotDefaults();
  return {
    port_name: "",
    baud_rate: d.baud_rate,
    data_bits: d.data_bits,
    stop_bits: d.stop_bits,
    parity: d.parity,
    flow_control: d.flow_control,
    auto_reconnect: d.auto_reconnect,
    reconnect_delay_ms: d.reconnect_delay_ms,
  };
}
```

Update `defaultTab()` to also pull line_ending and send_mode from settings:

Find the existing `defaultTab(...)` and replace the line_ending/sendMode lines:

```typescript
function defaultTab(id: TabId = allocTabId()): Tab {
  const d = snapshotDefaults();
  return {
    id,
    config: defaultConfig(),
    state: "disconnected",
    buffer: new Uint8Array(0),
    rxBytes: 0,
    txBytes: 0,
    viewMode: "ascii",
    stripAnsi: true,
    autoScroll: true,
    sendMode: d.send_mode,
    lineEnding: d.line_ending,
    dtr: false,
    rts: false,
    errorMessage: null,
    history: [],
    historyIndex: -1,
    filterEnabled: false,
    filterPattern: "",
    loggingEnabled: false,
    logEntries: [],
    searchOpen: false,
    searchPattern: "",
    searchUseRegex: false,
    searchMatchIndex: -1,
  };
}
```

- [ ] **Step 4: Load settings on app start**

Edit `src/App.svelte`. Add to imports:

```typescript
import { loadSettings } from "$lib/stores/settings";
```

In `onMount`, await loadSettings BEFORE startEventRouter (so the initial tab uses persisted defaults):

```typescript
onMount(async () => {
  await loadSettings();
  startPolling();
  startEventRouter();
  window.addEventListener("keydown", onWindowKey);
});
```

- [ ] **Step 5: Persist defaults when ConnectionPanel changes settings**

Edit `src/lib/components/panels/ConnectionPanel.svelte`. Add to imports:

```typescript
import { persistDefaults } from "$lib/stores/settings";
```

Add a function inside the script block:

```typescript
async function persistAsDefault() {
  const c = $activeTab.config;
  await persistDefaults({
    baud_rate: c.baud_rate,
    data_bits: c.data_bits,
    stop_bits: c.stop_bits,
    parity: c.parity,
    flow_control: c.flow_control,
    line_ending: $activeTab.lineEnding,
    send_mode: $activeTab.sendMode,
    auto_reconnect: c.auto_reconnect ?? false,
    reconnect_delay_ms: c.reconnect_delay_ms ?? 2000,
  });
}
```

Add a "Save as default" button below the Connect/Disconnect button (still inside the `.panel`):

```svelte
<button
  onclick={persistAsDefault}
  title="Save current settings as defaults for new tabs"
  class="save-default"
>
  Save as default
</button>
```

Add to the panel's `<style>`:

```css
.save-default { width: 100%; margin-top: 4px; padding: 4px 0; font-size: 10px; }
```

- [ ] **Step 6: Type check + build**

```bash
npm run check
npm run build
```

Both pass.

- [ ] **Step 7: Smoke**

`cargo tauri dev`. Open Connection panel, change baud to 115200, click "Save as default". Open a new tab — its baud is 115200. Restart the app — same.

- [ ] **Step 8: Commit**

```bash
git add src/lib/ipc.ts src/lib/stores/settings.ts src/lib/stores/tabs.ts src/App.svelte src/lib/components/panels/ConnectionPanel.svelte
git commit -m "feat(ui): persist connection defaults via settings.json"
```

---

## Task 8: Auto-reconnect state machine

**Files:**
- Modify: `src-tauri/src/port/worker.rs`
- Modify: `src-tauri/src/port/manager.rs`

`PortConfig.auto_reconnect` and `reconnect_delay_ms` already exist. Wire them so the worker, on a disconnect, retries opening the port indefinitely until either Shutdown is sent or the port returns.

- [ ] **Step 1: Refactor worker to support reconnect loop**

The current `spawn` function opens the port then enters `run_loop`. When `run_loop` returns (disconnect), the thread exits. We change this so the worker thread loops: try open → run until disconnect → if `auto_reconnect`, sleep + retry → else exit.

Edit `src-tauri/src/port/worker.rs`. Replace `pub fn spawn(...)` and `fn run_loop(...)` with:

```rust
pub fn spawn(
    tab_id: TabId,
    config: PortConfig,
    app: AppHandle,
) -> AppResult<PortHandle> {
    // First open is synchronous so a connect failure surfaces immediately.
    let port = config.to_serial().open()?;
    let (tx, rx) = mpsc::channel::<WorkerCommand>();
    let join = std::thread::Builder::new()
        .name(format!("rustcom-port-{tab_id}"))
        .spawn(move || run_supervisor(tab_id, port, config, rx, app))
        .map_err(|e| AppError::Io(e.to_string()))?;

    Ok(PortHandle { sender: tx, join })
}

fn run_supervisor(
    tab_id: TabId,
    initial_port: Box<dyn SerialPort>,
    config: PortConfig,
    rx: Receiver<WorkerCommand>,
    app: AppHandle,
) {
    let mut port = initial_port;
    loop {
        let outcome = run_loop(tab_id, port, &rx, &app);
        match outcome {
            LoopOutcome::Shutdown => {
                app.state::<crate::port::manager::PortManager>().evict(tab_id);
                return;
            }
            LoopOutcome::Disconnected(reason) => {
                emit_disconnect(&app, tab_id, reason.clone());
                if !config.auto_reconnect {
                    app.state::<crate::port::manager::PortManager>().evict(tab_id);
                    return;
                }
                // Reconnect loop: sleep, retry open, repeat. Honor Shutdown commands while sleeping.
                let delay = Duration::from_millis(config.reconnect_delay_ms);
                loop {
                    if shutdown_pending(&rx) {
                        app.state::<crate::port::manager::PortManager>().evict(tab_id);
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(100.min(delay.as_millis() as u64)));
                    let elapsed = std::time::Instant::now();
                    while elapsed.elapsed() < delay {
                        if shutdown_pending(&rx) {
                            app.state::<crate::port::manager::PortManager>().evict(tab_id);
                            return;
                        }
                        std::thread::sleep(Duration::from_millis(50));
                    }
                    match config.to_serial().open() {
                        Ok(p) => {
                            emit_reconnect(&app, tab_id);
                            port = p;
                            break;
                        }
                        Err(_) => continue,
                    }
                }
            }
        }
    }
}

enum LoopOutcome {
    Shutdown,
    Disconnected(String),
}

fn shutdown_pending(rx: &Receiver<WorkerCommand>) -> bool {
    matches!(rx.try_recv(), Ok(WorkerCommand::Shutdown) | Err(TryRecvError::Disconnected))
}

fn run_loop(
    tab_id: TabId,
    mut port: Box<dyn SerialPort>,
    rx: &Receiver<WorkerCommand>,
    app: &AppHandle,
) -> LoopOutcome {
    let mut read_buf = vec![0u8; READ_BUFFER_SIZE];
    loop {
        // Drain pending commands.
        loop {
            match rx.try_recv() {
                Ok(WorkerCommand::Write(bytes)) => {
                    let _ = port.set_timeout(Duration::from_secs(2));
                    let result = port.write_all(&bytes);
                    let _ = port.set_timeout(Duration::from_millis(10));
                    if result.is_ok() {
                        emit_tx(app, tab_id, bytes);
                    } else if let Err(e) = result {
                        return LoopOutcome::Disconnected(format!("write failed: {e}"));
                    }
                }
                Ok(WorkerCommand::SetDtr(state)) => {
                    let _ = port.write_data_terminal_ready(state);
                }
                Ok(WorkerCommand::SetRts(state)) => {
                    let _ = port.write_request_to_send(state);
                }
                Ok(WorkerCommand::Shutdown) => return LoopOutcome::Shutdown,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return LoopOutcome::Shutdown,
            }
        }

        match port.read(&mut read_buf) {
            Ok(n) if n > 0 => emit_rx(app, tab_id, read_buf[..n].to_vec()),
            Ok(_) => std::thread::sleep(Duration::from_millis(1)),
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => return LoopOutcome::Disconnected(format!("read failed: {e}")),
        }
    }
}

fn emit_reconnect(app: &AppHandle, tab_id: TabId) {
    #[derive(Serialize, Clone)]
    struct ReconnectPayload {
        ts: i64,
    }
    let payload = ReconnectPayload { ts: chrono::Local::now().timestamp_millis() };
    let _ = app.emit(&format!("reconnect://{tab_id}"), payload);
}
```

- [ ] **Step 2: Add a `reconnect` event listener on the frontend**

Edit `src/lib/ipc.ts`. Add type and listener:

```typescript
export interface ReconnectPayload {
  ts: number;
}

export function onReconnect(
  tabId: TabId,
  cb: EventCallback<ReconnectPayload>
): Promise<UnlistenFn> {
  return listen<ReconnectPayload>(`reconnect://${tabId}`, cb);
}
```

Edit `src/lib/eventRouter.ts`. Add subscription in `subscribe`:

```typescript
import { tabs, appendBytes, pushLog, type Tab } from "$lib/stores/tabs";
import {
  ipc,
  onRx,
  onTx,
  onDisconnect,
  onReconnect,
  type RxPayload,
  type TxPayload,
  type DisconnectPayload,
  type ReconnectPayload,
  type TabId,
} from "$lib/ipc";
```

In `subscribe`:

```typescript
const unre = await onReconnect(id, (_event) => {
  tabs.update((list) =>
    list.map((t) => (t.id === id ? { ...t, state: "connected", errorMessage: null } : t))
  );
});

subs.set(id, [unrx, untx, undisc, unre]);
```

Also: the existing `onDisconnect` handler currently calls `ipc.closePort(id)` defensively. With auto-reconnect, we DON'T want to evict on disconnect because the worker is still alive trying to reopen. Adjust the disconnect handler to read the tab's `auto_reconnect` setting:

```typescript
const undisc = await onDisconnect(id, (event) => {
  const p = event.payload as DisconnectPayload;
  let willReconnect = false;
  tabs.update((list) =>
    list.map((t) => {
      if (t.id !== id) return t;
      willReconnect = !!t.config.auto_reconnect;
      const next: Tab = {
        ...t,
        state: willReconnect ? "reconnecting" : "disconnected",
        errorMessage: p.reason,
        dtr: false,
        rts: false,
      };
      return next;
    })
  );
  if (!willReconnect) {
    ipc.closePort(id).catch(() => undefined);
  }
});
```

- [ ] **Step 3: Expose auto-reconnect toggle in ConnectionPanel**

Edit `src/lib/components/panels/ConnectionPanel.svelte`. Below the Flow select (and before the Connect button), add:

```svelte
<label class="toggle">
  <input
    type="checkbox"
    checked={$activeTab.config.auto_reconnect ?? false}
    onchange={(e) => patchActiveTab({
      config: { ...$activeTab.config, auto_reconnect: (e.currentTarget as HTMLInputElement).checked },
    })}
    disabled={$activeTab.state === "connected"}
  />
  Auto-reconnect
</label>
```

Add the matching style:

```css
.toggle { display: flex; align-items: center; gap: 6px; font-size: 11px; color: var(--fg-muted); margin-top: 6px; }
```

- [ ] **Step 4: Build + smoke**

```bash
cargo tauri dev
```

Connect to a port. Toggle Auto-reconnect ON. Yank the USB cable — the title bar dot turns yellow ("warn"), tab status flips to "reconnecting". Plug back in — within `reconnect_delay_ms` the tab returns to "connected".

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/port/worker.rs src/lib/ipc.ts src/lib/eventRouter.ts src/lib/components/panels/ConnectionPanel.svelte
git commit -m "feat(backend): auto-reconnect state machine in PortWorker"
```

---

## Task 9: Backend `mlua` + `script/api.rs` skeleton + tests

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/script/mod.rs`
- Create: `src-tauri/src/script/api.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add mlua to Cargo.toml**

Edit `src-tauri/Cargo.toml`, append to `[dependencies]`:

```toml
mlua = { version = "0.10", features = ["lua54", "vendored", "send"] }
```

The `vendored` feature builds Lua from source so we don't depend on a system Lua. `send` makes `Lua` Send (so we can move it to a thread).

Run `cargo build` from `src-tauri/` to confirm download + compile.

- [ ] **Step 2: Add a script module skeleton**

`src-tauri/src/script/mod.rs`:

```rust
pub mod api;
pub mod engine;
```

`src-tauri/src/script/engine.rs` (stub for now — fully written in Task 10):

```rust
// Filled in by Task 10.
```

`src-tauri/src/script/api.rs`:

```rust
use std::sync::mpsc::Sender;

use mlua::{Lua, Result as LuaResult, Table, Value};

use crate::port::worker::{TabId, WorkerCommand};

/// Commands the script engine sends out to the rest of the app.
#[derive(Debug)]
pub enum ScriptOut {
    Send { tab_id: TabId, bytes: Vec<u8> },
    Toast { level: String, msg: String },
    Log { msg: String },
}

/// Install the `serial`, `ui`, and free functions onto the Lua globals.
/// Closures capture an `mpsc::Sender<ScriptOut>` so script API calls become messages.
pub fn install_api(lua: &Lua, tab_id: TabId, out: Sender<ScriptOut>) -> LuaResult<()> {
    let globals = lua.globals();

    // serial.send(bytes) — bytes is a Lua table of integers 0..255.
    let send_out = out.clone();
    let serial = lua.create_table()?;
    serial.set(
        "send",
        lua.create_function(move |_, bytes: Table| {
            let mut buf = Vec::new();
            for pair in bytes.sequence_values::<u8>() {
                buf.push(pair?);
            }
            let _ = send_out.send(ScriptOut::Send { tab_id, bytes: buf });
            Ok(())
        })?,
    )?;

    let send_text_out = out.clone();
    serial.set(
        "send_text",
        lua.create_function(move |_, text: String| {
            let _ = send_text_out.send(ScriptOut::Send {
                tab_id,
                bytes: text.into_bytes(),
            });
            Ok(())
        })?,
    )?;

    let send_hex_out = out.clone();
    serial.set(
        "send_hex",
        lua.create_function(move |_, input: String| {
            match crate::hex::parse_hex_input(&input) {
                Ok(bytes) => {
                    let _ = send_hex_out.send(ScriptOut::Send { tab_id, bytes });
                    Ok(())
                }
                Err(e) => Err(mlua::Error::external(e)),
            }
        })?,
    )?;

    globals.set("serial", serial)?;

    // ui.toast(msg, level?)
    let toast_out = out.clone();
    let ui = lua.create_table()?;
    ui.set(
        "toast",
        lua.create_function(move |_, args: (String, Option<String>)| {
            let (msg, level) = args;
            let _ = toast_out.send(ScriptOut::Toast {
                level: level.unwrap_or_else(|| "info".to_string()),
                msg,
            });
            Ok(())
        })?,
    )?;
    globals.set("ui", ui)?;

    // log(msg)
    let log_out = out;
    globals.set(
        "log",
        lua.create_function(move |_, msg: Value| {
            let s = match msg {
                Value::String(s) => s.to_str()?.to_string(),
                other => format!("{other:?}"),
            };
            let _ = log_out.send(ScriptOut::Log { msg: s });
            Ok(())
        })?,
    )?;

    // delay(ms) — sleeps the script thread.
    globals.set(
        "delay",
        lua.create_function(|_, ms: u64| {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            Ok(())
        })?,
    )?;

    Ok(())
}

/// Convenience helper to translate a Lua table of bytes into a Vec<u8>
/// from inside an on_recv callback.
#[cfg(test)]
pub fn _ensure_workercommand_in_scope() {
    let _: Option<WorkerCommand> = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    fn fresh() -> (Lua, mpsc::Receiver<ScriptOut>) {
        let lua = Lua::new();
        let (tx, rx) = mpsc::channel();
        install_api(&lua, 7, tx).unwrap();
        (lua, rx)
    }

    #[test]
    fn serial_send_forwards_bytes() {
        let (lua, rx) = fresh();
        lua.load("serial.send({0x41, 0x42})").exec().unwrap();
        match rx.recv().unwrap() {
            ScriptOut::Send { tab_id, bytes } => {
                assert_eq!(tab_id, 7);
                assert_eq!(bytes, vec![0x41, 0x42]);
            }
            other => panic!("expected Send, got {other:?}"),
        }
    }

    #[test]
    fn serial_send_text_forwards_string_bytes() {
        let (lua, rx) = fresh();
        lua.load(r#"serial.send_text("AT")"#).exec().unwrap();
        match rx.recv().unwrap() {
            ScriptOut::Send { bytes, .. } => assert_eq!(bytes, b"AT".to_vec()),
            other => panic!("expected Send, got {other:?}"),
        }
    }

    #[test]
    fn serial_send_hex_forwards_parsed_bytes() {
        let (lua, rx) = fresh();
        lua.load(r#"serial.send_hex("AA BB 0D 0A")"#).exec().unwrap();
        match rx.recv().unwrap() {
            ScriptOut::Send { bytes, .. } => assert_eq!(bytes, vec![0xAA, 0xBB, 0x0D, 0x0A]),
            other => panic!("expected Send, got {other:?}"),
        }
    }

    #[test]
    fn log_forwards_string() {
        let (lua, rx) = fresh();
        lua.load(r#"log("hello")"#).exec().unwrap();
        match rx.recv().unwrap() {
            ScriptOut::Log { msg } => assert_eq!(msg, "hello"),
            other => panic!("expected Log, got {other:?}"),
        }
    }

    #[test]
    fn ui_toast_with_level() {
        let (lua, rx) = fresh();
        lua.load(r#"ui.toast("oops", "warn")"#).exec().unwrap();
        match rx.recv().unwrap() {
            ScriptOut::Toast { level, msg } => {
                assert_eq!(level, "warn");
                assert_eq!(msg, "oops");
            }
            other => panic!("expected Toast, got {other:?}"),
        }
    }
}
```

- [ ] **Step 3: Register the module in lib.rs**

Edit `src-tauri/src/lib.rs`. Add to the module decls:

```rust
pub mod script;
```

- [ ] **Step 4: Test**

```bash
cd src-tauri
cargo test --lib script::api
```

Expected: 5 tests pass. (First build of `mlua` with `vendored` will compile Lua C — slow on first run.)

- [ ] **Step 5: Commit**

```bash
cd ..
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/script src-tauri/src/lib.rs
git commit -m "feat(backend): mlua-backed Lua API for serial.send/log/toast/delay"
```

---

## Task 10: `ScriptEngine` thread + `commands/scripts.rs`

**Files:**
- Replace: `src-tauri/src/script/engine.rs`
- Create: `src-tauri/src/commands/scripts.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/storage.rs` (if needed for scripts dir)

The engine runs on its own thread, owns the `Lua` instance, and processes:
- `Run(tab_id, source)` — compile + execute, replace any prior `on_recv`.
- `Stop` — drop the current binding.
- `OnRecv(tab_id, bytes)` — invoke the registered `on_recv(bytes)` callback.

The engine emits `ScriptOut` to a forwarder thread that:
- For `Send`: dispatches to `PortManager.send(tab_id, bytes)`.
- For `Toast`: emits `toast` Tauri event.
- For `Log`: emits `script_log://<tab_id>` Tauri event.

- [ ] **Step 1: Write the engine**

`src-tauri/src/script/engine.rs`:

```rust
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use mlua::{Function, Lua};
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::{AppError, AppResult};
use crate::port::worker::TabId;
use crate::script::api::{install_api, ScriptOut};

#[derive(Debug)]
pub enum EngineCommand {
    Run { tab_id: TabId, source: String },
    Stop,
    OnRecv { tab_id: TabId, bytes: Vec<u8> },
    Shutdown,
}

pub struct ScriptEngine {
    sender: Mutex<Option<Sender<EngineCommand>>>,
    join: Mutex<Option<JoinHandle<()>>>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        Self {
            sender: Mutex::new(None),
            join: Mutex::new(None),
        }
    }

    pub fn start(&self, app: AppHandle) -> AppResult<()> {
        let mut sender_slot = self.sender.lock();
        if sender_slot.is_some() {
            return Ok(()); // already running
        }
        let (tx, rx) = mpsc::channel::<EngineCommand>();
        *sender_slot = Some(tx);
        drop(sender_slot);

        let app_for_thread = app.clone();
        let handle = thread::Builder::new()
            .name("rustcom-script-engine".to_string())
            .spawn(move || run_engine(rx, app_for_thread))
            .map_err(|e| AppError::Io(e.to_string()))?;
        *self.join.lock() = Some(handle);
        Ok(())
    }

    pub fn send(&self, cmd: EngineCommand) -> AppResult<()> {
        let guard = self.sender.lock();
        let sender = guard
            .as_ref()
            .ok_or_else(|| AppError::Invalid("script engine is not running".to_string()))?;
        sender
            .send(cmd)
            .map_err(|_| AppError::Invalid("script engine thread is dead".to_string()))
    }

    pub fn run(&self, tab_id: TabId, source: String) -> AppResult<()> {
        self.send(EngineCommand::Run { tab_id, source })
    }

    pub fn stop(&self) -> AppResult<()> {
        self.send(EngineCommand::Stop)
    }

    pub fn dispatch_recv(&self, tab_id: TabId, bytes: Vec<u8>) {
        let _ = self.send(EngineCommand::OnRecv { tab_id, bytes });
    }
}

impl Default for ScriptEngine {
    fn default() -> Self { Self::new() }
}

fn run_engine(rx: Receiver<EngineCommand>, app: AppHandle) {
    let lua = Lua::new();
    let (out_tx, out_rx) = mpsc::channel::<ScriptOut>();

    // Active script context: the tab id it was bound to.
    let mut bound_tab: Option<TabId> = None;

    // Spawn the forwarder thread that turns ScriptOut into Tauri events / port writes.
    let app_forward = app.clone();
    thread::Builder::new()
        .name("rustcom-script-forwarder".to_string())
        .spawn(move || forward_loop(out_rx, app_forward))
        .ok();

    loop {
        match rx.recv() {
            Ok(EngineCommand::Run { tab_id, source }) => {
                // Reset globals: clear on_recv from any previous run.
                let _ = lua.globals().set("on_recv", mlua::Value::Nil);
                if let Err(e) = install_api(&lua, tab_id, out_tx.clone()) {
                    let _ = app.emit(
                        "script_error",
                        format!("install_api failed: {e}"),
                    );
                    continue;
                }
                if let Err(e) = lua.load(&source).exec() {
                    let _ = app.emit("script_error", format!("{e}"));
                    continue;
                }
                bound_tab = Some(tab_id);
            }
            Ok(EngineCommand::Stop) => {
                let _ = lua.globals().set("on_recv", mlua::Value::Nil);
                bound_tab = None;
            }
            Ok(EngineCommand::OnRecv { tab_id, bytes }) => {
                if bound_tab != Some(tab_id) {
                    continue;
                }
                let cb_value: mlua::Value = match lua.globals().get("on_recv") {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let cb: Function = match cb_value {
                    mlua::Value::Function(f) => f,
                    _ => continue,
                };
                let table = match lua.create_sequence_from(bytes.into_iter()) {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                if let Err(e) = cb.call::<()>(table) {
                    let _ = app.emit("script_error", format!("on_recv: {e}"));
                }
            }
            Ok(EngineCommand::Shutdown) => return,
            Err(_) => return,
        }
    }
}

fn forward_loop(rx: Receiver<ScriptOut>, app: AppHandle) {
    use crate::port::manager::PortManager;
    loop {
        let item = match rx.recv() {
            Ok(v) => v,
            Err(_) => return,
        };
        match item {
            ScriptOut::Send { tab_id, bytes } => {
                let mgr = app.state::<PortManager>();
                let _ = mgr.send(tab_id, bytes);
            }
            ScriptOut::Toast { level, msg } => {
                #[derive(serde::Serialize, Clone)]
                struct ToastPayload { level: String, msg: String, ts: i64 }
                let _ = app.emit(
                    "toast",
                    ToastPayload { level, msg, ts: chrono::Local::now().timestamp_millis() },
                );
            }
            ScriptOut::Log { msg } => {
                #[derive(serde::Serialize, Clone)]
                struct LogPayload { msg: String, ts: i64 }
                let _ = app.emit(
                    "script_log",
                    LogPayload { msg, ts: chrono::Local::now().timestamp_millis() },
                );
            }
        }
    }
}
```

- [ ] **Step 2: Wire the ScriptEngine into PortManager's RX path**

The engine needs `OnRecv` events. The cleanest hook is in `PortWorker`'s `emit_rx` — but we don't want a circular dep. Instead, do it from the worker via the `AppHandle.state::<ScriptEngine>()`.

Edit `src-tauri/src/port/worker.rs`. Find `fn emit_rx` and update to also dispatch into the engine:

```rust
fn emit_rx(app: &AppHandle, tab_id: TabId, bytes: Vec<u8>) {
    let payload = RxPayload { bytes: bytes.clone(), ts: chrono::Local::now().timestamp_millis() };
    let _ = app.emit(&format!("rx://{tab_id}"), payload);

    if let Some(engine) = app.try_state::<crate::script::engine::ScriptEngine>() {
        engine.dispatch_recv(tab_id, bytes);
    }
}
```

(Use `try_state` because the engine state may not be installed in test contexts.)

- [ ] **Step 3: Register the engine in lib.rs**

Edit `src-tauri/src/lib.rs`. Add an import + register state + start the engine on app startup:

```rust
use port::manager::PortManager;
use script::engine::ScriptEngine;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    log::info!("RustCOM starting");
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(PortManager::new())
        .manage(ScriptEngine::new())
        .setup(|app| {
            let handle = app.handle().clone();
            let engine = handle.state::<ScriptEngine>();
            engine.start(handle.clone()).ok();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ports::list_ports,
            commands::ports::open_port,
            commands::ports::close_port,
            commands::ports::send_text,
            commands::ports::send_hex,
            commands::ports::set_dtr,
            commands::ports::set_rts,
            commands::macros::list_macros,
            commands::macros::save_macro,
            commands::macros::delete_macro,
            commands::logs::save_log,
            commands::settings::load_settings,
            commands::settings::save_settings,
            commands::scripts::list_scripts,
            commands::scripts::save_script,
            commands::scripts::delete_script,
            commands::scripts::script_run,
            commands::scripts::script_stop,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Write the script commands module**

`src-tauri/src/commands/scripts.rs`:

```rust
use std::fs;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::port::worker::TabId;
use crate::script::engine::ScriptEngine;
use crate::storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: String,   // file name without extension, e.g. "heartbeat"
    pub source: String,
}

fn scripts_dir() -> AppResult<std::path::PathBuf> {
    let dir = storage::data_dir()?.join("scripts");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| AppError::Io(e.to_string()))?;
    }
    Ok(dir)
}

fn validate_id(id: &str) -> AppResult<()> {
    if id.is_empty() || id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err(AppError::Invalid(format!("invalid script id: {id}")));
    }
    Ok(())
}

#[tauri::command]
pub fn list_scripts() -> AppResult<Vec<Script>> {
    let dir = scripts_dir()?;
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| AppError::Io(e.to_string()))? {
        let entry = entry.map_err(|e| AppError::Io(e.to_string()))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("lua") {
            continue;
        }
        let id = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let source = fs::read_to_string(&path).map_err(|e| AppError::Io(e.to_string()))?;
        out.push(Script { id, source });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

#[tauri::command]
pub fn save_script(item: Script) -> AppResult<Vec<Script>> {
    validate_id(&item.id)?;
    let path = scripts_dir()?.join(format!("{}.lua", item.id));
    fs::write(&path, item.source).map_err(|e| AppError::Io(e.to_string()))?;
    list_scripts()
}

#[tauri::command]
pub fn delete_script(id: String) -> AppResult<Vec<Script>> {
    validate_id(&id)?;
    let path = scripts_dir()?.join(format!("{}.lua", id));
    if path.exists() {
        fs::remove_file(&path).map_err(|e| AppError::Io(e.to_string()))?;
    }
    list_scripts()
}

#[tauri::command]
pub fn script_run(tab_id: TabId, source: String, engine: State<'_, ScriptEngine>) -> AppResult<()> {
    engine.run(tab_id, source)
}

#[tauri::command]
pub fn script_stop(engine: State<'_, ScriptEngine>) -> AppResult<()> {
    engine.stop()
}
```

- [ ] **Step 5: Register module**

Edit `src-tauri/src/commands/mod.rs`:

```rust
pub mod logs;
pub mod macros;
pub mod ports;
pub mod scripts;
pub mod settings;
```

- [ ] **Step 6: Build + smoke test**

```bash
cd src-tauri
cargo test --lib script
```

Expected: api tests still pass.

```bash
cargo build
```

Clean.

- [ ] **Step 7: Commit**

```bash
cd ..
git add src-tauri/src/script src-tauri/src/commands/scripts.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src-tauri/src/port/worker.rs
git commit -m "feat(backend): ScriptEngine thread + script CRUD/run/stop commands"
```

---

## Task 11: Frontend `stores/scripts.ts` + ipc wrappers

**Files:**
- Modify: `src/lib/ipc.ts`
- Create: `src/lib/stores/scripts.ts`
- Test: `src/lib/stores/scripts.test.ts`

- [ ] **Step 1: Add Script types + ipc methods**

Edit `src/lib/ipc.ts`. Add types:

```typescript
export interface Script {
  id: string;
  source: string;
}

export interface ToastPayload {
  level: string;
  msg: string;
  ts: number;
}

export interface ScriptLogPayload {
  msg: string;
  ts: number;
}
```

Add to `ipc`:

```typescript
  listScripts: () => invoke<Script[]>("list_scripts"),
  saveScript: (item: Script) => invoke<Script[]>("save_script", { item }),
  deleteScript: (id: string) => invoke<Script[]>("delete_script", { id }),
  scriptRun: (tabId: TabId, source: string) =>
    invoke<void>("script_run", { tabId, source }),
  scriptStop: () => invoke<void>("script_stop"),
```

Add event listeners:

```typescript
export function onToast(cb: EventCallback<ToastPayload>): Promise<UnlistenFn> {
  return listen<ToastPayload>("toast", cb);
}

export function onScriptLog(cb: EventCallback<ScriptLogPayload>): Promise<UnlistenFn> {
  return listen<ScriptLogPayload>("script_log", cb);
}

export function onScriptError(cb: EventCallback<string>): Promise<UnlistenFn> {
  return listen<string>("script_error", cb);
}
```

- [ ] **Step 2: Write the scripts store**

`src/lib/stores/scripts.ts`:

```typescript
import { writable, type Writable } from "svelte/store";
import { ipc, type Script, type TabId } from "$lib/ipc";

export const scripts: Writable<Script[]> = writable([]);
export const scriptsLoaded: Writable<boolean> = writable(false);
export const runningScriptId: Writable<string | null> = writable(null);

export async function loadScripts() {
  try {
    const list = await ipc.listScripts();
    scripts.set(list);
  } catch {
    scripts.set([]);
  } finally {
    scriptsLoaded.set(true);
  }
}

export async function upsertScript(item: Script) {
  const next = await ipc.saveScript(item);
  scripts.set(next);
}

export async function removeScript(id: string) {
  const next = await ipc.deleteScript(id);
  scripts.set(next);
}

export async function runScript(tabId: TabId, item: Script) {
  await ipc.scriptRun(tabId, item.source);
  runningScriptId.set(item.id);
}

export async function stopScript() {
  await ipc.scriptStop();
  runningScriptId.set(null);
}
```

- [ ] **Step 3: Tests**

`src/lib/stores/scripts.test.ts`:

```typescript
import { describe, it, expect, vi, beforeEach } from "vitest";
import { get } from "svelte/store";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => invokeMock(...args) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(() => Promise.resolve(() => undefined)) }));

import { scripts, loadScripts, upsertScript, removeScript, runScript, stopScript, runningScriptId } from "./scripts";

beforeEach(() => {
  invokeMock.mockReset();
  scripts.set([]);
  runningScriptId.set(null);
});

describe("scripts store", () => {
  it("loadScripts populates from list_scripts", async () => {
    invokeMock.mockResolvedValueOnce([{ id: "h", source: "log('hi')" }]);
    await loadScripts();
    expect(get(scripts).map((s) => s.id)).toEqual(["h"]);
  });

  it("upsertScript replaces store with response", async () => {
    invokeMock.mockResolvedValueOnce([{ id: "h", source: "x" }]);
    await upsertScript({ id: "h", source: "x" });
    expect(invokeMock).toHaveBeenCalledWith("save_script", { item: { id: "h", source: "x" } });
    expect(get(scripts).length).toBe(1);
  });

  it("removeScript replaces store with response", async () => {
    invokeMock.mockResolvedValueOnce([]);
    await removeScript("h");
    expect(invokeMock).toHaveBeenCalledWith("delete_script", { id: "h" });
    expect(get(scripts)).toEqual([]);
  });

  it("runScript records running id", async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    await runScript(3, { id: "h", source: "log('x')" });
    expect(invokeMock).toHaveBeenCalledWith("script_run", { tabId: 3, source: "log('x')" });
    expect(get(runningScriptId)).toBe("h");
  });

  it("stopScript clears running id", async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    runningScriptId.set("h");
    await stopScript();
    expect(invokeMock).toHaveBeenCalledWith("script_stop");
    expect(get(runningScriptId)).toBeNull();
  });
});
```

- [ ] **Step 4: Run tests + check + build**

```bash
npm run test
npm run check
npm run build
```

All three pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib/ipc.ts src/lib/stores/scripts.ts src/lib/stores/scripts.test.ts
git commit -m "feat(ui): scripts store backed by Tauri commands"
```

---

## Task 12: Monaco package + loader

**Files:**
- Modify: `package.json`
- Create: `src/lib/monaco.ts`
- Modify: `vite.config.ts`

The Monaco editor is heavy (~2 MB). We load it via dynamic import so it's a separate chunk.

- [ ] **Step 1: Install Monaco**

Edit `package.json` `dependencies`:

```json
"monaco-editor": "^0.52.0"
```

Run:

```bash
npm install
```

- [ ] **Step 2: Configure Vite to handle Monaco's web workers**

Edit `vite.config.ts`. Add to the `defineConfig` block (alongside `plugins` etc.):

```typescript
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import path from "node:path";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**", "**/legacy-egui/**"] },
  },
  resolve: {
    alias: { $lib: path.resolve("./src/lib") },
  },
  optimizeDeps: {
    include: ["monaco-editor/esm/vs/editor/editor.api"],
  },
  worker: {
    format: "es",
  },
  test: {
    globals: true,
    environment: "jsdom",
    include: ["src/**/*.test.ts"],
  },
});
```

- [ ] **Step 3: Write a Monaco loader**

`src/lib/monaco.ts`:

```typescript
import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
import EditorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";

// Tell Monaco how to spawn workers in a Vite-bundled context.
self.MonacoEnvironment = {
  getWorker: () => new EditorWorker(),
};

// Define a "zed-cool" theme matching our app.
monaco.editor.defineTheme("zed-cool", {
  base: "vs-dark",
  inherit: true,
  rules: [
    { token: "comment", foreground: "6a6d72", fontStyle: "italic" },
    { token: "string", foreground: "8fc4a8" },
    { token: "keyword", foreground: "7a86c8" },
    { token: "number", foreground: "c4a78c" },
  ],
  colors: {
    "editor.background": "#1c1d20",
    "editor.foreground": "#d4d6d9",
    "editorLineNumber.foreground": "#6a6d72",
    "editorCursor.foreground": "#7a86c8",
    "editor.selectionBackground": "#2a2c30",
    "editor.lineHighlightBackground": "#1f2124",
  },
});

export { monaco };
```

- [ ] **Step 4: Build to confirm**

```bash
npm run build
```

Expected: clean. The output bundle will list a separate `monaco-editor`-sized chunk.

- [ ] **Step 5: Commit**

```bash
git add package.json package-lock.json vite.config.ts src/lib/monaco.ts
git commit -m "feat(ui): add monaco-editor with Zed Cool theme + worker config"
```

---

## Task 13: `ScriptsPanel.svelte`

**Files:**
- Create: `src/lib/components/panels/ScriptsPanel.svelte`
- Modify: `src/App.svelte`
- Modify: `src/lib/components/ActivityBar.svelte`

- [ ] **Step 1: Write the panel**

`src/lib/components/panels/ScriptsPanel.svelte`:

```svelte
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {
    scripts,
    runningScriptId,
    loadScripts,
    upsertScript,
    removeScript,
    runScript,
    stopScript,
  } from "$lib/stores/scripts";
  import { activeTab, activeTabId, patchActiveTab } from "$lib/stores/tabs";
  import { onScriptError, type Script } from "$lib/ipc";

  let selected = $state<Script | null>(null);
  let editorEl: HTMLDivElement | undefined = $state();
  // Use the Monaco type-only import so the type is erased at build time and
  // we still get static checking without forcing a sync import of the heavy package.
  type MonacoEditor = import("monaco-editor/esm/vs/editor/editor.api").editor.IStandaloneCodeEditor;
  let editor: MonacoEditor | undefined;
  let saving = $state(false);
  let panelError = $state<string | null>(null);
  let unErr: (() => void) | null = null;

  onMount(async () => {
    await loadScripts();
    if ($scripts.length > 0) selected = $scripts[0];
    await mountEditor();
    unErr = await onScriptError((e) => { panelError = `Script error: ${String(e.payload)}`; });
  });

  onDestroy(() => {
    editor?.dispose();
    if (unErr) unErr();
  });

  async function mountEditor() {
    if (!editorEl) return;
    const { monaco } = await import("$lib/monaco");
    editor = monaco.editor.create(editorEl, {
      value: selected?.source ?? "-- write Lua here\n",
      language: "lua",
      theme: "zed-cool",
      fontFamily: "JetBrains Mono, Cascadia Code, monospace",
      fontSize: 12,
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      automaticLayout: true,
    });
  }

  $effect(() => {
    if (!editor || !selected) return;
    if (editor.getValue() !== selected.source) {
      editor.setValue(selected.source);
    }
  });

  async function newScript() {
    const id = `script_${Date.now()}`;
    const item: Script = { id, source: "-- log('hello from RustCOM Lua')\n" };
    await upsertScript(item);
    selected = item;
  }

  async function save() {
    if (!selected || !editor) return;
    panelError = null;
    saving = true;
    try {
      const item: Script = { id: selected.id, source: editor.getValue() };
      await upsertScript(item);
      selected = item;
    } catch (e) {
      panelError = `Save failed: ${String(e)}`;
    } finally {
      saving = false;
    }
  }

  async function rename(newId: string) {
    if (!selected) return;
    if (!newId.match(/^[a-zA-Z0-9_-]+$/)) {
      panelError = "Name must be letters/digits/underscore/hyphen only";
      return;
    }
    if (newId === selected.id) return;
    const newItem: Script = { id: newId, source: selected.source };
    await upsertScript(newItem);
    if (newId !== selected.id) {
      await removeScript(selected.id);
    }
    selected = newItem;
  }

  async function del() {
    if (!selected) return;
    await removeScript(selected.id);
    selected = $scripts[0] ?? null;
    editor?.setValue(selected?.source ?? "");
  }

  async function run() {
    if (!selected || !editor) return;
    if ($activeTab.state !== "connected") {
      patchActiveTab({ errorMessage: "Connect a port before running a script" });
      return;
    }
    panelError = null;
    try {
      await runScript($activeTabId, { id: selected.id, source: editor.getValue() });
    } catch (e) {
      panelError = `Run failed: ${String(e)}`;
    }
  }

  async function stop() {
    try { await stopScript(); } catch (e) { panelError = `Stop failed: ${String(e)}`; }
  }
</script>

<div class="panel">
  <div class="header">
    <h4>Scripts</h4>
    <button class="add" onclick={newScript}>+ New</button>
  </div>

  <div class="list">
    {#each $scripts as s (s.id)}
      <button class="item" class:active={selected?.id === s.id} onclick={() => (selected = s)}>
        {s.id}
        {#if $runningScriptId === s.id}<span class="dot ok"></span>{/if}
      </button>
    {/each}
    {#if $scripts.length === 0}
      <p class="empty">No scripts. Click <em>+ New</em>.</p>
    {/if}
  </div>

  {#if selected}
    <input
      class="rename"
      value={selected.id}
      onchange={(e) => rename((e.currentTarget as HTMLInputElement).value)}
    />
  {/if}

  <div class="editor" bind:this={editorEl}></div>

  {#if panelError}<p class="err">{panelError}</p>{/if}

  <div class="actions">
    <button class="primary" onclick={run} disabled={!selected || $activeTab.state !== "connected"}>Run</button>
    <button onclick={stop} disabled={$runningScriptId === null}>Stop</button>
    <button onclick={save} disabled={!selected || saving}>{saving ? "Saving…" : "Save"}</button>
    <button class="danger" onclick={del} disabled={!selected}>Delete</button>
  </div>
</div>

<style>
  .panel { padding: 12px; height: 100%; box-sizing: border-box; display: flex; flex-direction: column; gap: 8px; min-height: 0; }
  .header { display: flex; align-items: center; justify-content: space-between; }
  h4 { font-size: 10px; letter-spacing: 0.1em; text-transform: uppercase; color: var(--fg-subtle); margin: 0; font-weight: 600; }
  .add { padding: 4px 8px; font-size: 10px; }

  .list { display: flex; flex-direction: column; gap: 2px; max-height: 120px; overflow-y: auto; }
  .item { display: flex; align-items: center; justify-content: space-between; padding: 4px 8px; font-size: 11px; background: transparent; color: var(--fg-muted); border-radius: var(--radius-sm); text-align: left; }
  .item:hover { background: var(--bg-input); color: var(--fg); }
  .item.active { background: var(--bg-input-hover); color: var(--accent); }
  .dot { width: 6px; height: 6px; border-radius: 50%; background: var(--ok); }
  .empty { color: var(--fg-subtle); font-size: 11px; margin: 4px 0; }

  .rename { width: 100%; font-family: var(--font-mono); font-size: 11px; }
  .editor { flex: 1; min-height: 200px; border: 1px solid var(--border-strong); border-radius: var(--radius-md); overflow: hidden; }

  .err { color: var(--err); font-size: 11px; margin: 0; }

  .actions { display: flex; gap: 4px; flex-wrap: wrap; }
  .actions button { padding: 4px 10px; font-size: 10px; flex: 1 1 auto; min-width: 0; }
</style>
```

- [ ] **Step 2: Wire into App.svelte and ActivityBar**

Edit `src/App.svelte`. Add the import:

```typescript
import ScriptsPanel from "$lib/components/panels/ScriptsPanel.svelte";
```

Update the panel switch:

```svelte
{#if active === "connection"}
  <ConnectionPanel />
{:else if active === "macros"}
  <MacrosPanel />
{:else if active === "filter"}
  <FilterPanel />
{:else if active === "logs"}
  <LogsPanel />
{:else if active === "scripts"}
  <ScriptsPanel />
{/if}
```

(No more "Coming in Plan 3" fallback — all 5 entries now have panels.)

Edit `src/lib/components/ActivityBar.svelte`. Change scripts entry's title:

```typescript
{ key: "scripts",    icon: "{}", title: "Scripts" },
```

- [ ] **Step 3: Type check + build**

```bash
npm run check
npm run build
```

Both pass. Bundle size will jump (~2 MB Monaco chunk).

- [ ] **Step 4: Smoke**

```bash
cargo tauri dev
```

Open Scripts panel. Click "+ New". Editor appears with `-- log('hello from RustCOM Lua')`. Edit. Save. Connect a port. Click Run — your Lua runs. Click Stop.

Try a heartbeat-responder example:
```lua
log("hello")
on_recv = function(bytes)
  log("got " .. #bytes .. " bytes")
  if bytes[1] == 0x52 then  -- 'R'
    serial.send_text("ack\n")
  end
end
```

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/panels/ScriptsPanel.svelte src/App.svelte src/lib/components/ActivityBar.svelte
git commit -m "feat(ui): scripts panel with Monaco editor + run/stop"
```

---

## Task 14: Toast surface

**Files:**
- Create: `src/lib/stores/toasts.ts`
- Create: `src/lib/components/ToastStack.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Toasts store**

`src/lib/stores/toasts.ts`:

```typescript
import { writable, type Writable } from "svelte/store";

export interface Toast {
  id: number;
  level: string;
  msg: string;
  ts: number;
}

let next = 1;

export const toasts: Writable<Toast[]> = writable([]);

export function pushToast(level: string, msg: string, ts: number = Date.now()) {
  const id = next++;
  toasts.update((list) => [...list, { id, level, msg, ts }]);
  setTimeout(() => dismissToast(id), 5000);
}

export function dismissToast(id: number) {
  toasts.update((list) => list.filter((t) => t.id !== id));
}
```

- [ ] **Step 2: ToastStack component**

`src/lib/components/ToastStack.svelte`:

```svelte
<script lang="ts">
  import { toasts, dismissToast } from "$lib/stores/toasts";
</script>

<div class="stack">
  {#each $toasts as t (t.id)}
    <div class="toast {t.level}" onclick={() => dismissToast(t.id)} role="alert">
      <span>{t.msg}</span>
    </div>
  {/each}
</div>

<style>
  .stack { position: fixed; bottom: calc(var(--statusbar-h) + 8px); right: 12px; display: flex; flex-direction: column; gap: 6px; z-index: 100; pointer-events: none; }
  .toast { pointer-events: auto; padding: 8px 12px; border-radius: var(--radius-md); font-size: 11px; cursor: pointer; box-shadow: 0 4px 12px rgba(0,0,0,0.4); max-width: 320px; }
  .toast.info { background: var(--bg-input-hover); color: var(--fg); }
  .toast.warn { background: var(--warn-bg); color: var(--warn); }
  .toast.err, .toast.error { background: var(--err-bg); color: var(--err); }
</style>
```

- [ ] **Step 3: Wire toast events in App.svelte**

Edit `src/App.svelte`. Add imports:

```typescript
import ToastStack from "$lib/components/ToastStack.svelte";
import { onToast } from "$lib/ipc";
import { pushToast } from "$lib/stores/toasts";
```

Add event listener in `onMount` (track unlisten so cleanup works):

```typescript
let unToast: (() => void) | null = null;

onMount(async () => {
  await loadSettings();
  startPolling();
  startEventRouter();
  window.addEventListener("keydown", onWindowKey);
  unToast = await onToast((e) => {
    pushToast(e.payload.level, e.payload.msg, e.payload.ts);
  });
});

onDestroy(() => {
  if (unToast) unToast();
  window.removeEventListener("keydown", onWindowKey);
  stopEventRouter();
  stopPolling();
});
```

Add `<ToastStack />` to the markup (after `<StatusBar ... />`):

```svelte
<StatusBar ... />
<ToastStack />
```

- [ ] **Step 4: Type check + build**

```bash
npm run check
npm run build
```

Both pass.

- [ ] **Step 5: Smoke**

In a Lua script, `ui.toast("hello")` should pop a toast bottom-right that fades after 5 s. Click to dismiss early.

- [ ] **Step 6: Commit**

```bash
git add src/lib/stores/toasts.ts src/lib/components/ToastStack.svelte src/App.svelte
git commit -m "feat(ui): toast surface for ui.toast() from Lua"
```

---

## Task 15: Surface `script_log` in Logs panel

**Files:**
- Modify: `src/lib/eventRouter.ts`
- Modify: `src/lib/components/panels/LogsPanel.svelte`

We append `log(msg)` output into the active tab's log entries (so it shows up next to RX/TX in the buffered count and exports cleanly).

- [ ] **Step 1: Subscribe to script_log in eventRouter**

Edit `src/lib/eventRouter.ts`. Add:

```typescript
import { onScriptLog } from "$lib/ipc";
import { activeTabId } from "$lib/stores/tabs";
import { get } from "svelte/store";
```

In `startEventRouter`, after the `tabs.subscribe(...)` block:

```typescript
let stopLog: Unsub | null = null;
(async () => {
  stopLog = await onScriptLog((e) => {
    const id = get(activeTabId);
    pushLog(id, "rx", new TextEncoder().encode(`[lua] ${e.payload.msg}`), e.payload.ts);
  });
})();
```

Save `stopLog` somewhere reachable from `stopEventRouter` so it can be released. The simplest pattern is to keep it as a module-scoped `let` next to `stopReact`:

```typescript
let stopReact: Unsub | null = null;
let stopLog: Unsub | null = null;
```

Then in `stopEventRouter`:

```typescript
export function stopEventRouter() {
  if (stopReact) { stopReact(); stopReact = null; }
  if (stopLog) { stopLog(); stopLog = null; }
  for (const id of [...subs.keys()]) unsubscribe(id);
}
```

Move the `(async () => { stopLog = await onScriptLog(...); })();` lines inside `startEventRouter` so they run on start.

- [ ] **Step 2: Type check + build**

```bash
npm run check
npm run build
```

Both pass.

- [ ] **Step 3: Smoke**

Run a Lua script with `log("hello")`. Open Logs panel, enable capture if not already. The next entry buffered shows `[lua] hello`. Save log → file contains the line.

- [ ] **Step 4: Commit**

```bash
git add src/lib/eventRouter.ts
git commit -m "feat(ui): surface Lua log() output through the active tab's log buffer"
```

---

## Task 16: Drop `legacy-egui/` + README v0.2.0

**Files:**
- Delete: `legacy-egui/` (entire directory)
- Modify: `README.md`

- [ ] **Step 1: Remove the directory**

```bash
git rm -r legacy-egui
```

- [ ] **Step 2: Replace `README.md`**

```markdown
# RustCOM — COM Port Analyzer

A multi-port serial-port analyzer with embedded Lua scripting, built with Tauri 2 and Svelte 5.

## Features

- **Multi-port tabs** — open many COM ports at once, each with its own buffer, settings, and history.
- **Three views** — ASCII, Hex (16-byte rows with offset + ASCII gutter), or Both. ANSI escape stripping.
- **Send modes** — ASCII with selectable line endings (`None`, `\r`, `\n`, `\r\n`) or raw hex (`AA BB 0D 0A`).
- **Send macros** — named one-shot snippets, optional F1–F12 hotkeys, persisted to disk.
- **Lua scripting** — write scripts in a Monaco editor, run them against the active tab; API includes `serial.send`, `serial.send_text`, `serial.send_hex`, `on_recv`, `log`, `delay`, `ui.toast`.
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
```

- [ ] **Step 3: Commit**

```bash
git add README.md legacy-egui
git commit -m "chore: drop legacy-egui code and refresh README for v0.2.0"
```

---

## Task 17: Bump version to 0.2.0

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `package.json`

- [ ] **Step 1: Bump versions**

In `src-tauri/Cargo.toml`:

```toml
version = "0.2.0"
```

In `src-tauri/tauri.conf.json`:

```json
"version": "0.2.0"
```

In `package.json`:

```json
"version": "0.2.0"
```

- [ ] **Step 2: Build**

```bash
cargo tauri build
```

Expected: artifacts at `src-tauri/target/release/bundle/{nsis/RustCOM_0.2.0_x64-setup.exe, msi/RustCOM_0.2.0_x64_en-US.msi}`.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json package.json
git commit -m "chore: bump version to 0.2.0"
```

---

## Task 18: Final smoke + tag + merge

- [ ] **Step 1: Run dev binary and walk this checklist**

```bash
cargo tauri dev
```

- [ ] Open multi-tab → connect → send/receive on each.
- [ ] Connection panel "Save as default" persists baud/parity/etc. across restart.
- [ ] Auto-reconnect: enable on a tab, yank USB, plug back in — tab reconnects.
- [ ] Macros: create with F2, press F2 outside form fields → fires.
- [ ] Filter: regex hides non-matching lines in ASCII view; HEX bypasses.
- [ ] Logs: capture → save → file written with formatted entries.
- [ ] Ctrl+F: typing matches highlight; prev/next walks them.
- [ ] Scripts: Monaco editor opens, Lua script runs, `log()` appears in Logs panel as `[lua] ...`, `ui.toast()` shows a toast, `on_recv` reacts to incoming bytes.
- [ ] Window resize works; activity-bar collapse-on-active-icon-click works.

- [ ] **Step 2: Tag the milestone**

```bash
git tag v0.2.0
```

- [ ] **Step 3: Merge to main**

```bash
git checkout main
git merge --no-ff tauri-redesign -m "Merge tauri-redesign into main: full Tauri 2 + Svelte 5 rewrite"
git tag plan-3-complete
```

(Don't `git push` unless you explicitly want to publish — leave that decision to the human.)

- [ ] **Step 4: Verify post-merge build**

```bash
cargo tauri build
```

Same artifacts produced from `main`.

---

## Out of scope (future v2 if requested)

- **Plot/graph view of numeric RX** — real-time plotting of incoming numeric data.
- **Multi-tab Lua scripting** — `tabs[id]:send(...)` etc. v1 binds the script to one tab.
- **Session restore** — remember open tabs at the time of close.
- **File transfer** (XMODEM/YMODEM/ZMODEM).
- **Built-in protocol decoders** (Modbus RTU, etc.).
