import { describe, it, expect } from "vitest";
import { formatHex, stripAnsi, bytesToText } from "./format";

describe("formatHex", () => {
  it("formats a short buffer", () => {
    const out = formatHex(new Uint8Array([0x41, 0x42]));
    expect(out.startsWith("0000  41 42 ")).toBe(true);
    expect(out.trim().endsWith("AB")).toBe(true);
  });

  it("emits two rows for 32 bytes", () => {
    const data = new Uint8Array(32);
    for (let i = 0; i < 32; i++) data[i] = i;
    const lines = formatHex(data).trimEnd().split("\n");
    expect(lines.length).toBe(2);
    expect(lines[0].startsWith("0000  ")).toBe(true);
    expect(lines[1].startsWith("0010  ")).toBe(true);
  });
});

describe("stripAnsi", () => {
  it("removes color escapes", () => {
    expect(stripAnsi("\x1b[31mred\x1b[0m")).toBe("red");
  });
  it("passes plain text", () => {
    expect(stripAnsi("hi")).toBe("hi");
  });
});

describe("bytesToText", () => {
  it("decodes utf-8", () => {
    const bytes = new TextEncoder().encode("AT+VERSION");
    expect(bytesToText(bytes, true)).toBe("AT+VERSION");
  });
  it("strips ANSI when asked", () => {
    const bytes = new TextEncoder().encode("\x1b[32mok\x1b[0m");
    expect(bytesToText(bytes, true)).toBe("ok");
    expect(bytesToText(bytes, false)).toContain("\x1b");
  });
});
