export interface Sample {
  ts: number;             // ms since plot reset (relative)
  values: Record<string, number>;
}

export interface ExtractResult {
  samples: Sample[];
  invalid: string | null;
  unmatchedLines: number;
}

/**
 * Extract numeric samples from text using a regex with named capture groups.
 *
 * Each match produces one Sample. Each named group whose captured string parses
 * as a finite number becomes a series entry. Groups that don't parse are
 * silently skipped (so partial matches don't blow up the whole sample).
 *
 * `startTs` is the wall-clock ms at plot start; sample `ts` is offset from it.
 * `nowMs` is the wall-clock ms when the text arrived.
 */
export function extract(
  text: string,
  pattern: string,
  startTs: number,
  nowMs: number,
): ExtractResult {
  if (!pattern) return { samples: [], invalid: null, unmatchedLines: 0 };
  let re: RegExp;
  try {
    re = new RegExp(pattern, "g");
  } catch (e) {
    return { samples: [], invalid: (e as Error).message, unmatchedLines: 0 };
  }
  const samples: Sample[] = [];
  let unmatchedLines = 0;
  for (const line of text.split("\n")) {
    re.lastIndex = 0;
    const m = re.exec(line);
    if (!m || !m.groups) {
      if (line.length > 0) unmatchedLines += 1;
      continue;
    }
    const values: Record<string, number> = {};
    for (const [name, raw] of Object.entries(m.groups)) {
      if (raw === undefined) continue;
      const n = Number(raw);
      if (Number.isFinite(n)) values[name] = n;
    }
    if (Object.keys(values).length > 0) {
      samples.push({ ts: nowMs - startTs, values });
    }
  }
  return { samples, invalid: null, unmatchedLines };
}

/** Fixed-size ring buffer for samples. Drops oldest when full. */
export class SampleRing {
  private buf: Sample[] = [];
  constructor(public readonly capacity: number) {}

  push(s: Sample) {
    this.buf.push(s);
    if (this.buf.length > this.capacity) {
      this.buf.splice(0, this.buf.length - this.capacity);
    }
  }

  pushMany(ss: Sample[]) {
    for (const s of ss) this.push(s);
  }

  toArray(): Sample[] {
    return this.buf.slice();
  }

  clear() {
    this.buf = [];
  }

  get size(): number {
    return this.buf.length;
  }

  /** Return the union of all series names that have ever appeared. */
  seriesNames(): string[] {
    const set = new Set<string>();
    for (const s of this.buf) for (const k of Object.keys(s.values)) set.add(k);
    return [...set].sort();
  }

  /** Convert to columnar arrays suitable for uPlot: [xs, ys1, ys2, ...] */
  toUPlotData(names: string[]): number[][] {
    const xs: number[] = [];
    const ys: number[][] = names.map(() => []);
    for (const s of this.buf) {
      xs.push(s.ts / 1000); // seconds
      names.forEach((n, i) => {
        const v = s.values[n];
        ys[i].push(Number.isFinite(v) ? v : Number.NaN);
      });
    }
    return [xs, ...ys];
  }

  /** CSV with `ts_ms` plus one column per series name. */
  toCsv(names: string[]): string {
    const header = ["ts_ms", ...names].join(",");
    const rows = this.buf.map((s) => {
      const cols = [String(s.ts), ...names.map((n) => {
        const v = s.values[n];
        return Number.isFinite(v) ? String(v) : "";
      })];
      return cols.join(",");
    });
    return [header, ...rows].join("\n") + "\n";
  }
}
