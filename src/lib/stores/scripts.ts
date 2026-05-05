import { writable, type Writable } from "svelte/store";
import { ipc, type Script, type TabId } from "$lib/ipc";

export const scripts: Writable<Script[]> = writable([]);
export const scriptsLoaded: Writable<boolean> = writable(false);
export const runningScriptId: Writable<string | null> = writable(null);

export async function loadScripts() {
  try {
    const list = await ipc.listScripts();
    scripts.set(list);
  } catch {
    scripts.set([]);
  } finally {
    scriptsLoaded.set(true);
  }
}

export async function upsertScript(item: Script) {
  const next = await ipc.saveScript(item);
  scripts.set(next);
}

export async function removeScript(id: string) {
  const next = await ipc.deleteScript(id);
  scripts.set(next);
}

export async function runScript(tabId: TabId, item: Script) {
  await ipc.scriptRun(tabId, item.source);
  runningScriptId.set(item.id);
}

export async function stopScript() {
  await ipc.scriptStop();
  runningScriptId.set(null);
}
