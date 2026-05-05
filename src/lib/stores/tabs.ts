import { writable, derived, get, type Readable, type Writable } from "svelte/store";
import type { LineEnding, PortConfig, SendMode, TabId } from "$lib/ipc";

export type ViewMode = "ascii" | "hex" | "both";
export type ConnState = "disconnected" | "connecting" | "connected" | "reconnecting";

export const MAX_BUFFER_SIZE = 100_000;
export const BUFFER_DRAIN_SIZE = 10_000;

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
}

export const SOLE_TAB_ID: TabId = 1;

function defaultConfig(): PortConfig {
  return {
    port_name: "",
    baud_rate: 9600,
    data_bits: "eight",
    stop_bits: "one",
    parity: "none",
    flow_control: "none",
    auto_reconnect: false,
    reconnect_delay_ms: 2000,
  };
}

function defaultTab(): Tab {
  return {
    id: SOLE_TAB_ID,
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
  };
}

export const tabs: Writable<Tab[]> = writable([defaultTab()]);
export const activeTabId: Writable<TabId> = writable(SOLE_TAB_ID);

export const activeTab: Readable<Tab> = derived(
  [tabs, activeTabId],
  ([$tabs, $id]) => $tabs.find((t) => t.id === $id) ?? defaultTab()
);

export function patchActiveTab(patch: Partial<Tab>) {
  const id = get(activeTabId);
  tabs.update((list) => list.map((t) => (t.id === id ? { ...t, ...patch } : t)));
}

export type Direction = "rx" | "tx";

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
