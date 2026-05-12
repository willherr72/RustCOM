import { describe, it, expect } from "vitest";
import { findMatches } from "./search";

describe("findMatches", () => {
  it("returns no matches for empty pattern", () => {
    expect(findMatches("hello", "", false).matches).toEqual([]);
  });

  it("plain substring is case-insensitive", () => {
    const r = findMatches("HelloHELLO", "hello", false);
    expect(r.matches).toEqual([
      { start: 0, end: 5 },
      { start: 5, end: 10 },
    ]);
  });

  it("regex mode honors flags and returns ranges", () => {
    const r = findMatches("foo123bar45", "\\d+", true);
    expect(r.matches.map((m) => [m.start, m.end])).toEqual([
      [3, 6],
      [9, 11],
    ]);
  });

  it("invalid regex flagged without throwing", () => {
    const r = findMatches("hi", "(", true);
    expect(r.invalid).not.toBeNull();
    expect(r.matches).toEqual([]);
  });
});
