import { writable, type Writable, get } from "svelte/store";
import { ipc, type Settings, type DefaultsConfig } from "$lib/ipc";

const initial: Settings = {
  defaults: {
    baud_rate: 9600,
    data_bits: "eight",
    stop_bits: "one",
    parity: "none",
    flow_control: "none",
    line_ending: "crlf",
    send_mode: "ascii",
    auto_reconnect: false,
    reconnect_delay_ms: 2000,
  },
};

export const settings: Writable<Settings> = writable(initial);
export const settingsLoaded: Writable<boolean> = writable(false);

export async function loadSettings() {
  try {
    const s = await ipc.loadSettings();
    settings.set(s);
  } catch {
    settings.set(initial);
  } finally {
    settingsLoaded.set(true);
  }
}

export async function persistDefaults(defaults: DefaultsConfig) {
  const next: Settings = { ...get(settings), defaults };
  const saved = await ipc.saveSettings(next);
  settings.set(saved);
}

export function snapshotDefaults(): DefaultsConfig {
  return get(settings).defaults;
}
