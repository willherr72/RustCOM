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
