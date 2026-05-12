# RustCOM — Plan 4: Plot view + multi-tab Lua + session restore + polish

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the v2 features the user explicitly called out at brainstorming time — a streaming numeric plot of RX data, a multi-tab Lua API so one script can drive multiple ports, and session restore — plus the small UX papercuts surfaced during Plan 3 use.

**Architecture:** Plot view is a new activity-bar panel rendering `uPlot` against a per-tab in-memory ring of `{ts, [series]}` extracted by user-defined regex named-capture groups. Multi-tab Lua adds a `tabs` Lua table that exposes `list/send/send_text/send_hex/on_recv` against any open `TabId`; the engine keeps a `HashMap<TabId, Function>` of per-tab `on_recv` callbacks alongside the v1 global. Session restore writes open-tab connection configs to `settings.json` on close and reads them on startup behind an opt-in toggle. Polish: inline log entry preview, aria-live on toasts, real-icon plot glyph (SVG), root `Cargo.lock` cleanup, "Reconnecting (n)" tab indicator.

**Tech Stack:** Tauri 2 + Svelte 5 + TypeScript + uPlot (~40 KB streaming time-series). No new Rust crates.

**Spec:** `docs/superpowers/specs/2026-05-04-tauri-redesign-design.md` (section 11 lists plot/multi-tab Lua/session restore as v2).
**Predecessor plans:** Plan 1 / 2 / 3 in the same `plans/` directory.

**Working branch:** continue on `tauri-redesign` if the v0.2.0 merge to `main` hasn't happened yet, otherwise branch `plan-4-plots` from `main`. The first task verifies and switches if needed.

**Working directory:** `C:\Users\WilliamHerr\Desktop\Code\RustCOM`.

**Testing philosophy:** Same as Plans 1–3. Pure logic (regex extraction, ring buffer, multi-tab Lua dispatch table) gets unit tests. The plot rendering itself is verified by running.

---

## Plan 4 task overview

| # | Task | Verification |
|---|---|---|
| 1 | Branch / starting state confirmation | `git status` clean, branch correct |
| 2 | Polish: aria-live on toasts | Manual: screen-reader announces |
| 3 | Polish: untrack root Cargo.lock leftover | `git ls-files Cargo.lock` returns nothing |
| 4 | Polish: "Reconnecting (n)" indicator on tab strip | Manual: yank USB with auto-reconnect on |
| 5 | Polish: inline log preview in Logs panel | Manual: log() shows in panel as it happens |
| 6 | ActivityBar: support SVG icons (not just text glyphs) | Existing 5 entries unchanged |
| 7 | Plot regex extractor + ring buffer + tests | `vitest run` |
| 8 | uPlot package + `lib/plot.ts` lazy loader | `npm run build` clean, separate chunk |
| 9 | Per-tab plot fields + helpers + tests | `vitest run` |
| 10 | `PlotPanel.svelte` + new "Plot" activity-bar entry | Manual: regex matches → lines render |
| 11 | Plot CSV export via native save dialog | Manual: writes `.csv` with header + rows |
| 12 | Backend Lua `tabs` API + tests | `cargo test --lib script::tabs_api` |
| 13 | ScriptEngine multi-tab `on_recv` dispatch + tests | `cargo test --lib script::engine` |
| 14 | Backend session restore in settings.rs | `cargo test --lib settings` |
| 15 | Frontend session restore on app start | Manual: open tabs, close, reopen |
| 16 | Bump version to 0.3.0 + final smoke + tag `v0.3.0` | `cargo tauri build`, smoke checklist |

---

## Task 1: Branch / starting state confirmation

**Files:** none (verification + possible branch change).

- [ ] **Step 1: Check current branch and uncommitted state**

```bash
git -C "C:/Users/WilliamHerr/Desktop/Code/RustCOM" status
git -C "C:/Users/WilliamHerr/Desktop/Code/RustCOM" branch --show-current
```

If on `tauri-redesign` and clean, proceed on it.

If on `main` (because v0.2.0 was merged), create + switch to a fresh branch:

```bash
git -C "C:/Users/WilliamHerr/Desktop/Code/RustCOM" checkout -b plan-4-plots
```

If the working tree has uncommitted changes, stop and report BLOCKED — the controller will deal with it.

- [ ] **Step 2: Confirm v0.2.0 tag is reachable**

```bash
git -C "C:/Users/WilliamHerr/Desktop/Code/RustCOM" log --oneline -1 v0.2.0
```

Should print the commit `4ff008f chore: bump version to 0.2.0` (or similar SHA — the message is what matters).

- [ ] **Step 3: No commit (verification only).**

Report current branch + HEAD SHA back to the controller.

---

## Task 2: aria-live on toasts

**Files:**
- Modify: `src/lib/components/ToastStack.svelte`

- [ ] **Step 1: Add ARIA attributes**

Edit `src/lib/components/ToastStack.svelte`. Replace the outer `<div class="stack">` opening tag with:

```svelte
<div class="stack" role="region" aria-live="polite" aria-atomic="false" aria-label="Notifications">
```

Each toast `<button>` gets `aria-label` from the message:

```svelte
<button type="button" class="toast {t.level}" onclick={() => dismissToast(t.id)} aria-label={`Dismiss notification: ${t.msg}`}>
  <span>{t.msg}</span>
</button>
```

- [ ] **Step 2: Type check + build**

```bash
npm run check
npm run build
```

Both pass with 0 warnings.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ToastStack.svelte
git commit -m "chore(a11y): aria-live region + per-toast aria-labels"
```

---

## Task 3: Untrack root Cargo.lock leftover

**Files:**
- Possible change: `.gitignore` (verify only)

The `.gitignore` line `/Cargo.lock` correctly excludes a root-level lockfile (since the actual crate is in `src-tauri/`, the root `Cargo.lock` would only exist as legacy from before the Tauri scaffold).

- [ ] **Step 1: Confirm there is no tracked `Cargo.lock` at the repo root**

```bash
git -C "C:/Users/WilliamHerr/Desktop/Code/RustCOM" ls-files Cargo.lock
```

Expected: empty output. If anything prints, run `git rm Cargo.lock` and commit.

- [ ] **Step 2: Confirm `src-tauri/Cargo.lock` IS tracked**

```bash
git -C "C:/Users/WilliamHerr/Desktop/Code/RustCOM" ls-files src-tauri/Cargo.lock
```

Expected: prints `src-tauri/Cargo.lock`.

- [ ] **Step 3: No commit if both checks above passed.**

If something needed fixing in step 1, commit:

```bash
git commit -m "chore: drop stale root Cargo.lock"
```

Report DONE either way.

---

## Task 4: "Reconnecting (n)" indicator on tab strip

**Files:**
- Modify: `src/lib/stores/tabs.ts` (add `reconnectAttempts` field on Tab)
- Modify: `src/lib/eventRouter.ts` (bump on disconnect-while-auto-reconnecting; reset on reconnect)
- Modify: `src/lib/components/TabStrip.svelte` (show count when state === "reconnecting")

- [ ] **Step 1: Extend Tab type**

Edit `src/lib/stores/tabs.ts`. In the `Tab` interface add:

```typescript
  reconnectAttempts: number;
```

In `defaultTab()`:

```typescript
    reconnectAttempts: 0,
```

In the `beforeEach` test fixture in `src/lib/stores/tabs.test.ts`, add the same field:

```typescript
      reconnectAttempts: 0,
```

- [ ] **Step 2: Update event router**

Edit `src/lib/eventRouter.ts`. In `subscribe`, modify the disconnect handler block (the one that currently sets `state: "reconnecting"`):

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
        reconnectAttempts: willReconnect ? t.reconnectAttempts + 1 : t.reconnectAttempts,
      };
      return next;
    })
  );
  if (!willReconnect) {
    ipc.closePort(id).catch(() => undefined);
  }
});
```

Modify the reconnect handler to reset the counter:

```typescript
const unre = await onReconnect(id, (_event) => {
  tabs.update((list) =>
    list.map((t) =>
      t.id === id ? { ...t, state: "connected", errorMessage: null, reconnectAttempts: 0 } : t
    )
  );
});
```

- [ ] **Step 3: Update TabStrip rendering**

Edit `src/lib/components/TabStrip.svelte`. Inside the `{#each $tabs as t (t.id)}` loop, find the existing `<span class="label">{labelFor(t.id)}</span>` and replace with:

```svelte
<span class="label">
  {labelFor(t.id)}
  {#if t.state === "reconnecting" && t.reconnectAttempts > 1}
    <span class="attempt"> · retry {t.reconnectAttempts}</span>
  {/if}
</span>
```

Add to the `<style>` block:

```css
.attempt { color: var(--warn); font-size: 9px; }
```

- [ ] **Step 4: Type check + build**

```bash
npm run check
npm run build
```

Both pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib/stores/tabs.ts src/lib/stores/tabs.test.ts src/lib/eventRouter.ts src/lib/components/TabStrip.svelte
git commit -m "feat(ui): show reconnect attempt count on the tab strip"
```

---

## Task 5: Inline log preview in Logs panel

**Files:**
- Modify: `src/lib/components/panels/LogsPanel.svelte`

Add a scrollable monospace list of the last ~50 entries above the action row.

- [ ] **Step 1: Render entries inline**

Edit `src/lib/components/panels/LogsPanel.svelte`. Replace the existing `<p class="meta">{$activeTab.logEntries.length} entr...` line and the `<div class="row">` button block with:

```svelte
<p class="meta">{$activeTab.logEntries.length} entr{$activeTab.logEntries.length === 1 ? "y" : "ies"} buffered</p>

<div class="preview">
  {#if $activeTab.logEntries.length === 0}
    <p class="hint">No entries yet. Capture is {$activeTab.loggingEnabled ? "on" : "off"}.</p>
  {:else}
    {#each $activeTab.logEntries.slice(-50) as e, i (i)}
      <div class="line">
        <span class="ts">{new Date(e.ts_ms).toLocaleTimeString()}</span>
        <span class="dir {e.direction}">{e.direction.toUpperCase()}</span>
        <span class="text">{decodeForDisplay(e.bytes)}</span>
      </div>
    {/each}
  {/if}
</div>

<div class="row">
  <button class="primary" onclick={exportLog} disabled={$activeTab.logEntries.length === 0}>Save log…</button>
  <button onclick={() => clearLog($activeTabId)} disabled={$activeTab.logEntries.length === 0}>Clear</button>
</div>
```

Add a small helper above the markup (in the script block):

```typescript
const decoder = new TextDecoder("utf-8", { fatal: false });
function decodeForDisplay(bytes: number[]): string {
  return decoder.decode(new Uint8Array(bytes)).replace(/\r?\n$/, "");
}
```

Add to the `<style>` block:

```css
.preview {
  flex: 1;
  min-height: 80px;
  max-height: 320px;
  overflow-y: auto;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 6px 8px;
  font-family: var(--font-mono);
  font-size: 10px;
  display: flex;
  flex-direction: column;
  gap: 1px;
}
.line { display: grid; grid-template-columns: auto auto 1fr; gap: 6px; align-items: baseline; }
.ts { color: var(--fg-subtle); font-size: 9px; }
.dir.rx { color: var(--rx); font-weight: 600; }
.dir.tx { color: var(--tx); font-weight: 600; }
.text { color: var(--fg); white-space: pre-wrap; word-break: break-word; }
```

Update `.panel`'s rule so the preview can flex:

```css
.panel { padding: 12px; height: 100%; box-sizing: border-box; display: flex; flex-direction: column; gap: 8px; min-height: 0; }
```

(Add `min-height: 0` to allow the preview to shrink/scroll inside the flex parent.)

- [ ] **Step 2: Type check + build**

```bash
npm run check
npm run build
```

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/panels/LogsPanel.svelte
git commit -m "feat(ui): inline log entry preview in Logs panel"
```

---

## Task 6: ActivityBar — support SVG icons

**Files:**
- Modify: `src/lib/components/ActivityBar.svelte`

The icon field is currently a single character. To embed an SVG for the new Plot entry, switch the rendering to `{@html item.icon}` and let entries provide either plain text or SVG markup. (Existing entries keep their text; only the new one in Task 10 uses SVG.)

- [ ] **Step 1: Switch the rendering**

Edit `src/lib/components/ActivityBar.svelte`. Find the rendering of each item:

```svelte
<button
  class="ico"
  class:active={active === item.key}
  title={item.title}
  onclick={() => (active = item.key)}
>
  {item.icon}
</button>
```

Replace `{item.icon}` with `{@html item.icon}`:

```svelte
<button
  class="ico"
  class:active={active === item.key}
  title={item.title}
  onclick={() => (active = item.key)}
>
  {@html item.icon}
</button>
```

This is safe because `item.icon` values are constants defined in the same component file — never user input.

The existing 5 items continue to work because their `"⚙"`, `"▶"`, `"{}"`, `"⌕"`, `"≡"` strings are valid HTML.

- [ ] **Step 2: Adjust icon CSS so SVG sizes correctly**

Add to the existing `.ico` rule (in the `<style>` block):

```css
.ico :global(svg) { width: 18px; height: 18px; display: block; }
```

- [ ] **Step 3: Type check + build**

```bash
npm run check
npm run build
```

Both pass with 0 warnings.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/ActivityBar.svelte
git commit -m "refactor(ui): ActivityBar icon supports SVG markup"
```

---

## Task 7: Plot regex extractor + ring buffer + tests

**Files:**
- Create: `src/lib/plotExtractor.ts`
- Test: `src/lib/plotExtractor.test.ts`

The extractor consumes appended RX text and produces `{ts, series}` samples whenever the user-defined regex matches a line. A ring buffer keeps the last N samples for plotting.

- [ ] **Step 1: Write the extractor**

`src/lib/plotExtractor.ts`:

```typescript
export interface Sample {
  ts: number;             // ms since plot reset (relative)
  values: Record<string, number>;
}

export interface ExtractResult {
  samples: Sample[];
  invalid: string | null;
  unmatchedLines: number;
}

/**
 * Extract numeric samples from text using a regex with named capture groups.
 *
 * Each match produces one Sample. Each named group whose captured string parses
 * as a finite number becomes a series entry. Groups that don't parse are
 * silently skipped (so partial matches don't blow up the whole sample).
 *
 * `startTs` is the wall-clock ms at plot start; sample `ts` is offset from it.
 * `nowMs` is the wall-clock ms when the text arrived.
 */
export function extract(
  text: string,
  pattern: string,
  startTs: number,
  nowMs: number,
): ExtractResult {
  if (!pattern) return { samples: [], invalid: null, unmatchedLines: 0 };
  let re: RegExp;
  try {
    re = new RegExp(pattern, "g");
  } catch (e) {
    return { samples: [], invalid: (e as Error).message, unmatchedLines: 0 };
  }
  const samples: Sample[] = [];
  let unmatchedLines = 0;
  for (const line of text.split("\n")) {
    re.lastIndex = 0;
    const m = re.exec(line);
    if (!m || !m.groups) {
      if (line.length > 0) unmatchedLines += 1;
      continue;
    }
    const values: Record<string, number> = {};
    for (const [name, raw] of Object.entries(m.groups)) {
      if (raw === undefined) continue;
      const n = Number(raw);
      if (Number.isFinite(n)) values[name] = n;
    }
    if (Object.keys(values).length > 0) {
      samples.push({ ts: nowMs - startTs, values });
    }
  }
  return { samples, invalid: null, unmatchedLines };
}

/** Fixed-size ring buffer for samples. Drops oldest when full. */
export class SampleRing {
  private buf: Sample[] = [];
  constructor(public readonly capacity: number) {}

  push(s: Sample) {
    this.buf.push(s);
    if (this.buf.length > this.capacity) {
      this.buf.splice(0, this.buf.length - this.capacity);
    }
  }

  pushMany(ss: Sample[]) {
    for (const s of ss) this.push(s);
  }

  toArray(): Sample[] {
    return this.buf.slice();
  }

  clear() {
    this.buf = [];
  }

  get size(): number {
    return this.buf.length;
  }

  /** Return the union of all series names that have ever appeared. */
  seriesNames(): string[] {
    const set = new Set<string>();
    for (const s of this.buf) for (const k of Object.keys(s.values)) set.add(k);
    return [...set].sort();
  }

  /** Convert to columnar arrays suitable for uPlot: [xs, ys1, ys2, ...] */
  toUPlotData(names: string[]): number[][] {
    const xs: number[] = [];
    const ys: number[][] = names.map(() => []);
    for (const s of this.buf) {
      xs.push(s.ts / 1000); // seconds
      names.forEach((n, i) => {
        const v = s.values[n];
        ys[i].push(Number.isFinite(v) ? v : Number.NaN);
      });
    }
    return [xs, ...ys];
  }

  /** CSV with `ts_ms` plus one column per series name. */
  toCsv(names: string[]): string {
    const header = ["ts_ms", ...names].join(",");
    const rows = this.buf.map((s) => {
      const cols = [String(s.ts), ...names.map((n) => {
        const v = s.values[n];
        return Number.isFinite(v) ? String(v) : "";
      })];
      return cols.join(",");
    });
    return [header, ...rows].join("\n") + "\n";
  }
}
```

- [ ] **Step 2: Tests**

`src/lib/plotExtractor.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { extract, SampleRing } from "./plotExtractor";

describe("extract", () => {
  it("returns no samples for empty pattern", () => {
    const r = extract("temp:25.0\n", "", 0, 1000);
    expect(r.samples).toEqual([]);
    expect(r.invalid).toBeNull();
  });

  it("extracts one named group", () => {
    const r = extract("temp:25.5\n", "temp:(?<temp>\\d+\\.\\d+)", 0, 2000);
    expect(r.samples.length).toBe(1);
    expect(r.samples[0].ts).toBe(2000);
    expect(r.samples[0].values).toEqual({ temp: 25.5 });
  });

  it("extracts multiple named groups", () => {
    const r = extract(
      "temp:24.7 hum:55.0\n",
      "temp:(?<temp>\\d+\\.\\d+) hum:(?<hum>\\d+\\.\\d+)",
      0,
      1000,
    );
    expect(r.samples[0].values).toEqual({ temp: 24.7, hum: 55.0 });
  });

  it("counts unmatched non-empty lines", () => {
    const r = extract("nope\nyep:42\n", "yep:(?<v>\\d+)", 0, 0);
    expect(r.unmatchedLines).toBe(1);
    expect(r.samples.length).toBe(1);
  });

  it("flags invalid regex without throwing", () => {
    const r = extract("hi", "(", 0, 0);
    expect(r.invalid).not.toBeNull();
    expect(r.samples).toEqual([]);
  });

  it("skips groups that don't parse as numbers", () => {
    const r = extract("a:abc b:5", "a:(?<a>\\w+) b:(?<b>\\d+)", 0, 0);
    expect(r.samples[0].values).toEqual({ b: 5 });
  });
});

describe("SampleRing", () => {
  it("pushes up to capacity then drops oldest", () => {
    const ring = new SampleRing(3);
    ring.push({ ts: 1, values: { a: 1 } });
    ring.push({ ts: 2, values: { a: 2 } });
    ring.push({ ts: 3, values: { a: 3 } });
    ring.push({ ts: 4, values: { a: 4 } });
    expect(ring.size).toBe(3);
    expect(ring.toArray().map((s) => s.ts)).toEqual([2, 3, 4]);
  });

  it("seriesNames returns union sorted", () => {
    const ring = new SampleRing(10);
    ring.push({ ts: 1, values: { temp: 1 } });
    ring.push({ ts: 2, values: { hum: 2 } });
    ring.push({ ts: 3, values: { temp: 3, hum: 4 } });
    expect(ring.seriesNames()).toEqual(["hum", "temp"]);
  });

  it("toUPlotData emits columnar arrays in seconds", () => {
    const ring = new SampleRing(10);
    ring.push({ ts: 1000, values: { a: 1 } });
    ring.push({ ts: 2000, values: { a: 2, b: 5 } });
    const [xs, ays, bys] = ring.toUPlotData(["a", "b"]);
    expect(xs).toEqual([1, 2]);
    expect(ays).toEqual([1, 2]);
    expect(bys[0]).toBeNaN();
    expect(bys[1]).toBe(5);
  });

  it("toCsv emits header + rows", () => {
    const ring = new SampleRing(10);
    ring.push({ ts: 0, values: { a: 1, b: 2 } });
    ring.push({ ts: 100, values: { a: 3 } });
    const csv = ring.toCsv(["a", "b"]);
    expect(csv).toBe("ts_ms,a,b\n0,1,2\n100,3,\n");
  });
});
```

- [ ] **Step 3: Run tests + check + build**

```bash
npm run test
npm run check
npm run build
```

All three pass. Total test count rises by 10.

- [ ] **Step 4: Commit**

```bash
git add src/lib/plotExtractor.ts src/lib/plotExtractor.test.ts
git commit -m "feat(ui): plot regex extractor + sample ring buffer"
```

---

## Task 8: uPlot package + lazy loader

**Files:**
- Modify: `package.json`
- Create: `src/lib/plot.ts`

uPlot is small, but we still load it on-demand (only when the Plot panel mounts) so the boot bundle stays light.

- [ ] **Step 1: Install uPlot**

Edit `package.json` `dependencies`:

```json
"uplot": "^1.6.31"
```

Run:

```bash
npm install
```

- [ ] **Step 2: Write the loader**

`src/lib/plot.ts`:

```typescript
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";

export type UPlotInstance = uPlot;
export type UPlotOptions = uPlot.Options;
export type UPlotData = uPlot.AlignedData;

/**
 * Build uPlot options matching our Zed Cool theme. `seriesNames` is the list
 * of series in order; their colors come from `palette` cyclically.
 */
const palette = ["#7a86c8", "#8fc4a8", "#c4a78c", "#d4b483", "#c87a86", "#b8c4d8"];

export function buildOptions(
  seriesNames: string[],
  width: number,
  height: number,
): UPlotOptions {
  return {
    width,
    height,
    pxAlign: false,
    cursor: { drag: { x: true, y: false }, points: { size: 6 } },
    legend: { show: true, live: true },
    scales: {
      x: { time: false },
      y: { auto: true },
    },
    axes: [
      { stroke: "#8a8d92", grid: { stroke: "#1f2124" }, ticks: { stroke: "#1f2124" }, label: "seconds", labelSize: 18 },
      { stroke: "#8a8d92", grid: { stroke: "#1f2124" }, ticks: { stroke: "#1f2124" } },
    ],
    series: [
      { label: "t (s)" },
      ...seriesNames.map((name, i) => ({
        label: name,
        stroke: palette[i % palette.length],
        width: 1.5,
      })),
    ],
  };
}

export { uPlot };
```

- [ ] **Step 3: Type check + build**

```bash
npm run check
npm run build
```

Both pass. Build summary will show a uPlot chunk only after Task 10 imports `$lib/plot` dynamically.

- [ ] **Step 4: Commit**

```bash
git add package.json package-lock.json src/lib/plot.ts
git commit -m "feat(ui): uPlot loader with Zed Cool theme defaults"
```

---

## Task 9: Per-tab plot fields + helpers + tests

**Files:**
- Modify: `src/lib/stores/tabs.ts`
- Modify: `src/lib/stores/tabs.test.ts`

- [ ] **Step 1: Add plot fields to Tab**

Edit `src/lib/stores/tabs.ts`. Add to `Tab`:

```typescript
  plotPattern: string;
  plotPaused: boolean;
  plotStartedAt: number; // ms wall clock; 0 means not started
  plotCapacity: number;
```

In `defaultTab()`:

```typescript
    plotPattern: "",
    plotPaused: false,
    plotStartedAt: 0,
    plotCapacity: 1000,
```

In the `beforeEach` test fixture, add the same four fields.

Add helpers at the bottom of the file:

```typescript
export function setPlotPattern(id: TabId, pattern: string) {
  tabs.update((list) =>
    list.map((t) => (t.id === id ? { ...t, plotPattern: pattern } : t))
  );
}

export function startPlot(id: TabId) {
  tabs.update((list) =>
    list.map((t) =>
      t.id === id ? { ...t, plotStartedAt: Date.now(), plotPaused: false } : t
    )
  );
}

export function pausePlot(id: TabId, paused: boolean) {
  tabs.update((list) =>
    list.map((t) => (t.id === id ? { ...t, plotPaused: paused } : t))
  );
}

export function setPlotCapacity(id: TabId, capacity: number) {
  const cap = Math.max(50, Math.floor(capacity));
  tabs.update((list) =>
    list.map((t) => (t.id === id ? { ...t, plotCapacity: cap } : t))
  );
}
```

- [ ] **Step 2: Add tests**

In `src/lib/stores/tabs.test.ts`, add to the imports:

```typescript
import { setPlotPattern, startPlot, pausePlot, setPlotCapacity } from "./tabs";
```

Add tests at the end of the existing `describe` block:

```typescript
it("setPlotPattern stores the pattern", () => {
  setPlotPattern(1, "v=(?<v>\\d+)");
  expect(get(activeTab).plotPattern).toBe("v=(?<v>\\d+)");
});

it("startPlot stamps wall-clock and unpauses", () => {
  pausePlot(1, true);
  startPlot(1);
  const t = get(activeTab);
  expect(t.plotPaused).toBe(false);
  expect(t.plotStartedAt).toBeGreaterThan(0);
});

it("pausePlot toggles", () => {
  pausePlot(1, true);
  expect(get(activeTab).plotPaused).toBe(true);
  pausePlot(1, false);
  expect(get(activeTab).plotPaused).toBe(false);
});

it("setPlotCapacity clamps to a minimum", () => {
  setPlotCapacity(1, 10);
  expect(get(activeTab).plotCapacity).toBe(50);
  setPlotCapacity(1, 5000);
  expect(get(activeTab).plotCapacity).toBe(5000);
});
```

- [ ] **Step 3: Run tests + check**

```bash
npm run test
npm run check
```

Both pass.

- [ ] **Step 4: Commit**

```bash
git add src/lib/stores/tabs.ts src/lib/stores/tabs.test.ts
git commit -m "feat(ui): per-tab plot state fields + helpers"
```

---

## Task 10: PlotPanel.svelte + new "Plot" activity-bar entry

**Files:**
- Create: `src/lib/components/panels/PlotPanel.svelte`
- Modify: `src/lib/components/ActivityBar.svelte`
- Modify: `src/App.svelte`
- Modify: `src/lib/eventRouter.ts` (push extracted samples through the same path that Logs uses)

- [ ] **Step 1: Add a per-tab sample ring outside the store**

Plot data updates fast (every RX event) and a Svelte writable would re-render every subscriber. Keep the SampleRing in a module-scoped Map keyed by TabId, exposed via a notification store that just holds a counter the panel watches.

Create `src/lib/plotState.ts`:

```typescript
import { writable } from "svelte/store";
import { SampleRing, type Sample } from "$lib/plotExtractor";
import type { TabId } from "$lib/ipc";

const rings = new Map<TabId, SampleRing>();

/** Bumped on every push so the panel can re-render. */
export const plotTick = writable(0);

export function ringFor(id: TabId, capacity: number): SampleRing {
  let ring = rings.get(id);
  if (!ring || ring.capacity !== capacity) {
    ring = new SampleRing(capacity);
    rings.set(id, ring);
  }
  return ring;
}

export function pushSamples(id: TabId, samples: Sample[]) {
  const ring = rings.get(id);
  if (!ring || samples.length === 0) return;
  ring.pushMany(samples);
  plotTick.update((n) => n + 1);
}

export function clearRing(id: TabId) {
  rings.get(id)?.clear();
  plotTick.update((n) => n + 1);
}

export function dropRing(id: TabId) {
  rings.delete(id);
}
```

- [ ] **Step 2: Hook RX into the extractor in eventRouter**

Edit `src/lib/eventRouter.ts`. Add imports:

```typescript
import { extract } from "$lib/plotExtractor";
import { pushSamples, dropRing } from "$lib/plotState";
```

Inside `subscribe`, modify the `onRx` handler to also feed the plot:

```typescript
const unrx = await onRx(id, (event) => {
  const p = event.payload as RxPayload;
  const bytes = new Uint8Array(p.bytes);
  appendBytes(id, bytes, "rx");
  pushLog(id, "rx", bytes, p.ts);

  const tab = getTab(id);
  if (tab && tab.plotPattern && !tab.plotPaused && tab.plotStartedAt > 0) {
    const text = new TextDecoder("utf-8", { fatal: false }).decode(bytes);
    const r = extract(text, tab.plotPattern, tab.plotStartedAt, p.ts);
    if (r.samples.length > 0) pushSamples(id, r.samples);
  }
});
```

In `unsubscribe`, also drop the ring (called when a tab closes):

```typescript
function unsubscribe(id: TabId) {
  const list = subs.get(id);
  if (!list) return;
  list.forEach((u) => u());
  subs.delete(id);
  dropRing(id);
}
```

- [ ] **Step 3: Write the panel**

`src/lib/components/panels/PlotPanel.svelte`:

```svelte
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { activeTab, activeTabId, setPlotPattern, startPlot, pausePlot, setPlotCapacity } from "$lib/stores/tabs";
  import { plotTick, ringFor, clearRing } from "$lib/plotState";
  import { extract } from "$lib/plotExtractor";

  type UPlotInstance = import("uplot").default;
  type UPlotOptions = import("uplot").Options;

  let chartEl: HTMLDivElement | undefined = $state();
  let chart: UPlotInstance | undefined;
  let panelError = $state<string | null>(null);
  let resizeObserver: ResizeObserver | undefined;
  let lastWidth = 0;
  let lastHeight = 0;

  onMount(async () => {
    if (!chartEl) return;
    const { uPlot, buildOptions } = await import("$lib/plot");
    const rect = chartEl.getBoundingClientRect();
    lastWidth = Math.max(200, rect.width);
    lastHeight = Math.max(160, rect.height);
    chart = new uPlot(buildOptions([], lastWidth, lastHeight) as UPlotOptions, [[], []], chartEl);
    resizeObserver = new ResizeObserver(() => {
      if (!chartEl || !chart) return;
      const r = chartEl.getBoundingClientRect();
      const w = Math.max(200, r.width);
      const h = Math.max(160, r.height);
      if (w !== lastWidth || h !== lastHeight) {
        lastWidth = w;
        lastHeight = h;
        chart.setSize({ width: w, height: h });
      }
    });
    resizeObserver.observe(chartEl);
    redraw();
  });

  onDestroy(() => {
    chart?.destroy();
    resizeObserver?.disconnect();
  });

  // Validate regex live so the user gets feedback as they type.
  let validation = $derived(extract("", $activeTab.plotPattern, 0, 0));

  $effect(() => {
    void $plotTick;
    void $activeTab.plotPattern;
    void $activeTab.plotCapacity;
    redraw();
  });

  async function redraw() {
    if (!chart) return;
    const ring = ringFor($activeTabId, $activeTab.plotCapacity);
    const names = ring.seriesNames();
    const data = ring.toUPlotData(names);

    // If the series set changed (e.g. user added a new named group), rebuild.
    const series = chart.series.slice(1).map((s) => s.label);
    const same = series.length === names.length && series.every((n, i) => n === names[i]);
    if (!same) {
      const { buildOptions } = await import("$lib/plot");
      chart.destroy();
      const { uPlot } = await import("$lib/plot");
      chart = new uPlot(
        buildOptions(names, lastWidth, lastHeight) as UPlotOptions,
        data as never,
        chartEl as HTMLDivElement,
      );
      return;
    }

    chart.setData(data as never);
  }

  function start() {
    panelError = validation.invalid;
    if (panelError) return;
    if (!$activeTab.plotPattern) {
      panelError = "Set a regex pattern with named groups first.";
      return;
    }
    startPlot($activeTabId);
    clearRing($activeTabId);
  }

  function clearAll() {
    clearRing($activeTabId);
  }
</script>

<div class="panel">
  <h4>Plot</h4>

  <label class="lbl">
    Pattern
    <input
      placeholder="e.g. temp:(?<temp>\d+\.\d+) hum:(?<hum>\d+\.\d+)"
      value={$activeTab.plotPattern}
      oninput={(e) => setPlotPattern($activeTabId, (e.currentTarget as HTMLInputElement).value)}
    />
  </label>

  <label class="lbl">
    Window
    <input
      type="number"
      min="50"
      step="50"
      value={$activeTab.plotCapacity}
      onchange={(e) => setPlotCapacity($activeTabId, Number((e.currentTarget as HTMLInputElement).value))}
    />
  </label>

  {#if validation.invalid}
    <p class="err">{validation.invalid}</p>
  {/if}
  {#if panelError && !validation.invalid}
    <p class="err">{panelError}</p>
  {/if}

  <div class="actions">
    {#if $activeTab.plotStartedAt === 0}
      <button class="primary" onclick={start}>Start</button>
    {:else if $activeTab.plotPaused}
      <button class="primary" onclick={() => pausePlot($activeTabId, false)}>Resume</button>
    {:else}
      <button onclick={() => pausePlot($activeTabId, true)}>Pause</button>
    {/if}
    <button onclick={clearAll}>Clear</button>
  </div>

  <div class="chart" bind:this={chartEl}></div>
</div>

<style>
  .panel { padding: 12px; height: 100%; box-sizing: border-box; display: flex; flex-direction: column; gap: 8px; min-height: 0; }
  h4 { font-size: 10px; letter-spacing: 0.1em; text-transform: uppercase; color: var(--fg-subtle); margin: 0; font-weight: 600; }
  .lbl { font-size: 9px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--fg-subtle); display: flex; flex-direction: column; gap: 4px; }
  .lbl input { width: 100%; font-family: var(--font-mono); }
  .err { color: var(--err); font-size: 11px; margin: 0; }
  .actions { display: flex; gap: 4px; }
  .actions button { padding: 4px 10px; font-size: 10px; flex: 1 1 auto; }
  .chart { flex: 1; min-height: 200px; background: var(--bg-elevated); border-radius: var(--radius-md); overflow: hidden; }
</style>
```

- [ ] **Step 4: Add the Plot entry to the activity bar**

Edit `src/lib/components/ActivityBar.svelte`. Update `ActivityKey` and the `items` array:

```typescript
export type ActivityKey = "connection" | "macros" | "scripts" | "filter" | "logs" | "plot";

const items: Array<{ key: ActivityKey; icon: string; title: string }> = [
  { key: "connection", icon: "⚙", title: "Connection" },
  { key: "macros",     icon: "▶", title: "Macros" },
  { key: "scripts",    icon: "{}", title: "Scripts" },
  { key: "filter",     icon: "⌕", title: "Filter" },
  { key: "logs",       icon: "≡", title: "Logs" },
  { key: "plot",       icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 3 3 21 21 21"/><polyline points="6 17 10 11 14 14 21 6"/></svg>', title: "Plot" },
];
```

- [ ] **Step 5: Wire the panel into App.svelte**

Edit `src/App.svelte`. Add the import:

```typescript
import PlotPanel from "$lib/components/panels/PlotPanel.svelte";
```

Update the panel switch — append a new branch:

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
{:else if active === "plot"}
  <PlotPanel />
{/if}
```

- [ ] **Step 6: Type check + build**

```bash
npm run check
npm run build
```

Both pass. The build summary now shows a separate uPlot chunk.

- [ ] **Step 7: Smoke**

```bash
cargo tauri dev
```

Open Plot panel. Set a regex (e.g., `v=(?<v>\d+)`). Click Start. With a port that emits `v=12\nv=34\n...`, the chart updates in real time. Pause/Resume work. Clear empties the ring. Window size change rebuilds the ring.

- [ ] **Step 8: Commit**

```bash
git add src/lib/plotState.ts src/lib/components/panels/PlotPanel.svelte src/lib/components/ActivityBar.svelte src/App.svelte src/lib/eventRouter.ts
git commit -m "feat(ui): plot panel with uPlot live regex extraction"
```

---

## Task 11: Plot CSV export

**Files:**
- Modify: `src/lib/components/panels/PlotPanel.svelte`

- [ ] **Step 1: Add a Save CSV button + handler**

Edit `src/lib/components/panels/PlotPanel.svelte`. Add to imports:

```typescript
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
```

Add an export function in the script block:

```typescript
async function exportCsv() {
  const ring = ringFor($activeTabId, $activeTab.plotCapacity);
  if (ring.size === 0) {
    panelError = "Nothing to export yet.";
    return;
  }
  const path = await save({
    title: "Save plot data as CSV",
    defaultPath: `rustcom_plot_${new Date().toISOString().replace(/[:.]/g, "-")}.csv`,
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!path) return;
  try {
    const csv = ring.toCsv(ring.seriesNames());
    await writeTextFile(path, csv);
  } catch (e) {
    panelError = `Save failed: ${String(e)}`;
  }
}
```

Add the button to the actions row:

```svelte
<div class="actions">
  {#if $activeTab.plotStartedAt === 0}
    <button class="primary" onclick={start}>Start</button>
  {:else if $activeTab.plotPaused}
    <button class="primary" onclick={() => pausePlot($activeTabId, false)}>Resume</button>
  {:else}
    <button onclick={() => pausePlot($activeTabId, true)}>Pause</button>
  {/if}
  <button onclick={clearAll}>Clear</button>
  <button onclick={exportCsv}>Save CSV…</button>
</div>
```

- [ ] **Step 2: Add the fs plugin to package.json + Cargo.toml**

`@tauri-apps/plugin-fs` is the JS side. The Rust counterpart `tauri-plugin-fs` needs to be added too.

Edit `package.json` `dependencies`:

```json
"@tauri-apps/plugin-fs": "^2.0.0"
```

Edit `src-tauri/Cargo.toml` `[dependencies]`:

```toml
tauri-plugin-fs = "2"
```

Edit `src-tauri/src/lib.rs`. Register the plugin alongside dialog (right after the dialog plugin line):

```rust
.plugin(tauri_plugin_fs::init())
```

Edit `src-tauri/capabilities/default.json`. Add to `permissions`:

```json
"fs:default",
"fs:allow-write-text-file"
```

Run:

```bash
npm install
cd src-tauri
cargo build
```

Both should be clean.

- [ ] **Step 3: Type check + build**

```bash
cd ..
npm run check
npm run build
```

Both pass.

- [ ] **Step 4: Smoke**

In the dev window, fill the plot with samples, click Save CSV…, pick a path, open the file. Should look like:

```
ts_ms,temp,hum
1000,24.7,55.0
2000,24.8,55.1
```

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/panels/PlotPanel.svelte package.json package-lock.json src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/lib.rs src-tauri/capabilities/default.json
git commit -m "feat(ui): export plot data to CSV via native save dialog"
```

---

## Task 12: Backend Lua `tabs` API

**Files:**
- Modify: `src-tauri/src/script/api.rs`

Add a `tabs` Lua table exposing operations against any open `TabId`. The `serial.*` and global `on_recv` continue to target the bound tab (backwards compatible).

- [ ] **Step 1: Add a new ScriptOut variant for per-tab `on_recv` registration**

In `src-tauri/src/script/api.rs`, extend the `ScriptOut` enum:

```rust
#[derive(Debug)]
pub enum ScriptOut {
    Send { tab_id: TabId, bytes: Vec<u8> },
    Toast { level: String, msg: String },
    Log { msg: String },
    /// Set or clear the per-tab on_recv hook in the engine.
    /// `key` is a stable identifier the engine uses as a HashMap key.
    SetTabOnRecv { tab_id: TabId, key: String },
    ClearTabOnRecv { tab_id: TabId },
}
```

- [ ] **Step 2: Install the `tabs` table**

Append inside `install_api`, after the existing `globals.set("delay", ...)?;` line:

```rust
// tabs API
let tabs_table = lua.create_table()?;

// tabs.send(tab_id, bytes)
let send_tab_out = out.clone();
tabs_table.set(
    "send",
    lua.create_function(move |_, args: (u32, Table)| {
        let (tab_id, bytes) = args;
        let mut buf = Vec::new();
        for v in bytes.sequence_values::<u8>() { buf.push(v?); }
        let _ = send_tab_out.send(ScriptOut::Send { tab_id, bytes: buf });
        Ok(())
    })?,
)?;

// tabs.send_text(tab_id, text)
let send_text_tab_out = out.clone();
tabs_table.set(
    "send_text",
    lua.create_function(move |_, args: (u32, String)| {
        let (tab_id, text) = args;
        let _ = send_text_tab_out.send(ScriptOut::Send { tab_id, bytes: text.into_bytes() });
        Ok(())
    })?,
)?;

// tabs.send_hex(tab_id, str)
let send_hex_tab_out = out.clone();
tabs_table.set(
    "send_hex",
    lua.create_function(move |_, args: (u32, String)| {
        let (tab_id, input) = args;
        match crate::hex::parse_hex_input(&input) {
            Ok(bytes) => {
                let _ = send_hex_tab_out.send(ScriptOut::Send { tab_id, bytes });
                Ok(())
            }
            Err(e) => Err(mlua::Error::external(e)),
        }
    })?,
)?;

// tabs.on_recv(tab_id, fn) — store fn under a stable key, signal engine to use it.
// We stash the function in the global table _tab_callbacks[tab_id] and tell the
// engine which key to dispatch via.
let on_recv_out = out.clone();
tabs_table.set(
    "on_recv",
    lua.create_function(move |lua_inner, args: (u32, Function)| {
        let (tab_id, cb) = args;
        let key = format!("_tab_recv_{tab_id}");
        let globals = lua_inner.globals();
        let registry: Table = match globals.get::<Value>("_tab_callbacks")? {
            Value::Table(t) => t,
            _ => {
                let t = lua_inner.create_table()?;
                globals.set("_tab_callbacks", t.clone())?;
                t
            }
        };
        registry.set(key.clone(), cb)?;
        let _ = on_recv_out.send(ScriptOut::SetTabOnRecv { tab_id, key });
        Ok(())
    })?,
)?;

// tabs.clear_on_recv(tab_id) — drop the per-tab callback.
let clear_recv_out = out;
tabs_table.set(
    "clear_on_recv",
    lua.create_function(move |lua_inner, tab_id: u32| {
        let key = format!("_tab_recv_{tab_id}");
        let globals = lua_inner.globals();
        if let Ok(Value::Table(reg)) = globals.get::<Value>("_tab_callbacks") {
            let _ = reg.set(key, mlua::Value::Nil);
        }
        let _ = clear_recv_out.send(ScriptOut::ClearTabOnRecv { tab_id });
        Ok(())
    })?,
)?;

globals.set("tabs", tabs_table)?;
```

(`out` is moved into the last closure — that's why earlier closures use `out.clone()`.)

- [ ] **Step 3: Add tests**

In the existing `#[cfg(test)] mod tests { ... }` block, add:

```rust
#[test]
fn tabs_send_forwards_bytes_to_specific_tab() {
    let (lua, rx) = fresh();
    lua.load("tabs.send(99, {0xAA, 0xBB})").exec().unwrap();
    match rx.recv().unwrap() {
        ScriptOut::Send { tab_id, bytes } => {
            assert_eq!(tab_id, 99);
            assert_eq!(bytes, vec![0xAA, 0xBB]);
        }
        other => panic!("expected Send, got {other:?}"),
    }
}

#[test]
fn tabs_send_text_targets_tab() {
    let (lua, rx) = fresh();
    lua.load(r#"tabs.send_text(42, "PING")"#).exec().unwrap();
    match rx.recv().unwrap() {
        ScriptOut::Send { tab_id, bytes } => {
            assert_eq!(tab_id, 42);
            assert_eq!(bytes, b"PING".to_vec());
        }
        other => panic!("expected Send, got {other:?}"),
    }
}

#[test]
fn tabs_on_recv_signals_engine() {
    let (lua, rx) = fresh();
    lua.load(
        r#"
        tabs.on_recv(7, function(b) end)
        "#,
    ).exec().unwrap();
    match rx.recv().unwrap() {
        ScriptOut::SetTabOnRecv { tab_id, key } => {
            assert_eq!(tab_id, 7);
            assert_eq!(key, "_tab_recv_7");
        }
        other => panic!("expected SetTabOnRecv, got {other:?}"),
    }
}

#[test]
fn tabs_clear_on_recv_signals_engine() {
    let (lua, rx) = fresh();
    lua.load(
        r#"
        tabs.on_recv(3, function(b) end)
        tabs.clear_on_recv(3)
        "#,
    ).exec().unwrap();
    // First message is the SetTabOnRecv from on_recv.
    rx.recv().unwrap();
    match rx.recv().unwrap() {
        ScriptOut::ClearTabOnRecv { tab_id } => assert_eq!(tab_id, 3),
        other => panic!("expected ClearTabOnRecv, got {other:?}"),
    }
}
```

- [ ] **Step 4: Test + build**

```bash
cd src-tauri
cargo test --lib script::api
```

Expected: 9 tests pass (5 existing + 4 new).

```bash
cargo build
```

Clean.

- [ ] **Step 5: Commit**

```bash
cd ..
git add src-tauri/src/script/api.rs
git commit -m "feat(backend): Lua tabs.* API with per-tab send/on_recv"
```

---

## Task 13: ScriptEngine multi-tab `on_recv` dispatch

**Files:**
- Modify: `src-tauri/src/script/engine.rs`

The engine already routes a single global `on_recv` against `bound_tab`. Now add a `HashMap<TabId, String>` of per-tab callback keys and dispatch them on every RX (regardless of `bound_tab`).

- [ ] **Step 1: Track per-tab callbacks**

Edit `src-tauri/src/script/engine.rs`. Update the imports:

```rust
use std::collections::HashMap;
```

In `run_engine`, replace:

```rust
let mut bound_tab: Option<TabId> = None;
```

with:

```rust
let mut bound_tab: Option<TabId> = None;
let mut tab_recv_keys: HashMap<TabId, String> = HashMap::new();
```

- [ ] **Step 2: Handle the new ScriptOut variants in the forwarder**

The forwarder is a separate thread — but `SetTabOnRecv` / `ClearTabOnRecv` need to reach the engine, not the forwarder. We change topology slightly: have the script engine's mpsc receive both `EngineCommand` AND messages from the script. Cleanest: keep the forwarder for `Send`/`Toast`/`Log` (output), and route `SetTabOnRecv`/`ClearTabOnRecv` back into the engine via a separate mpsc.

Edit the engine to add a back-channel. After the `out_tx` channel setup in `run_engine`, add:

```rust
// Side-channel for engine-bound updates from running scripts (registering per-tab on_recv).
let (engine_tx, engine_rx) = mpsc::channel::<ScriptOut>();
```

Update the forwarder spawn so it gets BOTH receivers? Simpler: split `ScriptOut` into output vs control at the api.rs level. Actually let's make the change minimal — keep `ScriptOut` shared, but the forwarder filters out the control variants and re-dispatches them to `engine_tx`:

In `forward_loop`, add a parameter for the engine side-channel sender, and route:

Replace the `forward_loop` function body to:

```rust
fn forward_loop(rx: Receiver<ScriptOut>, engine_back: Sender<ScriptOut>, app: AppHandle) {
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
            ctrl @ (ScriptOut::SetTabOnRecv { .. } | ScriptOut::ClearTabOnRecv { .. }) => {
                let _ = engine_back.send(ctrl);
            }
        }
    }
}
```

Update the `thread::Builder...spawn(...)` call for the forwarder to pass `engine_tx.clone()`:

```rust
let app_forward = app.clone();
let engine_back = engine_tx.clone();
thread::Builder::new()
    .name("rustcom-script-forwarder".to_string())
    .spawn(move || forward_loop(out_rx, engine_back, app_forward))
    .ok();
```

- [ ] **Step 3: Process control messages in the engine loop**

Replace the engine's `loop { match rx.recv() ... }` with a select-style loop that polls both `rx` (commands from frontend) and `engine_rx` (control from scripts). Since std mpsc has no select, we use try_recv with a small sleep to keep latency tolerable:

```rust
loop {
    // Drain control messages from running scripts first (cheap).
    while let Ok(ctrl) = engine_rx.try_recv() {
        match ctrl {
            ScriptOut::SetTabOnRecv { tab_id, key } => {
                tab_recv_keys.insert(tab_id, key);
            }
            ScriptOut::ClearTabOnRecv { tab_id } => {
                tab_recv_keys.remove(&tab_id);
            }
            _ => {}
        }
    }

    // Block briefly on the main command channel.
    let cmd = match rx.recv_timeout(std::time::Duration::from_millis(50)) {
        Ok(c) => c,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
        Err(_) => return,
    };

    match cmd {
        EngineCommand::Run { tab_id, source } => {
            let _ = lua.globals().set("on_recv", mlua::Value::Nil);
            // Drop all per-tab callbacks from a previous script.
            if let Ok(mlua::Value::Table(reg)) = lua.globals().get::<mlua::Value>("_tab_callbacks") {
                // Clear by replacing with an empty table.
                drop(reg);
                let empty = lua.create_table().ok();
                if let Some(t) = empty {
                    let _ = lua.globals().set("_tab_callbacks", t);
                }
            }
            tab_recv_keys.clear();
            if let Err(e) = install_api(&lua, tab_id, out_tx.clone()) {
                let _ = app.emit("script_error", format!("install_api failed: {e}"));
                continue;
            }
            if let Err(e) = lua.load(&source).exec() {
                let _ = app.emit("script_error", format!("{e}"));
                continue;
            }
            bound_tab = Some(tab_id);
        }
        EngineCommand::Stop => {
            let _ = lua.globals().set("on_recv", mlua::Value::Nil);
            if let Ok(mlua::Value::Table(_)) = lua.globals().get::<mlua::Value>("_tab_callbacks") {
                let empty = lua.create_table().ok();
                if let Some(t) = empty { let _ = lua.globals().set("_tab_callbacks", t); }
            }
            tab_recv_keys.clear();
            bound_tab = None;
        }
        EngineCommand::OnRecv { tab_id, bytes } => {
            // 1) Per-tab callback dispatch.
            if let Some(key) = tab_recv_keys.get(&tab_id).cloned() {
                if let Ok(mlua::Value::Table(reg)) = lua.globals().get::<mlua::Value>("_tab_callbacks") {
                    if let Ok(mlua::Value::Function(cb)) = reg.get::<mlua::Value>(key.as_str()) {
                        if let Ok(table) = lua.create_sequence_from(bytes.iter().copied()) {
                            if let Err(e) = cb.call::<()>(table) {
                                let _ = app.emit("script_error", format!("tabs.on_recv: {e}"));
                            }
                        }
                    }
                }
            }
            // 2) Global bound on_recv (back-compat).
            if bound_tab == Some(tab_id) {
                if let Ok(mlua::Value::Function(cb)) = lua.globals().get::<mlua::Value>("on_recv") {
                    if let Ok(table) = lua.create_sequence_from(bytes.into_iter()) {
                        if let Err(e) = cb.call::<()>(table) {
                            let _ = app.emit("script_error", format!("on_recv: {e}"));
                        }
                    }
                }
            }
        }
        EngineCommand::Shutdown => return,
    }
}
```

(Note: this replaces the entire previous `loop { match rx.recv() {...} }` body. Read the existing file before editing to make sure the surrounding context is preserved.)

- [ ] **Step 4: Test**

```bash
cd src-tauri
cargo test --lib script
```

Expected: all script tests still pass (api: 9, manager remains 3, etc.).

```bash
cargo build
```

Clean.

- [ ] **Step 5: Smoke (optional, but worth it)**

```bash
cd ..
cargo tauri dev
```

Open two tabs, connect both. In Scripts panel, write:

```lua
tabs.on_recv(1, function(b)
  log("tab 1 saw " .. #b .. " bytes")
end)
tabs.on_recv(2, function(b)
  log("tab 2 saw " .. #b .. " bytes")
end)
```

Run. Send data to either tab — corresponding log line fires.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/script/engine.rs
git commit -m "feat(backend): ScriptEngine routes per-tab Lua on_recv callbacks"
```

---

## Task 14: Backend session restore in settings.rs

**Files:**
- Modify: `src-tauri/src/commands/settings.rs`

Add `last_session: Option<SessionSnapshot>` to `Settings`. The snapshot stores per-tab `PortConfig`s (already serializable).

- [ ] **Step 1: Extend the Settings struct**

Edit `src-tauri/src/commands/settings.rs`. Add imports:

```rust
use crate::port::config::PortConfig;
```

Add types:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTab {
    pub config: PortConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionSnapshot {
    #[serde(default)]
    pub tabs: Vec<SessionTab>,
}
```

Update `Settings`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub defaults: DefaultsConfig,
    #[serde(default)]
    pub session_restore_enabled: bool,
    #[serde(default)]
    pub last_session: SessionSnapshot,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            defaults: DefaultsConfig::default(),
            session_restore_enabled: false,
            last_session: SessionSnapshot::default(),
        }
    }
}
```

The existing `load_settings` and `save_settings` continue to work (round-trip the new fields automatically).

Add a more focused command for session writes (so the frontend doesn't have to bundle defaults every time it persists tabs):

```rust
#[tauri::command]
pub fn save_session(snapshot: SessionSnapshot) -> AppResult<Settings> {
    let mut s: Settings = storage::read_json(FILE)?;
    s.last_session = snapshot;
    storage::write_json(FILE, &s)?;
    Ok(s)
}

#[tauri::command]
pub fn set_session_restore_enabled(enabled: bool) -> AppResult<Settings> {
    let mut s: Settings = storage::read_json(FILE)?;
    s.session_restore_enabled = enabled;
    storage::write_json(FILE, &s)?;
    Ok(s)
}
```

- [ ] **Step 2: Register the new commands**

Edit `src-tauri/src/lib.rs` invoke_handler list:

```rust
            commands::settings::save_session,
            commands::settings::set_session_restore_enabled,
```

- [ ] **Step 3: Add tests**

Append to the `#[cfg(test)] mod tests` block in settings.rs:

```rust
#[test]
fn settings_default_session_restore_off() {
    let s = Settings::default();
    assert!(!s.session_restore_enabled);
    assert!(s.last_session.tabs.is_empty());
}

#[test]
fn settings_with_session_roundtrip() {
    let mut s = Settings::default();
    s.session_restore_enabled = true;
    s.last_session.tabs.push(SessionTab {
        config: PortConfig {
            port_name: "COM5".into(),
            baud_rate: 115200,
            data_bits: DataBits::Eight,
            stop_bits: StopBits::One,
            parity: Parity::None,
            flow_control: FlowControl::None,
            auto_reconnect: false,
            reconnect_delay_ms: 2000,
        },
    });
    let json = serde_json::to_string(&s).unwrap();
    let back: Settings = serde_json::from_str(&json).unwrap();
    assert!(back.session_restore_enabled);
    assert_eq!(back.last_session.tabs.len(), 1);
    assert_eq!(back.last_session.tabs[0].config.port_name, "COM5");
}
```

- [ ] **Step 4: Test + build**

```bash
cd src-tauri
cargo test --lib settings
```

Expected: 4 tests pass (2 existing + 2 new).

```bash
cargo build
```

Clean.

- [ ] **Step 5: Commit**

```bash
cd ..
git add src-tauri/src/commands/settings.rs src-tauri/src/lib.rs
git commit -m "feat(backend): session snapshot + restore toggle in settings.json"
```

---

## Task 15: Frontend session restore on app start

**Files:**
- Modify: `src/lib/ipc.ts`
- Modify: `src/lib/stores/settings.ts`
- Modify: `src/lib/stores/tabs.ts`
- Modify: `src/App.svelte`
- Modify: `src/lib/components/panels/ConnectionPanel.svelte` (add toggle)

- [ ] **Step 1: ipc + settings store updates**

Edit `src/lib/ipc.ts`. Add types:

```typescript
export interface SessionTab {
  config: PortConfig;
}
export interface SessionSnapshot {
  tabs: SessionTab[];
}
```

Update Settings:

```typescript
export interface Settings {
  defaults: DefaultsConfig;
  session_restore_enabled: boolean;
  last_session: SessionSnapshot;
}
```

Add to ipc:

```typescript
  saveSession: (snapshot: SessionSnapshot) =>
    invoke<Settings>("save_session", { snapshot }),
  setSessionRestoreEnabled: (enabled: boolean) =>
    invoke<Settings>("set_session_restore_enabled", { enabled }),
```

Edit `src/lib/stores/settings.ts`. Update the `initial` constant:

```typescript
const initial: Settings = {
  defaults: { /* unchanged */ },
  session_restore_enabled: false,
  last_session: { tabs: [] },
};
```

Add helpers:

```typescript
export async function persistSession(snapshot: SessionSnapshot) {
  const saved = await ipc.saveSession(snapshot);
  settings.set(saved);
}

export async function persistSessionRestore(enabled: boolean) {
  const saved = await ipc.setSessionRestoreEnabled(enabled);
  settings.set(saved);
}
```

(Also `import type { SessionSnapshot } from "$lib/ipc";` at the top.)

- [ ] **Step 2: Tab restore in tabs.ts**

Edit `src/lib/stores/tabs.ts`. Add:

```typescript
import type { PortConfig } from "$lib/ipc";

export function restoreTabs(configs: PortConfig[]) {
  if (configs.length === 0) return;
  const restored: Tab[] = configs.map((cfg) => {
    const t = defaultTab();
    t.config = cfg;
    return t;
  });
  tabs.set(restored);
  activeTabId.set(restored[0].id);
}

export function snapshotTabsForSession(): PortConfig[] {
  return get(tabs).map((t) => t.config);
}
```

- [ ] **Step 3: App.svelte — restore on mount, save on close**

Edit `src/App.svelte`. Add to imports:

```typescript
import { persistSession, settings as settingsStore } from "$lib/stores/settings";
import { restoreTabs, snapshotTabsForSession } from "$lib/stores/tabs";
```

In `onMount` (after `await loadSettings()`), restore if enabled:

```typescript
onMount(async () => {
  await loadSettings();
  const s = get(settingsStore);
  if (s.session_restore_enabled && s.last_session.tabs.length > 0) {
    restoreTabs(s.last_session.tabs.map((t) => t.config));
  }
  startPolling();
  startEventRouter();
  // ... rest unchanged
});
```

(Add `import { get } from "svelte/store";` if not already imported.)

Add a `beforeunload` handler that persists the current tabs:

```typescript
let unBeforeUnload: (() => void) | null = null;

onMount(async () => {
  // ... existing ...
  const handler = () => {
    if (get(settingsStore).session_restore_enabled) {
      const snap = { tabs: snapshotTabsForSession().map((c) => ({ config: c })) };
      // Fire-and-forget; the page is closing.
      void persistSession(snap);
    }
  };
  window.addEventListener("beforeunload", handler);
  unBeforeUnload = () => window.removeEventListener("beforeunload", handler);
});

onDestroy(() => {
  if (unBeforeUnload) unBeforeUnload();
  // ... existing ...
});
```

- [ ] **Step 4: ConnectionPanel toggle**

Edit `src/lib/components/panels/ConnectionPanel.svelte`. Add to imports:

```typescript
import { settings, persistSessionRestore } from "$lib/stores/settings";
```

Add a section at the bottom of the `.panel` div (below the existing Signals block):

```svelte
<h4 class="signals">Session</h4>
<label class="toggle">
  <input
    type="checkbox"
    checked={$settings.session_restore_enabled}
    onchange={(e) => persistSessionRestore((e.currentTarget as HTMLInputElement).checked)}
  />
  Restore tabs on next launch
</label>
```

(The `.toggle` style was added in Plan 3 Task 8 already.)

- [ ] **Step 5: Type check + build**

```bash
npm run check
npm run build
```

Both pass.

- [ ] **Step 6: Smoke**

`cargo tauri dev`. In the Connection panel toggle "Restore tabs on next launch". Open 2-3 tabs with different port settings. Close the window. Relaunch — same tabs reappear in disconnected state. Toggle the option off, repeat — single fresh tab on relaunch.

- [ ] **Step 7: Commit**

```bash
git add src/lib/ipc.ts src/lib/stores/settings.ts src/lib/stores/tabs.ts src/App.svelte src/lib/components/panels/ConnectionPanel.svelte
git commit -m "feat(ui): opt-in session restore"
```

---

## Task 16: Bump version + final smoke + tag

**Files:**
- Modify: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `package.json`
- Modify: `README.md` (mention plot view + multi-tab Lua)

- [ ] **Step 1: Bump versions**

In all three files: `0.2.0` → `0.3.0`.

- [ ] **Step 2: README — add new feature lines**

Edit the Features section of `README.md`. Insert after the existing "Lua scripting" line:

```markdown
- **Live plot view** — define a regex with named capture groups (e.g. `temp:(?<temp>\d+\.\d+)`) and watch RX values render as a streaming line chart. Pause / Resume / Save CSV.
- **Multi-tab Lua** — `tabs.send(id, ...)`, `tabs.send_text(id, "AT")`, `tabs.on_recv(id, fn)` so one script can drive several ports.
- **Session restore** — opt-in: open tabs are remembered across restarts (disconnected on relaunch).
```

- [ ] **Step 3: Build release**

```bash
cargo tauri build
```

Expected artifacts: `RustCOM_0.3.0_x64-setup.exe` and `.msi`.

- [ ] **Step 4: Smoke checklist**

`cargo tauri dev`, walk:

- [ ] Plot panel: regex pattern with two named groups → both lines render. Pause/Resume. Save CSV opens dialog and writes file.
- [ ] Multi-tab Lua: open 2 tabs, run a script using `tabs.on_recv(1, ...)` and `tabs.on_recv(2, ...)`, send to each → corresponding callbacks fire.
- [ ] Session restore toggle ON → close → reopen → tabs return.
- [ ] Session restore toggle OFF → close → reopen → blank slate.
- [ ] Logs panel inline preview shows entries as they arrive.
- [ ] Toast announces via screen reader (if you have one handy).
- [ ] Tab strip shows "retry N" while reconnecting.

- [ ] **Step 5: Commit + tag**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json package.json README.md
git commit -m "chore: bump version to 0.3.0 and document plot/multi-tab/session features"
git tag v0.3.0
git tag plan-4-complete
```

(No push; user merges and pushes when ready.)

---

## Out of scope (potential future work)

- File transfer (XMODEM / YMODEM / ZMODEM).
- Built-in protocol decoders (Modbus RTU, MQTT-over-serial).
- Plot dual-Y axes (currently single shared Y).
- Plot persisted alongside session restore (regex pattern restored, but ring is wiped on restart).
- Lua: `tabs.list()` returning open-tab metadata. Easy follow-up if needed.
- Lua: `serial.read(n, timeout_ms)` for synchronous reads inside a script (would need careful blocking semantics).
