import { describe, it, expect } from "vitest";
import { extract, SampleRing } from "./plotExtractor";

describe("extract", () => {
  it("returns no samples for empty pattern", () => {
    const r = extract("temp:25.0\n", "", 0, 1000);
    expect(r.samples).toEqual([]);
    expect(r.invalid).toBeNull();
  });

  it("extracts one named group", () => {
    const r = extract("temp:25.5\n", "temp:(?<temp>\\d+\\.\\d+)", 0, 2000);
    expect(r.samples.length).toBe(1);
    expect(r.samples[0].ts).toBe(2000);
    expect(r.samples[0].values).toEqual({ temp: 25.5 });
  });

  it("extracts multiple named groups", () => {
    const r = extract(
      "temp:24.7 hum:55.0\n",
      "temp:(?<temp>\\d+\\.\\d+) hum:(?<hum>\\d+\\.\\d+)",
      0,
      1000,
    );
    expect(r.samples[0].values).toEqual({ temp: 24.7, hum: 55.0 });
  });

  it("counts unmatched non-empty lines", () => {
    const r = extract("nope\nyep:42\n", "yep:(?<v>\\d+)", 0, 0);
    expect(r.unmatchedLines).toBe(1);
    expect(r.samples.length).toBe(1);
  });

  it("flags invalid regex without throwing", () => {
    const r = extract("hi", "(", 0, 0);
    expect(r.invalid).not.toBeNull();
    expect(r.samples).toEqual([]);
  });

  it("skips groups that don't parse as numbers", () => {
    const r = extract("a:abc b:5", "a:(?<a>\\w+) b:(?<b>\\d+)", 0, 0);
    expect(r.samples[0].values).toEqual({ b: 5 });
  });
});

describe("SampleRing", () => {
  it("pushes up to capacity then drops oldest", () => {
    const ring = new SampleRing(3);
    ring.push({ ts: 1, values: { a: 1 } });
    ring.push({ ts: 2, values: { a: 2 } });
    ring.push({ ts: 3, values: { a: 3 } });
    ring.push({ ts: 4, values: { a: 4 } });
    expect(ring.size).toBe(3);
    expect(ring.toArray().map((s) => s.ts)).toEqual([2, 3, 4]);
  });

  it("seriesNames returns union sorted", () => {
    const ring = new SampleRing(10);
    ring.push({ ts: 1, values: { temp: 1 } });
    ring.push({ ts: 2, values: { hum: 2 } });
    ring.push({ ts: 3, values: { temp: 3, hum: 4 } });
    expect(ring.seriesNames()).toEqual(["hum", "temp"]);
  });

  it("toUPlotData emits columnar arrays in seconds", () => {
    const ring = new SampleRing(10);
    ring.push({ ts: 1000, values: { a: 1 } });
    ring.push({ ts: 2000, values: { a: 2, b: 5 } });
    const [xs, ays, bys] = ring.toUPlotData(["a", "b"]);
    expect(xs).toEqual([1, 2]);
    expect(ays).toEqual([1, 2]);
    expect(bys[0]).toBeNaN();
    expect(bys[1]).toBe(5);
  });

  it("toCsv emits header + rows", () => {
    const ring = new SampleRing(10);
    ring.push({ ts: 0, values: { a: 1, b: 2 } });
    ring.push({ ts: 100, values: { a: 3 } });
    const csv = ring.toCsv(["a", "b"]);
    expect(csv).toBe("ts_ms,a,b\n0,1,2\n100,3,\n");
  });
});
