import { writable } from "svelte/store";
import { SampleRing, type Sample } from "$lib/plotExtractor";
import type { TabId } from "$lib/ipc";

const rings = new Map<TabId, SampleRing>();
// Trailing text from the last RX chunk that didn't end in a newline. Serial
// reads fragment lines mid-message, but the extractor needs whole lines, so we
// hold the partial here until the rest of the line arrives.
const residuals = new Map<TabId, string>();
const MAX_RESIDUAL = 65536;

/** Bumped on every push so the chart can re-render. */
export const plotTick = writable(0);

export function ringFor(id: TabId, capacity: number): SampleRing {
  let ring = rings.get(id);
  if (!ring || ring.capacity !== capacity) {
    const fresh = new SampleRing(capacity);
    if (ring) for (const s of ring.toArray()) fresh.push(s);
    ring = fresh;
    rings.set(id, ring);
  }
  return ring;
}

/**
 * Append `chunk` to the tab's residual buffer and return every *complete* line
 * (everything up to and including the last newline), leaving the trailing
 * partial buffered for the next chunk. Returns "" when no full line is ready.
 */
export function feedPlotLines(id: TabId, chunk: string): string {
  const combined = (residuals.get(id) ?? "") + chunk;
  const lastNl = combined.lastIndexOf("\n");
  if (lastNl === -1) {
    // No full line yet. Cap growth in case the stream never emits newlines.
    residuals.set(id, combined.length > MAX_RESIDUAL ? combined.slice(-4096) : combined);
    return "";
  }
  residuals.set(id, combined.slice(lastNl + 1));
  return combined.slice(0, lastNl);
}

export function pushSamples(id: TabId, capacity: number, samples: Sample[]) {
  if (samples.length === 0) return;
  const ring = ringFor(id, capacity);
  ring.pushMany(samples);
  plotTick.update((n) => n + 1);
}

export function clearRing(id: TabId) {
  rings.get(id)?.clear();
  residuals.delete(id);
  plotTick.update((n) => n + 1);
}

export function dropRing(id: TabId) {
  rings.delete(id);
  residuals.delete(id);
}
