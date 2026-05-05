# RustCOM Tauri Redesign — Plan 2: Multi-Tab + Macros + Filter + Logs + Search

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Lift the single-tab assumption, then fill in four of the five activity-bar panels (Macros, Scripts is Plan 3, Filter, Logs) plus an in-terminal Ctrl+F search overlay. After Plan 2, RustCOM is a real multi-port serial workbench.

**Architecture:** Frontend takes most of the work — `tabs.ts` becomes a real multi-tab store, components stop hardcoding `SOLE_TAB_ID`, an event-router subscribes to per-tab Tauri events as tabs come and go. Backend grows two new modules: `storage.rs` (resolves the OS user-data dir + atomic JSON I/O, used for macros now and scripts/settings later) and `commands/macros.rs` (CRUD on macros). Filter and search are pure-frontend display-time transforms.

**Tech Stack:** Same as Plan 1 — Rust + Tauri 2 + Svelte 5 + TypeScript + Vite + Vitest. Adds `tauri-plugin-dialog` for native file pickers (used by Save Log).

**Spec:** `docs/superpowers/specs/2026-05-04-tauri-redesign-design.md`
**Predecessor plan:** `docs/superpowers/plans/2026-05-04-tauri-redesign-plan-1-foundations.md`

**Working branch:** continue on `tauri-redesign` (don't create a new branch — Plan 1 hasn't been merged yet, Plan 2 piles on top).

**Working directory:** `C:\Users\WilliamHerr\Desktop\Code\RustCOM`. Commands use forward slashes for path safety on PowerShell + bash.

**Testing philosophy:** Same as Plan 1. Pure logic (multi-tab store mutations, filter line matching, search match indexing, macro persistence) is unit-tested. UI rendering and IO are verified by running.

---

## Plan 2 task overview

| # | Task | Verification |
|---|---|---|
| 1 | Multi-tab `tabs.ts` refactor (allocator, add/close/switch helpers, history → per-tab via id) | Vitest |
| 2 | Update `Terminal.svelte` to use active tab id (and react to switch) | Manual: switch tabs, RX still routes |
| 3 | Update `SendRow.svelte` to use active tab id | Manual |
| 4 | Update `ConnectionPanel.svelte` for active-tab semantics | Manual |
| 5 | Per-tab event router (`lib/eventRouter.ts`) — subscribe per tab, route into store | Manual: open 2 ports, both stream |
| 6 | New `TabStrip.svelte` — list / switch / close / `+` | Manual |
| 7 | Backend `storage.rs` + tests | `cargo test` |
| 8 | Backend `commands/macros.rs` + register | Smoke + commands callable |
| 9 | Frontend `stores/macros.ts` | Vitest (mocked ipc) |
| 10 | `MacrosPanel.svelte` (list + add/edit/delete inline) | Manual |
| 11 | Global hotkey capture (F1–F12) in `App.svelte` | Manual: assign + press |
| 12 | Per-tab filter fields + helper + tests | Vitest |
| 13 | `FilterPanel.svelte` | Manual |
| 14 | Apply filter in `Terminal.svelte` | Manual + visual diff |
| 15 | Backend `commands/logs.rs` (save formatted log) | Smoke |
| 16 | Per-tab log entries + helpers | Vitest |
| 17 | `LogsPanel.svelte` (toggle + native save dialog) | Manual |
| 18 | Per-tab search overlay state + match finder + tests | Vitest |
| 19 | `SearchOverlay.svelte` (Ctrl+F triggered) | Manual |
| 20 | Match highlighting in `Terminal.svelte` | Manual |
| 21 | `cargo tauri build` + smoke checklist + tag `plan-2-panels` | Build artifact |

---

## Task 1: Multi-tab `tabs.ts` refactor

**Files:**
- Modify: `src/lib/stores/tabs.ts`
- Modify: `src/lib/stores/tabs.test.ts`

**Note:** Plan 1's `SOLE_TAB_ID` constant goes away. Tabs are allocated by a monotonically-increasing counter starting at 1.

- [ ] **Step 1: Replace SOLE_TAB_ID with an allocator and add tab CRUD**

Edit `src/lib/stores/tabs.ts`. Replace the `SOLE_TAB_ID` constant and the initial `tabs.set([defaultTab()])` setup with this:

```typescript
let nextTabId: TabId = 1;

function allocTabId(): TabId {
  return nextTabId++;
}

function defaultTab(id: TabId = allocTabId()): Tab {
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
    sendMode: "ascii",
    lineEnding: "crlf",
    dtr: false,
    rts: false,
    errorMessage: null,
    history: [],
    historyIndex: -1,
  };
}

export const tabs: Writable<Tab[]> = writable([defaultTab()]);
export const activeTabId: Writable<TabId> = writable(1);

export function addTab(): TabId {
  const tab = defaultTab();
  tabs.update((list) => [...list, tab]);
  activeTabId.set(tab.id);
  return tab.id;
}

export function closeTab(id: TabId) {
  tabs.update((list) => {
    const next = list.filter((t) => t.id !== id);
    return next.length === 0 ? [defaultTab()] : next;
  });
  // If the closed tab was active, switch to the last remaining tab.
  const current = get(activeTabId);
  if (current === id) {
    const list = get(tabs);
    activeTabId.set(list[list.length - 1].id);
  }
}

export function setActiveTab(id: TabId) {
  activeTabId.set(id);
}

export function getTab(id: TabId): Tab | undefined {
  return get(tabs).find((t) => t.id === id);
}
```

Remove the line `export const SOLE_TAB_ID: TabId = 1;` (callers will be updated in Tasks 2–4).

Update `patchActiveTab`, `appendBytes`, `clearBuffer`, `pushHistory`, `navigateHistory` — none need behavior changes, but `appendBytes` and the history helpers already take an `id` parameter, so they're fine.

- [ ] **Step 2: Update tests for the new shape**

Edit `src/lib/stores/tabs.test.ts`. Replace the `import` block at the top to drop `SOLE_TAB_ID` and add the new helpers:

```typescript
import { describe, it, expect, beforeEach } from "vitest";
import { get } from "svelte/store";
import {
  tabs,
  activeTabId,
  activeTab,
  appendBytes,
  clearBuffer,
  MAX_BUFFER_SIZE,
  BUFFER_DRAIN_SIZE,
  pushHistory,
  navigateHistory,
  addTab,
  closeTab,
  setActiveTab,
  getTab,
  type Tab,
} from "./tabs";
```

Replace the `beforeEach` setup so it resets to a single fresh tab with id 1:

```typescript
beforeEach(() => {
  tabs.set([
    {
      id: 1,
      config: {
        port_name: "",
        baud_rate: 9600,
        data_bits: "eight",
        stop_bits: "one",
        parity: "none",
        flow_control: "none",
      },
      state: "disconnected",
      buffer: new Uint8Array(0),
      rxBytes: 0,
      txBytes: 0,
      viewMode: "ascii",
      stripAnsi: true,
      autoScroll: true,
      sendMode: "ascii",
      lineEnding: "crlf",
      dtr: false,
      rts: false,
      errorMessage: null,
      history: [],
      historyIndex: -1,
    },
  ]);
  activeTabId.set(1);
});
```

Replace every `SOLE_TAB_ID` usage in the existing tests with the literal `1` (TabId).

Add these new tests at the end of the `describe` block:

```typescript
it("addTab appends and switches active", () => {
  const before = get(tabs).length;
  const newId = addTab();
  expect(get(tabs).length).toBe(before + 1);
  expect(get(activeTabId)).toBe(newId);
  expect(newId).toBeGreaterThan(1);
});

it("closeTab removes by id and falls back to last tab", () => {
  const a = addTab();
  const b = addTab();
  expect(get(activeTabId)).toBe(b);
  closeTab(b);
  expect(get(tabs).map((t: Tab) => t.id)).toEqual([1, a]);
  expect(get(activeTabId)).toBe(a);
});

it("closeTab on the last tab keeps a fresh single tab", () => {
  closeTab(1);
  expect(get(tabs).length).toBe(1);
  expect(get(tabs)[0].id).toBeGreaterThan(1); // a new fresh one was minted
});

it("setActiveTab switches", () => {
  const newId = addTab();
  setActiveTab(1);
  expect(get(activeTabId)).toBe(1);
  setActiveTab(newId);
  expect(get(activeTabId)).toBe(newId);
});

it("getTab finds by id", () => {
  expect(getTab(1)?.id).toBe(1);
  expect(getTab(999)).toBeUndefined();
});
```

- [ ] **Step 3: Run tests + check + build**

```bash
npm run test
npm run check
```

Both must pass. (`npm run build` will fail because Tasks 2–4 haven't migrated callers yet — skip until Task 4 completes.)

- [ ] **Step 4: Commit**

```bash
git add src/lib/stores/tabs.ts src/lib/stores/tabs.test.ts
git commit -m "refactor(ui): tabs store supports multiple tabs"
```

---

## Task 2: Update `Terminal.svelte` to use active tab id

**Files:**
- Modify: `src/lib/components/Terminal.svelte`

**Note:** This task only changes how Terminal subscribes to events and renders. The actual cross-tab event router lands in Task 5; this task just stops hardcoding `SOLE_TAB_ID` for the bookkeeping calls (`appendBytes`, `clearBuffer`, `patchActiveTab`).

- [ ] **Step 1: Replace `SOLE_TAB_ID` references with `$activeTabId`**

Edit `src/lib/components/Terminal.svelte`:

1. Update the imports — remove `SOLE_TAB_ID`, add `activeTabId`:

```typescript
import { activeTab, activeTabId, appendBytes, clearBuffer, patchActiveTab } from "$lib/stores/tabs";
```

2. **Remove the `onMount` block entirely** — event subscription moves to the dedicated event router in Task 5. The Terminal's only job is to render the active tab's buffer.

3. Remove the now-unused imports from `$lib/ipc` (`onRx`, `onTx`, `onDisconnect`, `RxPayload`, `TxPayload`, `DisconnectPayload`, `ipc`). After this task the import line becomes:

```typescript
import { bytesToText, formatHex } from "$lib/format";
```

4. Remove the `onMount` import (`import { onMount } from "svelte";`) if it's no longer used elsewhere in the file.

5. Update the `clearBuffer` call:

```svelte
<button onclick={() => clearBuffer($activeTabId)}>Clear</button>
```

(Auto-scroll effect and the `$derived.by` `display` block stay as-is — they read from `$activeTab` which already follows the active tab id.)

- [ ] **Step 2: Type check + build**

```bash
npm run check
```

Expected: 0 errors (a11y warnings on ConnectionPanel still allowed).

`npm run build` may fail because the rest of the code still references `SOLE_TAB_ID`. We'll resolve that when Tasks 3–4 land. For now `npm run check` is sufficient verification.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/Terminal.svelte
git commit -m "refactor(ui): Terminal reads/writes via active tab id"
```

---

## Task 3: Update `SendRow.svelte` to use active tab id

**Files:**
- Modify: `src/lib/components/SendRow.svelte`

- [ ] **Step 1: Replace SOLE_TAB_ID with activeTabId**

Edit `src/lib/components/SendRow.svelte`. Update the imports:

```typescript
import { activeTab, activeTabId, navigateHistory, patchActiveTab, pushHistory } from "$lib/stores/tabs";
import { ipc, type LineEnding, type SendMode } from "$lib/ipc";
```

Replace every `SOLE_TAB_ID` literal in the file with `$activeTabId`. There should be five occurrences:
- `ipc.sendText($activeTabId, value, tab.lineEnding)`
- `ipc.sendHex($activeTabId, value)`
- `pushHistory($activeTabId, value)`
- `navigateHistory($activeTabId, "up")`
- `navigateHistory($activeTabId, "down")`

- [ ] **Step 2: Type check**

```bash
npm run check
```

Expected: 0 errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/SendRow.svelte
git commit -m "refactor(ui): SendRow reads/writes via active tab id"
```

---

## Task 4: Update `ConnectionPanel.svelte` for active-tab semantics

**Files:**
- Modify: `src/lib/components/panels/ConnectionPanel.svelte`

- [ ] **Step 1: Replace SOLE_TAB_ID with activeTabId**

Edit `src/lib/components/panels/ConnectionPanel.svelte`. Update the imports:

```typescript
import { activeTab, activeTabId, patchActiveTab } from "$lib/stores/tabs";
```

Replace every `SOLE_TAB_ID` literal in the file with `$activeTabId`. There are four occurrences (`ipc.openPort`, `ipc.closePort`, `ipc.setDtr`, `ipc.setRts`).

- [ ] **Step 2: Type check + build**

```bash
npm run check
npm run build
```

Both must pass now. `SOLE_TAB_ID` should appear nowhere in `src/`.

Verify:

```bash
grep -r "SOLE_TAB_ID" src/
```

Expected: no matches.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/panels/ConnectionPanel.svelte
git commit -m "refactor(ui): ConnectionPanel reads/writes via active tab id"
```

---

## Task 5: Per-tab event router

**Files:**
- Create: `src/lib/eventRouter.ts`
- Modify: `src/App.svelte`

**Note:** The router subscribes to RX/TX/disconnect events for every tab in the `tabs` store. When tabs are added or removed, it adjusts subscriptions. This way each open port keeps streaming into its tab's buffer regardless of which tab is currently visible.

- [ ] **Step 1: Write the router**

`src/lib/eventRouter.ts`:

```typescript
import { get } from "svelte/store";
import { tabs, appendBytes, patchActiveTab, type Tab } from "$lib/stores/tabs";
import {
  ipc,
  onRx,
  onTx,
  onDisconnect,
  type RxPayload,
  type TxPayload,
  type DisconnectPayload,
  type TabId,
} from "$lib/ipc";

type Unsub = () => void;

const subs = new Map<TabId, Unsub[]>();

async function subscribe(id: TabId) {
  if (subs.has(id)) return;

  const unrx = await onRx(id, (event) => {
    const p = event.payload as RxPayload;
    appendBytes(id, new Uint8Array(p.bytes), "rx");
  });
  const untx = await onTx(id, (event) => {
    const p = event.payload as TxPayload;
    appendBytes(id, new Uint8Array(p.bytes), "tx");
  });
  const undisc = await onDisconnect(id, (event) => {
    const p = event.payload as DisconnectPayload;
    // We patch any tab matching the id, not just the active one.
    tabs.update((list) =>
      list.map((t) =>
        t.id === id
          ? { ...t, state: "disconnected", errorMessage: p.reason, dtr: false, rts: false }
          : t
      )
    );
    ipc.closePort(id).catch(() => undefined);
  });

  subs.set(id, [unrx, untx, undisc]);
}

function unsubscribe(id: TabId) {
  const list = subs.get(id);
  if (!list) return;
  list.forEach((u) => u());
  subs.delete(id);
}

let stopReact: Unsub | null = null;

export function startEventRouter() {
  if (stopReact) return;

  // Subscribe for whatever tabs are present right now.
  for (const t of get(tabs)) void subscribe(t.id);

  stopReact = tabs.subscribe((list: Tab[]) => {
    const present = new Set(list.map((t) => t.id));
    // Add subs for new tabs.
    for (const t of list) if (!subs.has(t.id)) void subscribe(t.id);
    // Drop subs for closed tabs.
    for (const id of [...subs.keys()]) if (!present.has(id)) unsubscribe(id);
  });
}

export function stopEventRouter() {
  if (stopReact) {
    stopReact();
    stopReact = null;
  }
  for (const id of [...subs.keys()]) unsubscribe(id);
}
```

- [ ] **Step 2: Wire it in App.svelte**

Edit `src/App.svelte`. In the `<script>` block, add the router import alongside the ports import:

```typescript
import { startPolling, stopPolling } from "$lib/stores/ports";
import { startEventRouter, stopEventRouter } from "$lib/eventRouter";
```

Update the lifecycle hooks:

```typescript
onMount(() => {
  startPolling();
  startEventRouter();
});
onDestroy(() => {
  stopEventRouter();
  stopPolling();
});
```

- [ ] **Step 3: Type check + build**

```bash
npm run check
npm run build
```

Both pass.

- [ ] **Step 4: Commit**

```bash
git add src/lib/eventRouter.ts src/App.svelte
git commit -m "feat(ui): per-tab event router subscribes to RX/TX/disconnect dynamically"
```

---

## Task 6: New `TabStrip.svelte` — list / switch / close / `+`

**Files:**
- Replace: `src/lib/components/TabStrip.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Write the new TabStrip**

`src/lib/components/TabStrip.svelte`:

```svelte
<script lang="ts">
  import { tabs, activeTabId, addTab, closeTab, setActiveTab } from "$lib/stores/tabs";
  import { ipc } from "$lib/ipc";
  import type { TabId } from "$lib/ipc";

  function labelFor(id: TabId): string {
    const t = $tabs.find((x) => x.id === id);
    if (!t) return "—";
    if (t.state === "connected" && t.config.port_name) return t.config.port_name;
    return `Tab ${id}`;
  }

  async function onClose(e: MouseEvent, id: TabId) {
    e.stopPropagation();
    // Best-effort disconnect; ignore errors (port may not be open).
    await ipc.closePort(id).catch(() => undefined);
    closeTab(id);
  }
</script>

<div class="strip">
  {#each $tabs as t (t.id)}
    <div
      class="tab"
      class:active={t.id === $activeTabId}
      onclick={() => setActiveTab(t.id)}
      role="tab"
      tabindex="0"
      onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") setActiveTab(t.id); }}
    >
      {#if t.state === "connected"}
        <span class="dot ok"></span>
      {:else if t.state === "connecting" || t.state === "reconnecting"}
        <span class="dot warn"></span>
      {:else}
        <span class="dot muted"></span>
      {/if}
      <span class="label">{labelFor(t.id)}</span>
      <button class="close" onclick={(e) => onClose(e, t.id)} title="Close tab">×</button>
    </div>
  {/each}
  <button class="add" onclick={addTab} title="New tab">+</button>
</div>

<style>
  .strip {
    display: flex;
    gap: 2px;
    padding: 6px 8px 0;
    background: var(--bg);
    border-bottom: 1px solid var(--border);
    align-items: center;
    overflow-x: auto;
  }
  .tab {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px 4px 10px;
    font-size: 11px;
    border-radius: var(--radius-md) var(--radius-md) 0 0;
    color: var(--fg-muted);
    background: transparent;
    cursor: pointer;
    user-select: none;
  }
  .tab:hover { background: var(--bg-input); color: var(--fg); }
  .tab.active { background: var(--bg-elevated); color: var(--fg); }
  .label { white-space: nowrap; }
  .dot { width: 6px; height: 6px; border-radius: 50%; display: inline-block; }
  .dot.ok { background: var(--ok); }
  .dot.warn { background: var(--warn); }
  .dot.muted { background: var(--fg-subtle); }
  .close {
    background: transparent;
    color: var(--fg-subtle);
    padding: 0 4px;
    font-size: 13px;
    line-height: 1;
    border-radius: 3px;
  }
  .close:hover { background: var(--err-bg); color: var(--err); }
  .add {
    background: transparent;
    color: var(--fg-subtle);
    padding: 4px 8px;
    font-size: 12px;
    border-radius: var(--radius-md);
  }
  .add:hover { background: var(--bg-input); color: var(--fg); }
</style>
```

- [ ] **Step 2: Update App.svelte to drop the TabStrip props**

Edit `src/App.svelte`. The current `<TabStrip ... />` passes `label` and `active` props that no longer exist. Replace with:

```svelte
<TabStrip />
```

- [ ] **Step 3: Type check + build**

```bash
npm run check
npm run build
```

Both pass.

- [ ] **Step 4: Manual smoke**

Run `cargo tauri dev`. Open multiple tabs with `+`, connect them to different ports, send/receive on each, switch between them, close them. Verify:
- Each tab keeps its own buffer.
- Disconnecting one tab doesn't kill the other.
- The active-tab indicator (highlighted background) tracks `activeTabId`.
- Closing the only tab leaves a fresh blank tab (doesn't leave the app empty).

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/TabStrip.svelte src/App.svelte
git commit -m "feat(ui): real multi-tab strip with add/switch/close"
```

---

## Task 7: Backend `storage.rs` + tests

**Files:**
- Create: `src-tauri/src/storage.rs`
- Modify: `src-tauri/src/lib.rs` (add `pub mod storage;`)

- [ ] **Step 1: Add the `dirs` crate**

In `src-tauri/Cargo.toml`, add to `[dependencies]`:

```toml
dirs = "5"
```

Run `cargo build` from `src-tauri/` to download.

- [ ] **Step 2: Write storage.rs**

`src-tauri/src/storage.rs`:

```rust
use std::fs;
use std::path::PathBuf;

use serde::{de::DeserializeOwned, Serialize};

use crate::error::{AppError, AppResult};

pub fn data_dir() -> AppResult<PathBuf> {
    let base = dirs::data_local_dir()
        .ok_or_else(|| AppError::Io("could not resolve user data dir".to_string()))?;
    let dir = base.join("rustcom");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| AppError::Io(e.to_string()))?;
    }
    Ok(dir)
}

pub fn read_json<T: DeserializeOwned + Default>(file_name: &str) -> AppResult<T> {
    let path = data_dir()?.join(file_name);
    if !path.exists() {
        return Ok(T::default());
    }
    let body = fs::read_to_string(&path).map_err(|e| AppError::Io(e.to_string()))?;
    serde_json::from_str::<T>(&body).map_err(|e| AppError::Invalid(e.to_string()))
}

pub fn write_json<T: Serialize>(file_name: &str, value: &T) -> AppResult<()> {
    let dir = data_dir()?;
    let final_path = dir.join(file_name);
    let tmp_path = dir.join(format!("{file_name}.tmp"));

    let body = serde_json::to_string_pretty(value).map_err(|e| AppError::Invalid(e.to_string()))?;
    fs::write(&tmp_path, body).map_err(|e| AppError::Io(e.to_string()))?;
    fs::rename(&tmp_path, &final_path).map_err(|e| AppError::Io(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Default, Serialize, Deserialize, PartialEq, Debug)]
    struct Sample { name: String, count: u32 }

    #[test]
    fn read_returns_default_when_file_missing() {
        // Use a unique file name so we don't collide with real data on the dev machine.
        let _: Sample = read_json("rustcom_test_missing_xyz_42.json").unwrap();
    }

    #[test]
    fn round_trip_value() {
        let v = Sample { name: "alpha".into(), count: 7 };
        write_json("rustcom_test_roundtrip.json", &v).unwrap();
        let back: Sample = read_json("rustcom_test_roundtrip.json").unwrap();
        assert_eq!(back, v);
        // Cleanup so repeated runs stay clean.
        let _ = fs::remove_file(data_dir().unwrap().join("rustcom_test_roundtrip.json"));
    }
}
```

- [ ] **Step 3: Register the module**

Edit `src-tauri/src/lib.rs`. Add to the module decls:

```rust
pub mod storage;
```

- [ ] **Step 4: Test**

```bash
cd src-tauri
cargo test --lib storage
```

Expected: 2 tests pass.

- [ ] **Step 5: Commit**

```bash
cd ..
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/storage.rs src-tauri/src/lib.rs
git commit -m "feat(backend): storage module with atomic JSON I/O in user data dir"
```

---

## Task 8: Backend `commands/macros.rs` + register

**Files:**
- Create: `src-tauri/src/commands/macros.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the macros module**

`src-tauri/src/commands/macros.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::port::config::LineEnding;
use crate::storage;

const FILE: &str = "macros.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MacroMode { Ascii, Hex }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macro {
    pub id: String,
    pub name: String,
    pub mode: MacroMode,
    pub payload: String,
    #[serde(default)]
    pub line_ending: Option<LineEnding>,
    #[serde(default)]
    pub hotkey: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct MacroFile {
    #[serde(default)]
    macros: Vec<Macro>,
}

#[tauri::command]
pub fn list_macros() -> AppResult<Vec<Macro>> {
    let file: MacroFile = storage::read_json(FILE)?;
    Ok(file.macros)
}

#[tauri::command]
pub fn save_macro(item: Macro) -> AppResult<Vec<Macro>> {
    let mut file: MacroFile = storage::read_json(FILE)?;
    if item.id.is_empty() {
        return Err(AppError::Invalid("macro id is empty".to_string()));
    }
    if let Some(slot) = file.macros.iter_mut().find(|m| m.id == item.id) {
        *slot = item;
    } else {
        file.macros.push(item);
    }
    storage::write_json(FILE, &file)?;
    Ok(file.macros)
}

#[tauri::command]
pub fn delete_macro(id: String) -> AppResult<Vec<Macro>> {
    let mut file: MacroFile = storage::read_json(FILE)?;
    file.macros.retain(|m| m.id != id);
    storage::write_json(FILE, &file)?;
    Ok(file.macros)
}
```

- [ ] **Step 2: Register the module**

Edit `src-tauri/src/commands/mod.rs`:

```rust
pub mod macros;
pub mod ports;
```

Edit `src-tauri/src/lib.rs`. Append the three macro commands to the `tauri::generate_handler!` list:

```rust
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
        ])
```

- [ ] **Step 3: Build**

```bash
cd src-tauri
cargo build
```

Expected: clean.

- [ ] **Step 4: Commit**

```bash
cd ..
git add src-tauri/src/commands
git commit -m "feat(backend): macro CRUD commands with macros.json persistence"
```

---

## Task 9: Frontend `stores/macros.ts`

**Files:**
- Create: `src/lib/stores/macros.ts`
- Modify: `src/lib/ipc.ts`
- Test: `src/lib/stores/macros.test.ts`

- [ ] **Step 1: Add Macro types and ipc wrappers**

Edit `src/lib/ipc.ts`. Append after the existing types:

```typescript
export type MacroMode = "ascii" | "hex";

export interface Macro {
  id: string;
  name: string;
  mode: MacroMode;
  payload: string;
  line_ending?: LineEnding | null;
  hotkey?: string | null;
}
```

In the `ipc` const, add three more methods:

```typescript
  listMacros: () => invoke<Macro[]>("list_macros"),
  saveMacro: (item: Macro) => invoke<Macro[]>("save_macro", { item }),
  deleteMacro: (id: string) => invoke<Macro[]>("delete_macro", { id }),
```

- [ ] **Step 2: Write the macros store**

`src/lib/stores/macros.ts`:

```typescript
import { writable, type Writable } from "svelte/store";
import { ipc, type Macro } from "$lib/ipc";

export const macros: Writable<Macro[]> = writable([]);
export const macrosLoaded: Writable<boolean> = writable(false);

export async function loadMacros() {
  try {
    const list = await ipc.listMacros();
    macros.set(list);
  } catch {
    macros.set([]);
  } finally {
    macrosLoaded.set(true);
  }
}

export async function upsertMacro(item: Macro) {
  const next = await ipc.saveMacro(item);
  macros.set(next);
}

export async function removeMacro(id: string) {
  const next = await ipc.deleteMacro(id);
  macros.set(next);
}

export function newMacroId(): string {
  return `m_${Date.now().toString(36)}_${Math.floor(Math.random() * 1e6).toString(36)}`;
}
```

- [ ] **Step 3: Tests**

`src/lib/stores/macros.test.ts`:

```typescript
import { describe, it, expect, vi, beforeEach } from "vitest";
import { get } from "svelte/store";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => invokeMock(...args) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(() => Promise.resolve(() => undefined)) }));

import { macros, loadMacros, upsertMacro, removeMacro, newMacroId } from "./macros";

beforeEach(() => {
  invokeMock.mockReset();
  macros.set([]);
});

describe("macros store", () => {
  it("loadMacros populates from list_macros", async () => {
    invokeMock.mockResolvedValueOnce([{ id: "m1", name: "Reset", mode: "ascii", payload: "RESET" }]);
    await loadMacros();
    expect(get(macros).map((m) => m.id)).toEqual(["m1"]);
  });

  it("upsertMacro replaces store with response", async () => {
    invokeMock.mockResolvedValueOnce([{ id: "m1", name: "Reset", mode: "ascii", payload: "RESET" }]);
    await upsertMacro({ id: "m1", name: "Reset", mode: "ascii", payload: "RESET" });
    expect(invokeMock).toHaveBeenCalledWith("save_macro", { item: expect.any(Object) });
    expect(get(macros).length).toBe(1);
  });

  it("removeMacro replaces store with response", async () => {
    invokeMock.mockResolvedValueOnce([]);
    await removeMacro("m1");
    expect(invokeMock).toHaveBeenCalledWith("delete_macro", { id: "m1" });
    expect(get(macros)).toEqual([]);
  });

  it("newMacroId emits unique strings", () => {
    const a = newMacroId();
    const b = newMacroId();
    expect(a).not.toBe(b);
    expect(a.startsWith("m_")).toBe(true);
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
git add src/lib/ipc.ts src/lib/stores/macros.ts src/lib/stores/macros.test.ts
git commit -m "feat(ui): macros store backed by Tauri commands"
```

---

## Task 10: `MacrosPanel.svelte` (list + add/edit/delete inline)

**Files:**
- Create: `src/lib/components/panels/MacrosPanel.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Write the panel**

`src/lib/components/panels/MacrosPanel.svelte`:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { macros, loadMacros, upsertMacro, removeMacro, newMacroId } from "$lib/stores/macros";
  import { activeTab, activeTabId, patchActiveTab } from "$lib/stores/tabs";
  import { ipc, type LineEnding, type Macro } from "$lib/ipc";

  const HOTKEYS = ["", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12"];

  let editingId = $state<string | null>(null);
  let draft = $state<Macro>(blankMacro());

  function blankMacro(): Macro {
    return { id: newMacroId(), name: "", mode: "ascii", payload: "", line_ending: "crlf", hotkey: null };
  }

  onMount(() => { void loadMacros(); });

  function startNew() {
    draft = blankMacro();
    editingId = draft.id;
  }

  function startEdit(m: Macro) {
    draft = { ...m };
    editingId = m.id;
  }

  async function save() {
    if (!draft.name.trim()) {
      patchActiveTab({ errorMessage: "Macro needs a name" });
      return;
    }
    await upsertMacro(draft);
    editingId = null;
  }

  async function del(id: string) {
    await removeMacro(id);
    if (editingId === id) editingId = null;
  }

  async function run(m: Macro) {
    if ($activeTab.state !== "connected") {
      patchActiveTab({ errorMessage: "Connect a port before running a macro" });
      return;
    }
    try {
      if (m.mode === "ascii") {
        await ipc.sendText($activeTabId, m.payload, (m.line_ending ?? "none") as LineEnding);
      } else {
        await ipc.sendHex($activeTabId, m.payload);
      }
    } catch (e) {
      patchActiveTab({ errorMessage: String(e) });
    }
  }
</script>

<div class="panel">
  <div class="header">
    <h4>Macros</h4>
    <button class="add" onclick={startNew}>+ New</button>
  </div>

  {#if $macros.length === 0}
    <p class="empty">No macros yet. Click <em>+ New</em> to add one.</p>
  {/if}

  {#each $macros as m (m.id)}
    {#if editingId === m.id}
      <div class="edit">
        <input placeholder="Name" bind:value={draft.name} />
        <div class="row">
          <select bind:value={draft.mode}>
            <option value="ascii">ASCII</option>
            <option value="hex">Hex</option>
          </select>
          {#if draft.mode === "ascii"}
            <select value={draft.line_ending ?? "none"} onchange={(e) => (draft.line_ending = (e.currentTarget as HTMLSelectElement).value as LineEnding)}>
              <option value="none">None</option>
              <option value="cr">\r</option>
              <option value="lf">\n</option>
              <option value="crlf">\r\n</option>
            </select>
          {/if}
          <select value={draft.hotkey ?? ""} onchange={(e) => (draft.hotkey = (e.currentTarget as HTMLSelectElement).value || null)}>
            {#each HOTKEYS as k}
              <option value={k}>{k || "no hotkey"}</option>
            {/each}
          </select>
        </div>
        <textarea rows="2" bind:value={draft.payload} placeholder={draft.mode === "ascii" ? "Message…" : "AA BB 0D 0A"}></textarea>
        <div class="row gap">
          <button class="primary" onclick={save}>Save</button>
          <button onclick={() => (editingId = null)}>Cancel</button>
          <button class="danger" onclick={() => del(m.id)}>Delete</button>
        </div>
      </div>
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
    <div class="edit">
      <input placeholder="Name" bind:value={draft.name} />
      <div class="row">
        <select bind:value={draft.mode}>
          <option value="ascii">ASCII</option>
          <option value="hex">Hex</option>
        </select>
        {#if draft.mode === "ascii"}
          <select value={draft.line_ending ?? "none"} onchange={(e) => (draft.line_ending = (e.currentTarget as HTMLSelectElement).value as LineEnding)}>
            <option value="none">None</option>
            <option value="cr">\r</option>
            <option value="lf">\n</option>
            <option value="crlf">\r\n</option>
          </select>
        {/if}
        <select value={draft.hotkey ?? ""} onchange={(e) => (draft.hotkey = (e.currentTarget as HTMLSelectElement).value || null)}>
          {#each HOTKEYS as k}
            <option value={k}>{k || "no hotkey"}</option>
          {/each}
        </select>
      </div>
      <textarea rows="2" bind:value={draft.payload} placeholder={draft.mode === "ascii" ? "Message…" : "AA BB 0D 0A"}></textarea>
      <div class="row gap">
        <button class="primary" onclick={save}>Save</button>
        <button onclick={() => (editingId = null)}>Cancel</button>
      </div>
    </div>
  {/if}
</div>

<style>
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

  .edit { display: flex; flex-direction: column; gap: 6px; padding: 8px; background: var(--bg-input); border-radius: var(--radius-md); border: 1px solid var(--accent); }
  .edit input, .edit select, .edit textarea { font-family: var(--font-mono); font-size: 11px; }
  .edit textarea { resize: vertical; min-height: 36px; padding: 6px 8px; border-radius: var(--radius-md); background: var(--bg); color: var(--fg); border: 1px solid transparent; }
  .edit textarea:focus { border-color: var(--accent); outline: none; }
  .row { display: flex; gap: 4px; }
  .row.gap { gap: 6px; }
  .row select { flex: 1; }
</style>
```

- [ ] **Step 2: Wire it into App.svelte's panel switch**

Edit `src/App.svelte`. Add the import:

```typescript
import MacrosPanel from "$lib/components/panels/MacrosPanel.svelte";
```

Update the panel switch in the `<aside class="side">` block:

```svelte
<aside class="side">
  {#if active === "connection"}
    <ConnectionPanel />
  {:else if active === "macros"}
    <MacrosPanel />
  {:else}
    <div class="muted-pad">Coming in Plan 3.</div>
  {/if}
</aside>
```

(Filter / Logs panels land later in this plan and will be added to this switch then.)

- [ ] **Step 3: Update ActivityBar tooltip**

Edit `src/lib/components/ActivityBar.svelte`. Change the macros entry's `title`:

```typescript
{ key: "macros",     icon: "▶", title: "Macros" },
```

(Drop the "(Plan 2)" tag.)

- [ ] **Step 4: Type check + build**

```bash
npm run check
npm run build
```

Both pass.

- [ ] **Step 5: Manual smoke**

`cargo tauri dev`. Click the Macros icon in the activity bar. Click "+ New", enter a name + payload, save. Edit. Delete. Restart the app — macros persist.

- [ ] **Step 6: Commit**

```bash
git add src/lib/components/panels/MacrosPanel.svelte src/lib/components/ActivityBar.svelte src/App.svelte
git commit -m "feat(ui): macros panel — list/edit/delete + run against active tab"
```

---

## Task 11: Global hotkey capture (F1–F12)

**Files:**
- Modify: `src/App.svelte`

- [ ] **Step 1: Add a window keydown listener**

Edit `src/App.svelte`. Add to imports:

```typescript
import { macros } from "$lib/stores/macros";
import { get } from "svelte/store";
```

Add a function inside the `<script>` block:

```typescript
function onWindowKey(e: KeyboardEvent) {
  // F1–F12 trigger macros if assigned. Skip when focus is in an input/textarea
  // so users can still rebind in OS-controlled forms etc.
  if (!/^F([1-9]|1[0-2])$/.test(e.key)) return;
  const tag = (e.target as HTMLElement | null)?.tagName?.toLowerCase();
  if (tag === "input" || tag === "textarea" || tag === "select") return;

  const m = get(macros).find((x) => x.hotkey === e.key);
  if (!m) return;
  e.preventDefault();
  void runMacro(m);
}

async function runMacro(m: { mode: "ascii" | "hex"; payload: string; line_ending?: import("$lib/ipc").LineEnding | null }) {
  const tab = get(activeTab);
  if (tab.state !== "connected") return;
  const { ipc } = await import("$lib/ipc");
  const id = get(activeTabId);
  if (m.mode === "ascii") {
    await ipc.sendText(id, m.payload, (m.line_ending ?? "none"));
  } else {
    await ipc.sendHex(id, m.payload);
  }
}
```

Also add `import { activeTabId } from "$lib/stores/tabs";` to the existing tabs import line if it's not already there.

In `onMount`, add the listener:

```typescript
onMount(() => {
  startPolling();
  startEventRouter();
  window.addEventListener("keydown", onWindowKey);
});
onDestroy(() => {
  window.removeEventListener("keydown", onWindowKey);
  stopEventRouter();
  stopPolling();
});
```

- [ ] **Step 2: Type check**

```bash
npm run check
```

Expected: 0 errors.

- [ ] **Step 3: Manual smoke**

Assign F1 to a macro in the Macros panel. Connect a port. Press F1 (with focus NOT in the send input) — the macro fires. Focus the send input and press F1 — nothing happens (browser default may bubble; that's fine, we just don't trigger the macro).

- [ ] **Step 4: Commit**

```bash
git add src/App.svelte
git commit -m "feat(ui): F1–F12 macro hotkeys when focus is outside form fields"
```

---

## Task 12: Per-tab filter fields + helper + tests

**Files:**
- Modify: `src/lib/stores/tabs.ts`
- Modify: `src/lib/stores/tabs.test.ts`
- Create: `src/lib/filter.ts`
- Test: `src/lib/filter.test.ts`

- [ ] **Step 1: Add filter fields to the Tab type**

Edit `src/lib/stores/tabs.ts`. In the `Tab` interface add:

```typescript
  filterEnabled: boolean;
  filterPattern: string;
```

In `defaultTab()` initialize them:

```typescript
    filterEnabled: false,
    filterPattern: "",
```

- [ ] **Step 2: Filter helper**

`src/lib/filter.ts`:

```typescript
export interface FilterResult {
  text: string;
  regex: RegExp | null;
  invalid: string | null;
}

export function applyFilter(text: string, pattern: string, enabled: boolean): FilterResult {
  if (!enabled || !pattern) return { text, regex: null, invalid: null };
  let re: RegExp;
  try {
    re = new RegExp(pattern);
  } catch (e) {
    return { text, regex: null, invalid: (e as Error).message };
  }
  const lines = text.split("\n");
  const kept: string[] = [];
  for (const line of lines) {
    if (re.test(line)) kept.push(line);
  }
  return { text: kept.join("\n"), regex: re, invalid: null };
}
```

- [ ] **Step 3: Tests for the helper**

`src/lib/filter.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { applyFilter } from "./filter";

describe("applyFilter", () => {
  it("passes through when disabled", () => {
    const r = applyFilter("a\nb\nc", "x", false);
    expect(r.text).toBe("a\nb\nc");
    expect(r.invalid).toBeNull();
  });

  it("passes through when pattern empty", () => {
    const r = applyFilter("a\nb\nc", "", true);
    expect(r.text).toBe("a\nb\nc");
  });

  it("keeps only matching lines", () => {
    const r = applyFilter("foo\nbar\nfoobar", "^foo", true);
    expect(r.text).toBe("foo\nfoobar");
  });

  it("flags invalid regex without throwing", () => {
    const r = applyFilter("hi", "(", true);
    expect(r.invalid).not.toBeNull();
    expect(r.text).toBe("hi");
  });
});
```

- [ ] **Step 4: Update tabs.test.ts fixture**

Edit `src/lib/stores/tabs.test.ts`. In `beforeEach`, add to the Tab object literal:

```typescript
      filterEnabled: false,
      filterPattern: "",
```

- [ ] **Step 5: Run tests + check + build**

```bash
npm run test
npm run check
npm run build
```

All three pass.

- [ ] **Step 6: Commit**

```bash
git add src/lib/stores/tabs.ts src/lib/stores/tabs.test.ts src/lib/filter.ts src/lib/filter.test.ts
git commit -m "feat(ui): per-tab filter fields and applyFilter helper"
```

---

## Task 13: `FilterPanel.svelte`

**Files:**
- Create: `src/lib/components/panels/FilterPanel.svelte`
- Modify: `src/App.svelte`
- Modify: `src/lib/components/ActivityBar.svelte`

- [ ] **Step 1: Write the panel**

`src/lib/components/panels/FilterPanel.svelte`:

```svelte
<script lang="ts">
  import { activeTab, patchActiveTab } from "$lib/stores/tabs";
  import { applyFilter } from "$lib/filter";

  let liveError = $derived(applyFilter("", $activeTab.filterPattern, $activeTab.filterEnabled).invalid);
</script>

<div class="panel">
  <h4>Filter</h4>

  <label class="toggle">
    <input
      type="checkbox"
      checked={$activeTab.filterEnabled}
      onchange={(e) => patchActiveTab({ filterEnabled: (e.currentTarget as HTMLInputElement).checked })}
    />
    Enable regex filter
  </label>

  <label class="lbl">Pattern</label>
  <input
    placeholder="e.g. ^READY|^ERROR"
    value={$activeTab.filterPattern}
    oninput={(e) => patchActiveTab({ filterPattern: (e.currentTarget as HTMLInputElement).value })}
    disabled={!$activeTab.filterEnabled}
  />

  {#if liveError}
    <p class="err">{liveError}</p>
  {/if}

  <p class="hint">Filter is applied at display time. Lines not matching are hidden; underlying buffer is preserved.</p>
</div>

<style>
  .panel { padding: 12px; height: 100%; box-sizing: border-box; display: flex; flex-direction: column; gap: 6px; }
  h4 { font-size: 10px; letter-spacing: 0.1em; text-transform: uppercase; color: var(--fg-subtle); margin: 0 0 6px; font-weight: 600; }
  .toggle { display: flex; align-items: center; gap: 6px; font-size: 11px; color: var(--fg-muted); }
  .lbl { font-size: 9px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--fg-subtle); margin-top: 6px; }
  input[type="text"], input:not([type]) { width: 100%; font-family: var(--font-mono); }
  .err { color: var(--err); font-size: 11px; margin: 4px 0 0; }
  .hint { color: var(--fg-subtle); font-size: 10px; margin-top: 8px; }
</style>
```

- [ ] **Step 2: Wire into App.svelte and ActivityBar**

Edit `src/App.svelte`. Add the import:

```typescript
import FilterPanel from "$lib/components/panels/FilterPanel.svelte";
```

Update the panel switch:

```svelte
{#if active === "connection"}
  <ConnectionPanel />
{:else if active === "macros"}
  <MacrosPanel />
{:else if active === "filter"}
  <FilterPanel />
{:else}
  <div class="muted-pad">Coming in Plan 3.</div>
{/if}
```

Edit `src/lib/components/ActivityBar.svelte`. Change the filter entry's title:

```typescript
{ key: "filter",     icon: "⌕", title: "Filter" },
```

- [ ] **Step 3: Type check + build**

```bash
npm run check
npm run build
```

Both pass.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/panels/FilterPanel.svelte src/lib/components/ActivityBar.svelte src/App.svelte
git commit -m "feat(ui): filter panel with per-tab regex pattern"
```

---

## Task 14: Apply filter in `Terminal.svelte`

**Files:**
- Modify: `src/lib/components/Terminal.svelte`

- [ ] **Step 1: Apply the filter to the rendered display string**

Edit `src/lib/components/Terminal.svelte`. Add the filter import next to the existing `bytesToText, formatHex` import:

```typescript
import { applyFilter } from "$lib/filter";
```

Replace the existing `display` derivation with one that applies the filter to the ASCII portion:

```typescript
let display = $derived.by(() => {
  const t = $activeTab;
  const filter = (text: string) => applyFilter(text, t.filterPattern, t.filterEnabled).text;
  if (t.viewMode === "ascii") return filter(bytesToText(t.buffer, t.stripAnsi));
  if (t.viewMode === "hex") return formatHex(t.buffer);
  const ascii = filter(bytesToText(t.buffer, t.stripAnsi));
  const hex = formatHex(t.buffer);
  return `=== HEX ===\n${hex}\n=== ASCII ===\n${ascii}`;
});
```

(HEX mode bypasses the filter — filtering hex dumps doesn't have an obvious meaning.)

- [ ] **Step 2: Type check + build**

```bash
npm run check
npm run build
```

Both pass.

- [ ] **Step 3: Manual smoke**

Connect a port that emits multi-line output (e.g. heartbeats). Open Filter panel, enable filter, enter a pattern that matches some lines. Confirm only matching lines render. Disable filter — full buffer returns. Switch to HEX view — filter is bypassed.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/Terminal.svelte
git commit -m "feat(ui): Terminal applies per-tab filter at display time"
```

---

## Task 15: Backend `commands/logs.rs` (save formatted log)

**Files:**
- Create: `src-tauri/src/commands/logs.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add `tauri-plugin-dialog`**

Edit `src-tauri/Cargo.toml`, add to `[dependencies]`:

```toml
tauri-plugin-dialog = "2"
```

- [ ] **Step 2: Add npm side**

Edit `package.json` `dependencies`:

```json
"@tauri-apps/plugin-dialog": "^2.0.0"
```

Then `npm install`.

- [ ] **Step 3: Write the logs command module**

`src-tauri/src/commands/logs.rs`:

```rust
use std::fs;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Direction { Rx, Tx }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub ts_ms: i64,
    pub direction: Direction,
    pub bytes: Vec<u8>,
}

#[tauri::command]
pub fn save_log(path: String, entries: Vec<LogEntry>) -> AppResult<String> {
    if path.trim().is_empty() {
        return Err(AppError::Invalid("path is empty".to_string()));
    }
    let mut body = String::new();
    for e in &entries {
        let dir = match e.direction {
            Direction::Rx => "RX",
            Direction::Tx => "TX",
        };
        let text = String::from_utf8_lossy(&e.bytes);
        body.push_str(&format!("[{}] {}: {}\n", e.ts_ms, dir, text));
    }
    fs::write(&path, body).map_err(|e| AppError::Io(e.to_string()))?;
    Ok(format!("wrote {} entries to {}", entries.len(), path))
}
```

- [ ] **Step 4: Register module + commands + dialog plugin**

Edit `src-tauri/src/commands/mod.rs`:

```rust
pub mod logs;
pub mod macros;
pub mod ports;
```

Edit `src-tauri/src/lib.rs`. Add the dialog plugin and the new command to the builder:

```rust
        tauri::Builder::default()
            .plugin(tauri_plugin_dialog::init())
            .manage(PortManager::new())
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
            ])
```

Edit `src-tauri/capabilities/default.json`. Add the dialog plugin's permissions:

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
    "dialog:default"
  ]
}
```

- [ ] **Step 5: Build**

```bash
cd src-tauri
cargo build
```

Expected: clean.

- [ ] **Step 6: Commit**

```bash
cd ..
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/commands src-tauri/src/lib.rs src-tauri/capabilities/default.json package.json package-lock.json
git commit -m "feat(backend): save_log command + tauri-plugin-dialog"
```

---

## Task 16: Per-tab log entries + helpers

**Files:**
- Modify: `src/lib/stores/tabs.ts`
- Modify: `src/lib/stores/tabs.test.ts`
- Modify: `src/lib/eventRouter.ts`
- Modify: `src/lib/ipc.ts`

- [ ] **Step 1: Extend the Tab type and add a logging toggle**

Edit `src/lib/stores/tabs.ts`. Add to imports if needed:

```typescript
import type { Direction } from "$lib/ipc";
```

Wait — `Direction` should be a frontend type matching the backend `Direction` enum. Add it to `ipc.ts` first (Step 2 below) before this import works. For now write the store as if the import exists; we'll add the type next.

In the `Tab` interface, add:

```typescript
  loggingEnabled: boolean;
  logEntries: LogEntry[];
```

Add the `LogEntry` type at the top of the file (above `Tab`):

```typescript
export interface LogEntry {
  ts_ms: number;
  direction: Direction;
  bytes: number[];
}
```

In `defaultTab()`:

```typescript
    loggingEnabled: false,
    logEntries: [],
```

Add helpers at the bottom of the file:

```typescript
export function pushLog(id: TabId, direction: Direction, bytes: Uint8Array, ts_ms: number) {
  tabs.update((list) =>
    list.map((t) => {
      if (t.id !== id || !t.loggingEnabled) return t;
      const entry: LogEntry = { ts_ms, direction, bytes: Array.from(bytes) };
      const next = [...t.logEntries, entry];
      // Cap log at 10_000 entries to prevent runaway memory.
      if (next.length > 10_000) next.splice(0, next.length - 10_000);
      return { ...t, logEntries: next };
    })
  );
}

export function clearLog(id: TabId) {
  tabs.update((list) => list.map((t) => (t.id === id ? { ...t, logEntries: [] } : t)));
}
```

- [ ] **Step 2: Add `Direction` type to ipc.ts**

Edit `src/lib/ipc.ts`. Add near the other type exports:

```typescript
export type Direction = "rx" | "tx";

export interface LogEntryDto {
  ts_ms: number;
  direction: Direction;
  bytes: number[];
}
```

Add a `saveLog` to the `ipc` const:

```typescript
  saveLog: (path: string, entries: LogEntryDto[]) =>
    invoke<string>("save_log", { path, entries }),
```

- [ ] **Step 3: Update event router to also push to logs**

Edit `src/lib/eventRouter.ts`. Update the `subscribe` function:

```typescript
import { tabs, appendBytes, pushLog, type Tab } from "$lib/stores/tabs";
```

In each handler:

```typescript
const unrx = await onRx(id, (event) => {
  const p = event.payload as RxPayload;
  const bytes = new Uint8Array(p.bytes);
  appendBytes(id, bytes, "rx");
  pushLog(id, "rx", bytes, p.ts);
});
const untx = await onTx(id, (event) => {
  const p = event.payload as TxPayload;
  const bytes = new Uint8Array(p.bytes);
  appendBytes(id, bytes, "tx");
  pushLog(id, "tx", bytes, p.ts);
});
```

- [ ] **Step 4: Update tabs.test.ts fixture**

Add to the `beforeEach` Tab literal:

```typescript
      loggingEnabled: false,
      logEntries: [],
```

Add a small test:

```typescript
it("pushLog only captures when logging is enabled", () => {
  // logging is off by default → entry is dropped
  pushLog(1, "rx", new Uint8Array([1, 2, 3]), 1000);
  expect(get(activeTab).logEntries.length).toBe(0);

  patchActiveTab({ loggingEnabled: true });
  pushLog(1, "rx", new Uint8Array([4, 5]), 2000);
  expect(get(activeTab).logEntries).toEqual([
    { ts_ms: 2000, direction: "rx", bytes: [4, 5] },
  ]);
});
```

Add `pushLog` to the imports of the test file.

- [ ] **Step 5: Run tests + check + build**

```bash
npm run test
npm run check
npm run build
```

All three pass.

- [ ] **Step 6: Commit**

```bash
git add src/lib/stores/tabs.ts src/lib/stores/tabs.test.ts src/lib/eventRouter.ts src/lib/ipc.ts
git commit -m "feat(ui): per-tab log capture wired through event router"
```

---

## Task 17: `LogsPanel.svelte` (toggle + native save dialog)

**Files:**
- Create: `src/lib/components/panels/LogsPanel.svelte`
- Modify: `src/App.svelte`
- Modify: `src/lib/components/ActivityBar.svelte`

- [ ] **Step 1: Write the panel**

`src/lib/components/panels/LogsPanel.svelte`:

```svelte
<script lang="ts">
  import { activeTab, activeTabId, patchActiveTab, clearLog } from "$lib/stores/tabs";
  import { ipc, type LogEntryDto } from "$lib/ipc";
  import { save } from "@tauri-apps/plugin-dialog";

  async function exportLog() {
    const path = await save({
      title: "Save log as",
      defaultPath: `rustcom_${new Date().toISOString().replace(/[:.]/g, "-")}.log`,
      filters: [{ name: "Log", extensions: ["log", "txt"] }],
    });
    if (!path) return;
    const entries: LogEntryDto[] = $activeTab.logEntries.map((e) => ({
      ts_ms: e.ts_ms,
      direction: e.direction,
      bytes: e.bytes,
    }));
    try {
      const msg = await ipc.saveLog(path, entries);
      patchActiveTab({ errorMessage: msg });
    } catch (e) {
      patchActiveTab({ errorMessage: String(e) });
    }
  }
</script>

<div class="panel">
  <h4>Logs</h4>

  <label class="toggle">
    <input
      type="checkbox"
      checked={$activeTab.loggingEnabled}
      onchange={(e) => patchActiveTab({ loggingEnabled: (e.currentTarget as HTMLInputElement).checked })}
    />
    Capture RX/TX to log
  </label>

  <p class="meta">{$activeTab.logEntries.length} entr{$activeTab.logEntries.length === 1 ? "y" : "ies"} buffered</p>

  <div class="row">
    <button class="primary" onclick={exportLog} disabled={$activeTab.logEntries.length === 0}>Save log…</button>
    <button onclick={() => clearLog($activeTabId)} disabled={$activeTab.logEntries.length === 0}>Clear</button>
  </div>

  <p class="hint">Capture starts from the moment you toggle this on. Entries are kept in memory only (not persisted across restarts).</p>
</div>

<style>
  .panel { padding: 12px; height: 100%; box-sizing: border-box; display: flex; flex-direction: column; gap: 8px; }
  h4 { font-size: 10px; letter-spacing: 0.1em; text-transform: uppercase; color: var(--fg-subtle); margin: 0 0 6px; font-weight: 600; }
  .toggle { display: flex; align-items: center; gap: 6px; font-size: 11px; color: var(--fg-muted); }
  .meta { color: var(--fg-subtle); font-size: 11px; margin: 4px 0; }
  .row { display: flex; gap: 6px; }
  .row button { padding: 6px 10px; font-size: 10px; }
  .hint { color: var(--fg-subtle); font-size: 10px; margin-top: 8px; }
</style>
```

- [ ] **Step 2: Wire into App.svelte and ActivityBar**

Edit `src/App.svelte`:

```typescript
import LogsPanel from "$lib/components/panels/LogsPanel.svelte";
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
{:else}
  <div class="muted-pad">Coming in Plan 3.</div>
{/if}
```

Edit `src/lib/components/ActivityBar.svelte`:

```typescript
{ key: "logs",       icon: "≡", title: "Logs" },
```

- [ ] **Step 3: Type check + build**

```bash
npm run check
npm run build
```

Both pass.

- [ ] **Step 4: Manual smoke**

Open Logs panel. Enable capture. Receive some bytes. Click Save log… — native dialog opens, pick a path, file is written with `[ts] RX: ...` / `[ts] TX: ...` lines. Disable capture, no new entries.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/panels/LogsPanel.svelte src/lib/components/ActivityBar.svelte src/App.svelte
git commit -m "feat(ui): logs panel with native save dialog"
```

---

## Task 18: Per-tab search overlay state + match finder + tests

**Files:**
- Modify: `src/lib/stores/tabs.ts`
- Modify: `src/lib/stores/tabs.test.ts`
- Create: `src/lib/search.ts`
- Test: `src/lib/search.test.ts`

- [ ] **Step 1: Add search fields to Tab**

Edit `src/lib/stores/tabs.ts`. In the `Tab` interface:

```typescript
  searchOpen: boolean;
  searchPattern: string;
  searchUseRegex: boolean;
  searchMatchIndex: number; // -1 when no matches
```

In `defaultTab()`:

```typescript
    searchOpen: false,
    searchPattern: "",
    searchUseRegex: false,
    searchMatchIndex: -1,
```

In the `beforeEach` test fixture, add the same four fields.

- [ ] **Step 2: Search helper**

`src/lib/search.ts`:

```typescript
export interface MatchRange {
  start: number;
  end: number; // exclusive
}

export interface SearchResult {
  matches: MatchRange[];
  invalid: string | null;
}

export function findMatches(text: string, pattern: string, useRegex: boolean): SearchResult {
  if (!pattern) return { matches: [], invalid: null };
  let re: RegExp;
  try {
    if (useRegex) {
      re = new RegExp(pattern, "g");
    } else {
      re = new RegExp(escapeRegex(pattern), "gi");
    }
  } catch (e) {
    return { matches: [], invalid: (e as Error).message };
  }
  const out: MatchRange[] = [];
  let m: RegExpExecArray | null;
  while ((m = re.exec(text)) !== null) {
    if (m.index === re.lastIndex) re.lastIndex += 1; // guard against zero-width loops
    out.push({ start: m.index, end: m.index + m[0].length });
  }
  return { matches: out, invalid: null };
}

function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
```

- [ ] **Step 3: Tests**

`src/lib/search.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { findMatches } from "./search";

describe("findMatches", () => {
  it("returns no matches for empty pattern", () => {
    expect(findMatches("hello", "", false).matches).toEqual([]);
  });

  it("plain substring is case-insensitive", () => {
    const r = findMatches("HelloHELLO", "hello", false);
    expect(r.matches).toEqual([
      { start: 0, end: 5 },
      { start: 5, end: 10 },
    ]);
  });

  it("regex mode honors flags and returns ranges", () => {
    const r = findMatches("foo123bar45", "\\d+", true);
    expect(r.matches.map((m) => [m.start, m.end])).toEqual([
      [3, 6],
      [9, 11],
    ]);
  });

  it("invalid regex flagged without throwing", () => {
    const r = findMatches("hi", "(", true);
    expect(r.invalid).not.toBeNull();
    expect(r.matches).toEqual([]);
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
git add src/lib/stores/tabs.ts src/lib/stores/tabs.test.ts src/lib/search.ts src/lib/search.test.ts
git commit -m "feat(ui): per-tab search state and findMatches helper"
```

---

## Task 19: `SearchOverlay.svelte` (Ctrl+F triggered)

**Files:**
- Create: `src/lib/components/SearchOverlay.svelte`
- Modify: `src/lib/components/Terminal.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Write the overlay**

`src/lib/components/SearchOverlay.svelte`:

```svelte
<script lang="ts">
  import { activeTab, patchActiveTab } from "$lib/stores/tabs";
  import { findMatches } from "$lib/search";

  let { displayText = "" }: { displayText?: string } = $props();

  let result = $derived(findMatches(displayText, $activeTab.searchPattern, $activeTab.searchUseRegex));
  let total = $derived(result.matches.length);
  let current = $derived(total === 0 ? -1 : Math.max(0, Math.min($activeTab.searchMatchIndex, total - 1)));

  function close() {
    patchActiveTab({ searchOpen: false, searchMatchIndex: -1 });
  }

  function next() {
    if (total === 0) return;
    const i = current === -1 ? 0 : (current + 1) % total;
    patchActiveTab({ searchMatchIndex: i });
  }

  function prev() {
    if (total === 0) return;
    const i = current === -1 ? total - 1 : (current - 1 + total) % total;
    patchActiveTab({ searchMatchIndex: i });
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") { e.preventDefault(); close(); }
    else if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); next(); }
    else if (e.key === "Enter" && e.shiftKey) { e.preventDefault(); prev(); }
  }
</script>

{#if $activeTab.searchOpen}
  <div class="overlay">
    <input
      autofocus
      placeholder="Find…"
      value={$activeTab.searchPattern}
      oninput={(e) => patchActiveTab({ searchPattern: (e.currentTarget as HTMLInputElement).value, searchMatchIndex: -1 })}
      onkeydown={onKey}
    />
    <label class="re">
      <input
        type="checkbox"
        checked={$activeTab.searchUseRegex}
        onchange={(e) => patchActiveTab({ searchUseRegex: (e.currentTarget as HTMLInputElement).checked, searchMatchIndex: -1 })}
      />
      .* regex
    </label>
    <span class="count">{total === 0 ? "0" : `${current + 1} / ${total}`}</span>
    <button onclick={prev} disabled={total === 0}>↑</button>
    <button onclick={next} disabled={total === 0}>↓</button>
    <button onclick={close}>×</button>
    {#if result.invalid}
      <span class="err">{result.invalid}</span>
    {/if}
  </div>
{/if}

<style>
  .overlay {
    position: absolute;
    top: 8px;
    right: 12px;
    background: var(--bg-titlebar);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    font-size: 11px;
    z-index: 10;
  }
  .overlay input { font-family: var(--font-mono); width: 200px; }
  .re { display: inline-flex; align-items: center; gap: 4px; color: var(--fg-muted); }
  .count { color: var(--fg-subtle); min-width: 50px; text-align: right; }
  .overlay button { padding: 2px 8px; font-size: 11px; }
  .err { color: var(--err); }
</style>
```

- [ ] **Step 2: Embed the overlay in Terminal**

Edit `src/lib/components/Terminal.svelte`. Import the overlay:

```typescript
import SearchOverlay from "$lib/components/SearchOverlay.svelte";
```

Wrap the `.term` div in a positioned container, and put the overlay inside it. Replace:

```svelte
<div class="term" bind:this={scrollEl}>
  <pre>{display}</pre>
</div>
```

with:

```svelte
<div class="term-wrap">
  <SearchOverlay displayText={display} />
  <div class="term" bind:this={scrollEl}>
    <pre>{display}</pre>
  </div>
</div>
```

Add the wrapper style:

```css
.term-wrap { position: relative; flex: 1; display: flex; flex-direction: column; min-height: 0; }
.term-wrap .term { flex: 1; }
```

- [ ] **Step 3: Add Ctrl+F shortcut in App.svelte**

Edit `src/App.svelte`. In `onWindowKey` (added in Task 11), add a Ctrl+F branch BEFORE the F-key check:

```typescript
function onWindowKey(e: KeyboardEvent) {
  // Ctrl+F (or Cmd+F on macOS) toggles the search overlay for the active tab.
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "f") {
    e.preventDefault();
    const open = !get(activeTab).searchOpen;
    patchActiveTab({ searchOpen: open, searchMatchIndex: -1 });
    return;
  }

  // F1–F12 trigger macros if assigned.
  if (!/^F([1-9]|1[0-2])$/.test(e.key)) return;
  // ... rest of existing handler
}
```

Add `patchActiveTab, activeTab` to the tabs imports if not already there.

- [ ] **Step 4: Type check + build**

```bash
npm run check
npm run build
```

Both pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/SearchOverlay.svelte src/lib/components/Terminal.svelte src/App.svelte
git commit -m "feat(ui): Ctrl+F search overlay with regex, prev/next, count"
```

---

## Task 20: Match highlighting in `Terminal.svelte`

**Files:**
- Modify: `src/lib/components/Terminal.svelte`

- [ ] **Step 1: Render highlighted spans for matches**

Edit `src/lib/components/Terminal.svelte`. Add the search import:

```typescript
import { findMatches, type MatchRange } from "$lib/search";
```

Add a derived value for match ranges and a helper to render them as HTML:

```typescript
let matches = $derived(
  $activeTab.searchOpen
    ? findMatches(display, $activeTab.searchPattern, $activeTab.searchUseRegex).matches
    : []
);

function renderHighlighted(text: string, ranges: MatchRange[], current: number): string {
  if (ranges.length === 0) return escapeHtml(text);
  let out = "";
  let pos = 0;
  ranges.forEach((r, i) => {
    if (r.start > pos) out += escapeHtml(text.slice(pos, r.start));
    const cls = i === current ? "match current" : "match";
    out += `<mark class="${cls}">${escapeHtml(text.slice(r.start, r.end))}</mark>`;
    pos = r.end;
  });
  if (pos < text.length) out += escapeHtml(text.slice(pos));
  return out;
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}
```

Replace the rendering of the `<pre>` block. Find:

```svelte
<pre>{display}</pre>
```

Replace with:

```svelte
<pre>{@html renderHighlighted(display, matches, $activeTab.searchMatchIndex)}</pre>
```

Add CSS for the marks (inside the existing `<style>` block):

```css
pre :global(mark.match) {
  background: var(--warn-bg);
  color: var(--warn);
  padding: 0 1px;
  border-radius: 2px;
}
pre :global(mark.match.current) {
  background: var(--accent);
  color: var(--accent-fg);
}
```

- [ ] **Step 2: Type check + build**

```bash
npm run check
npm run build
```

Both pass. Note: using `{@html ...}` with `escapeHtml` is safe — we never inject the user's regex, only the matched substrings, all escaped.

- [ ] **Step 3: Manual smoke**

Connect a port that emits some text. Press Ctrl+F. Type a substring — matches highlight in yellow, current match is in accent color. Up/down arrows in the overlay (or Shift+Enter / Enter) walk through matches. Esc closes.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/Terminal.svelte
git commit -m "feat(ui): highlight search matches inline in the terminal"
```

---

## Task 21: Final smoke test for Plan 2

- [ ] **Step 1: Clean release build**

```bash
cargo tauri build
```

Expected: `RustCOM_0.1.0_x64-setup.exe` and `.msi` are produced. (Version bump to 0.2.0 stays a Plan 3 task — the v0.1.0 binary is fine for the milestone.)

- [ ] **Step 2: Run the dev binary and walk this checklist**

```bash
cargo tauri dev
```

- [ ] Multi-tab: open 3 tabs, connect each to a different (or the same — for a smoke test) port, send/receive on each independently, switch between them, close them.
- [ ] Closing the last tab spawns a fresh blank one.
- [ ] Macros: create a macro with name + ASCII payload + line ending, save, verify it persists across an app restart.
- [ ] Macros: assign F2 to a macro, connect a port, focus the terminal area (not the send input), press F2 — macro fires, byte counter goes up.
- [ ] Filter: enable filter on a tab, enter `^DEBUG` (or any pattern), verify only matching lines render. Switch to HEX view — filter is bypassed.
- [ ] Filter: invalid regex shows the error inline.
- [ ] Logs: enable capture, receive some bytes, see counter rise. Click Save log… — native dialog opens, save to a file, file contains formatted entries.
- [ ] Search: press Ctrl+F, type a substring, see highlights. Up/down walks them, count updates. Esc closes.
- [ ] Search: regex toggle works (try `\d+` against a numeric line).

- [ ] **Step 3: Tag the milestone**

```bash
git tag plan-2-panels
```

- [ ] **Step 4: Fix anything smoke turned up**

If anything misbehaves, fix with focused commits before declaring Plan 2 done. Re-run the smoke checklist after each fix.

---

## Out of scope (planned for follow-up plans)

**Plan 3:**
- Lua scripting (mlua) — `script_run`, `script_stop`, `on_recv` Lua API.
- Monaco editor in the Scripts panel.
- Settings persistence (`settings.json` for last-used connection defaults, theme prefs).
- Auto-reconnect (per-tab toggle already in `PortConfig` but not wired in the worker).
- README + multi-platform packaging polish.
- Cleanup items from Plan 1 final review:
  - Rename `Cargo.toml` package from `app` → `rustcom`.
  - Wire or drop `tauri-plugin-log`.
  - Add minimal CSP to `tauri.conf.json`.
  - Fix a11y warnings on `ConnectionPanel.svelte`.
- Drop `legacy-egui/`.

**Future v2 (after Plan 3):**
- Plot/graph view of numeric RX.
- Multi-tab Lua scripting.
