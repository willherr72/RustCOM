import { writable, type Writable } from "svelte/store";
import { ipc, type PortInfo } from "$lib/ipc";

export const availablePorts: Writable<PortInfo[]> = writable([]);

let pollHandle: ReturnType<typeof setInterval> | null = null;

export async function refreshPorts() {
  try {
    const list = await ipc.listPorts();
    availablePorts.set(list);
  } catch {
    // Listing can transiently fail on Windows during USB enumeration —
    // ignore; we'll get the next poll.
  }
}

export function startPolling(intervalMs = 3000) {
  if (pollHandle) return;
  void refreshPorts();
  pollHandle = setInterval(refreshPorts, intervalMs);
}

export function stopPolling() {
  if (pollHandle) {
    clearInterval(pollHandle);
    pollHandle = null;
  }
}
