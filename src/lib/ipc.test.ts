import { describe, it, expect, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args: unknown) => Promise.resolve({ cmd, args })),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => undefined)),
}));

import { ipc } from "./ipc";

describe("ipc wrapper", () => {
  it("openPort forwards camelCase args", async () => {
    const result = (await ipc.openPort(1, {
      port_name: "COM3",
      baud_rate: 9600,
      data_bits: "eight",
      stop_bits: "one",
      parity: "none",
      flow_control: "none",
    })) as unknown as { cmd: string; args: { tabId: number; config: { port_name: string } } };
    expect(result.cmd).toBe("open_port");
    expect(result.args.tabId).toBe(1);
    expect(result.args.config.port_name).toBe("COM3");
  });

  it("sendText carries lineEnding", async () => {
    const result = (await ipc.sendText(2, "AT", "crlf")) as unknown as {
      args: { tabId: number; text: string; lineEnding: string };
    };
    expect(result.args.lineEnding).toBe("crlf");
  });
});
