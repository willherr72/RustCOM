import { writable, derived, get, type Readable, type Writable } from "svelte/store";
import type { Direction, LineEnding, PortConfig, SendMode, TabId } from "$lib/ipc";
import { snapshotDefaults } from "$lib/stores/settings";

export type { Direction };

export interface LogEntry {
  ts_ms: number;
  direction: Direction;
  bytes: number[];
}

export type ViewMode = "ascii" | "hex" | "both";
export type ConnState = "disconnected" | "connecting" | "connected" | "reconnecting";

export const MAX_BUFFER_SIZE = 100_000;
export const BUFFER_DRAIN_SIZE = 10_000;
export const HISTORY_MAX = 200;

export interface Tab {
  id: TabId;
  config: PortConfig;
  state: ConnState;
  buffer: Uint8Array;
  rxBytes: number;
  txBytes: number;
  viewMode: ViewMode;
  stripAnsi: boolean;
  autoScroll: boolean;
  sendMode: SendMode;
  lineEnding: LineEnding;
  dtr: boolean;
  rts: boolean;
  errorMessage: string | null;
  reconnectAttempts: number;
  history: string[];
  historyIndex: number; // -1 == not navigating
  filterEnabled: boolean;
  filterPattern: string;
  loggingEnabled: boolean;
  logEntries: LogEntry[];
  searchOpen: boolean;
  searchPattern: string;
  searchUseRegex: boolean;
  searchMatchIndex: number; // -1 when no matches
  plotPattern: string;
  plotPaused: boolean;
  plotStartedAt: number; // ms wall clock; 0 means not started
  plotCapacity: number;
}

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

let nextTabId: TabId = 1;

function allocTabId(): TabId {
  return nextTabId++;
}

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
    reconnectAttempts: 0,
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
    plotPattern: "",
    plotPaused: false,
    plotStartedAt: 0,
    plotCapacity: 1000,
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

export const activeTab: Readable<Tab> = derived(
  [tabs, activeTabId],
  ([$tabs, $id]) => $tabs.find((t) => t.id === $id) ?? defaultTab()
);

export function patchActiveTab(patch: Partial<Tab>) {
  const id = get(activeTabId);
  tabs.update((list) => list.map((t) => (t.id === id ? { ...t, ...patch } : t)));
}

export function appendBytes(id: TabId, bytes: Uint8Array, direction: Direction = "rx") {
  tabs.update((list) =>
    list.map((t) => {
      if (t.id !== id) return t;
      let combined: Uint8Array;
      if (t.buffer.length + bytes.length > MAX_BUFFER_SIZE) {
        const drainStart = Math.min(BUFFER_DRAIN_SIZE, t.buffer.length);
        const kept = t.buffer.subarray(drainStart);
        combined = new Uint8Array(kept.length + bytes.length);
        combined.set(kept, 0);
        combined.set(bytes, kept.length);
      } else {
        combined = new Uint8Array(t.buffer.length + bytes.length);
        combined.set(t.buffer, 0);
        combined.set(bytes, t.buffer.length);
      }
      const next = { ...t, buffer: combined };
      if (direction === "rx") next.rxBytes = t.rxBytes + bytes.length;
      else next.txBytes = t.txBytes + bytes.length;
      return next;
    })
  );
}

export function clearBuffer(id: TabId) {
  tabs.update((list) =>
    list.map((t) => (t.id === id ? { ...t, buffer: new Uint8Array(0) } : t))
  );
}

export function pushHistory(id: TabId, entry: string) {
  if (!entry) return;
  tabs.update((list) =>
    list.map((t) => {
      if (t.id !== id) return t;
      let h = t.history;
      if (h[h.length - 1] === entry) {
        h = [...h]; // no duplicate consecutive entry
      } else {
        h = [...h, entry];
        if (h.length > HISTORY_MAX) h = h.slice(h.length - HISTORY_MAX);
      }
      return { ...t, history: h, historyIndex: -1 };
    })
  );
}

export function navigateHistory(id: TabId, direction: "up" | "down"): string | null {
  let result: string | null = null;
  tabs.update((list) =>
    list.map((t) => {
      if (t.id !== id) return t;
      const h = t.history;
      if (h.length === 0) return t;
      let idx = t.historyIndex;
      if (direction === "up") {
        idx = idx === -1 ? h.length - 1 : Math.max(0, idx - 1);
      } else {
        if (idx === -1) return t;
        idx = idx + 1;
        if (idx >= h.length) {
          result = "";
          return { ...t, historyIndex: -1 };
        }
      }
      result = h[idx];
      return { ...t, historyIndex: idx };
    })
  );
  return result;
}

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
