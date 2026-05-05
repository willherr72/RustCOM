import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn, type EventCallback } from "@tauri-apps/api/event";

export type TabId = number;

export type DataBits = "five" | "six" | "seven" | "eight";
export type StopBits = "one" | "two";
export type Parity = "none" | "even" | "odd";
export type FlowControl = "none" | "software" | "hardware";
export type LineEnding = "none" | "cr" | "lf" | "crlf";
export type SendMode = "ascii" | "hex";

export interface PortConfig {
  port_name: string;
  baud_rate: number;
  data_bits: DataBits;
  stop_bits: StopBits;
  parity: Parity;
  flow_control: FlowControl;
  auto_reconnect?: boolean;
  reconnect_delay_ms?: number;
}

export interface PortInfo {
  name: string;
  kind: string;
}

export interface RxPayload {
  bytes: number[];
  ts: number;
}

export interface TxPayload {
  bytes: number[];
  ts: number;
}

export interface DisconnectPayload {
  reason: string;
  ts: number;
}

export interface ReconnectPayload {
  ts: number;
}

export type Direction = "rx" | "tx";

export interface LogEntryDto {
  ts_ms: number;
  direction: Direction;
  bytes: number[];
}

export type MacroMode = "ascii" | "hex";

export interface Macro {
  id: string;
  name: string;
  mode: MacroMode;
  payload: string;
  line_ending?: LineEnding | null;
  hotkey?: string | null;
}

export interface DefaultsConfig {
  baud_rate: number;
  data_bits: DataBits;
  stop_bits: StopBits;
  parity: Parity;
  flow_control: FlowControl;
  line_ending: LineEnding;
  send_mode: SendMode;
  auto_reconnect: boolean;
  reconnect_delay_ms: number;
}

export interface Settings {
  defaults: DefaultsConfig;
}

export const ipc = {
  listPorts: () => invoke<PortInfo[]>("list_ports"),
  openPort: (tabId: TabId, config: PortConfig) =>
    invoke<void>("open_port", { tabId, config }),
  closePort: (tabId: TabId) => invoke<void>("close_port", { tabId }),
  sendText: (tabId: TabId, text: string, lineEnding: LineEnding) =>
    invoke<void>("send_text", { tabId, text, lineEnding }),
  sendHex: (tabId: TabId, input: string) =>
    invoke<void>("send_hex", { tabId, input }),
  setDtr: (tabId: TabId, state: boolean) =>
    invoke<void>("set_dtr", { tabId, state }),
  setRts: (tabId: TabId, state: boolean) =>
    invoke<void>("set_rts", { tabId, state }),
  listMacros: () => invoke<Macro[]>("list_macros"),
  saveMacro: (item: Macro) => invoke<Macro[]>("save_macro", { item }),
  deleteMacro: (id: string) => invoke<Macro[]>("delete_macro", { id }),
  saveLog: (path: string, entries: LogEntryDto[]) =>
    invoke<string>("save_log", { path, entries }),
  loadSettings: () => invoke<Settings>("load_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
};

export function onRx(tabId: TabId, cb: EventCallback<RxPayload>): Promise<UnlistenFn> {
  return listen<RxPayload>(`rx://${tabId}`, cb);
}

export function onTx(tabId: TabId, cb: EventCallback<TxPayload>): Promise<UnlistenFn> {
  return listen<TxPayload>(`tx://${tabId}`, cb);
}

export function onDisconnect(
  tabId: TabId,
  cb: EventCallback<DisconnectPayload>
): Promise<UnlistenFn> {
  return listen<DisconnectPayload>(`disconnect://${tabId}`, cb);
}

export function onReconnect(
  tabId: TabId,
  cb: EventCallback<ReconnectPayload>
): Promise<UnlistenFn> {
  return listen<ReconnectPayload>(`reconnect://${tabId}`, cb);
}
