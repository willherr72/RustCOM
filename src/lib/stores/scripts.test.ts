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
