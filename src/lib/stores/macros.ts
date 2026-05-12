import { writable, type Writable } from "svelte/store";
import { ipc, type Macro } from "$lib/ipc";

export const macros: Writable<Macro[]> = writable([]);
export const macrosLoaded: Writable<boolean> = writable(false);

export async function loadMacros() {
  try {
    const list = await ipc.listMacros();
    macros.set(list);
  } catch {
    macros.set([]);
  } finally {
    macrosLoaded.set(true);
  }
}

export async function upsertMacro(item: Macro) {
  const next = await ipc.saveMacro(item);
  macros.set(next);
}

export async function removeMacro(id: string) {
  const next = await ipc.deleteMacro(id);
  macros.set(next);
}

export function newMacroId(): string {
  return `m_${Date.now().toString(36)}_${Math.floor(Math.random() * 1e6).toString(36)}`;
}
