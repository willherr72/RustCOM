import { describe, it, expect, beforeEach } from "vitest";
import { get } from "svelte/store";
import {
  tabs,
  activeTabId,
  activeTab,
  appendBytes,
  clearBuffer,
  SOLE_TAB_ID,
  MAX_BUFFER_SIZE,
  BUFFER_DRAIN_SIZE,
} from "./tabs";
import { pushHistory, navigateHistory } from "./tabs";

describe("tabs store", () => {
  beforeEach(() => {
    tabs.set([
      {
        id: SOLE_TAB_ID,
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
    activeTabId.set(SOLE_TAB_ID);
  });

  it("activeTab follows activeTabId", () => {
    const t = get(activeTab);
    expect(t.id).toBe(SOLE_TAB_ID);
  });

  it("appendBytes grows the buffer and bumps rxBytes", () => {
    appendBytes(SOLE_TAB_ID, new Uint8Array([0x41, 0x42, 0x43]));
    const t = get(activeTab);
    expect(Array.from(t.buffer)).toEqual([0x41, 0x42, 0x43]);
    expect(t.rxBytes).toBe(3);
  });

  it("appendBytes drains when buffer would exceed MAX_BUFFER_SIZE", () => {
    const big = new Uint8Array(MAX_BUFFER_SIZE);
    big.fill(0x55);
    appendBytes(SOLE_TAB_ID, big);
    appendBytes(SOLE_TAB_ID, new Uint8Array([1, 2, 3]));
    const t = get(activeTab);
    expect(t.buffer.length).toBe(MAX_BUFFER_SIZE - BUFFER_DRAIN_SIZE + 3);
    expect(t.rxBytes).toBe(MAX_BUFFER_SIZE + 3);
  });

  it("appendBytes with direction='tx' bumps txBytes not rxBytes", () => {
    appendBytes(SOLE_TAB_ID, new Uint8Array([0x41, 0x42, 0x43]), "tx");
    const t = get(activeTab);
    expect(t.txBytes).toBe(3);
    expect(t.rxBytes).toBe(0);
    expect(Array.from(t.buffer)).toEqual([0x41, 0x42, 0x43]);
  });

  it("clearBuffer resets the buffer but not the counter", () => {
    appendBytes(SOLE_TAB_ID, new Uint8Array([1, 2, 3]));
    clearBuffer(SOLE_TAB_ID);
    const t = get(activeTab);
    expect(t.buffer.length).toBe(0);
    expect(t.rxBytes).toBe(3);
  });

  it("pushHistory dedupes consecutive duplicates", () => {
    pushHistory(SOLE_TAB_ID, "AT");
    pushHistory(SOLE_TAB_ID, "AT");
    pushHistory(SOLE_TAB_ID, "STATUS");
    expect(get(activeTab).history).toEqual(["AT", "STATUS"]);
  });

  it("navigateHistory walks up then down", () => {
    pushHistory(SOLE_TAB_ID, "one");
    pushHistory(SOLE_TAB_ID, "two");
    pushHistory(SOLE_TAB_ID, "three");
    expect(navigateHistory(SOLE_TAB_ID, "up")).toBe("three");
    expect(navigateHistory(SOLE_TAB_ID, "up")).toBe("two");
    expect(navigateHistory(SOLE_TAB_ID, "up")).toBe("one");
    expect(navigateHistory(SOLE_TAB_ID, "up")).toBe("one"); // clamped
    expect(navigateHistory(SOLE_TAB_ID, "down")).toBe("two");
    expect(navigateHistory(SOLE_TAB_ID, "down")).toBe("three");
    expect(navigateHistory(SOLE_TAB_ID, "down")).toBe("");
  });
});
