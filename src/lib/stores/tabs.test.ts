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
  pushLog,
  patchActiveTab,
  type Tab,
} from "./tabs";

describe("tabs store", () => {
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
      },
    ]);
    activeTabId.set(1);
  });

  it("activeTab follows activeTabId", () => {
    const t = get(activeTab);
    expect(t.id).toBe(1);
  });

  it("appendBytes grows the buffer and bumps rxBytes", () => {
    appendBytes(1, new Uint8Array([0x41, 0x42, 0x43]));
    const t = get(activeTab);
    expect(Array.from(t.buffer)).toEqual([0x41, 0x42, 0x43]);
    expect(t.rxBytes).toBe(3);
  });

  it("appendBytes drains when buffer would exceed MAX_BUFFER_SIZE", () => {
    const big = new Uint8Array(MAX_BUFFER_SIZE);
    big.fill(0x55);
    appendBytes(1, big);
    appendBytes(1, new Uint8Array([1, 2, 3]));
    const t = get(activeTab);
    expect(t.buffer.length).toBe(MAX_BUFFER_SIZE - BUFFER_DRAIN_SIZE + 3);
    expect(t.rxBytes).toBe(MAX_BUFFER_SIZE + 3);
  });

  it("appendBytes with direction='tx' bumps txBytes not rxBytes", () => {
    appendBytes(1, new Uint8Array([0x41, 0x42, 0x43]), "tx");
    const t = get(activeTab);
    expect(t.txBytes).toBe(3);
    expect(t.rxBytes).toBe(0);
    expect(Array.from(t.buffer)).toEqual([0x41, 0x42, 0x43]);
  });

  it("clearBuffer resets the buffer but not the counter", () => {
    appendBytes(1, new Uint8Array([1, 2, 3]));
    clearBuffer(1);
    const t = get(activeTab);
    expect(t.buffer.length).toBe(0);
    expect(t.rxBytes).toBe(3);
  });

  it("pushHistory dedupes consecutive duplicates", () => {
    pushHistory(1, "AT");
    pushHistory(1, "AT");
    pushHistory(1, "STATUS");
    expect(get(activeTab).history).toEqual(["AT", "STATUS"]);
  });

  it("navigateHistory walks up then down", () => {
    pushHistory(1, "one");
    pushHistory(1, "two");
    pushHistory(1, "three");
    expect(navigateHistory(1, "up")).toBe("three");
    expect(navigateHistory(1, "up")).toBe("two");
    expect(navigateHistory(1, "up")).toBe("one");
    expect(navigateHistory(1, "up")).toBe("one"); // clamped
    expect(navigateHistory(1, "down")).toBe("two");
    expect(navigateHistory(1, "down")).toBe("three");
    expect(navigateHistory(1, "down")).toBe("");
  });

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
});
