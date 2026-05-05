import { writable, type Writable } from "svelte/store";

export interface Toast {
  id: number;
  level: string;
  msg: string;
  ts: number;
}

let next = 1;

export const toasts: Writable<Toast[]> = writable([]);

export function pushToast(level: string, msg: string, ts: number = Date.now()) {
  const id = next++;
  toasts.update((list) => [...list, { id, level, msg, ts }]);
  setTimeout(() => dismissToast(id), 5000);
}

export function dismissToast(id: number) {
  toasts.update((list) => list.filter((t) => t.id !== id));
}
