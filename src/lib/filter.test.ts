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
