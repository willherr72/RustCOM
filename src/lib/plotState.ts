import { writable } from "svelte/store";
import { SampleRing, type Sample } from "$lib/plotExtractor";
import type { TabId } from "$lib/ipc";

const rings = new Map<TabId, SampleRing>();

/** Bumped on every push so the panel can re-render. */
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

export function pushSamples(id: TabId, samples: Sample[]) {
  const ring = rings.get(id);
  if (!ring || samples.length === 0) return;
  ring.pushMany(samples);
  plotTick.update((n) => n + 1);
}

export function clearRing(id: TabId) {
  rings.get(id)?.clear();
  plotTick.update((n) => n + 1);
}

export function dropRing(id: TabId) {
  rings.delete(id);
}
