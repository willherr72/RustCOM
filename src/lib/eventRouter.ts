import { get } from "svelte/store";
import { tabs, activeTabId, appendBytes, pushLog, type Tab } from "$lib/stores/tabs";
import {
  ipc,
  onRx,
  onTx,
  onDisconnect,
  onReconnect,
  onScriptLog,
  type RxPayload,
  type TxPayload,
  type DisconnectPayload,
  type TabId,
} from "$lib/ipc";

type Unsub = () => void;

const subs = new Map<TabId, Unsub[]>();

async function subscribe(id: TabId) {
  if (subs.has(id)) return;

  const unrx = await onRx(id, (event) => {
    const p = event.payload as RxPayload;
    const bytes = new Uint8Array(p.bytes);
    appendBytes(id, bytes, "rx");
    pushLog(id, "rx", bytes, p.ts);
  });
  const untx = await onTx(id, (event) => {
    const p = event.payload as TxPayload;
    const bytes = new Uint8Array(p.bytes);
    appendBytes(id, bytes, "tx");
    pushLog(id, "tx", bytes, p.ts);
  });
  const undisc = await onDisconnect(id, (event) => {
    const p = event.payload as DisconnectPayload;
    let willReconnect = false;
    tabs.update((list) =>
      list.map((t) => {
        if (t.id !== id) return t;
        willReconnect = !!t.config.auto_reconnect;
        const next: Tab = {
          ...t,
          state: willReconnect ? "reconnecting" : "disconnected",
          errorMessage: p.reason,
          dtr: false,
          rts: false,
        };
        return next;
      })
    );
    if (!willReconnect) {
      ipc.closePort(id).catch(() => undefined);
    }
  });
  const unre = await onReconnect(id, (_event) => {
    tabs.update((list) =>
      list.map((t) => (t.id === id ? { ...t, state: "connected", errorMessage: null } : t))
    );
  });

  subs.set(id, [unrx, untx, undisc, unre]);
}

function unsubscribe(id: TabId) {
  const list = subs.get(id);
  if (!list) return;
  list.forEach((u) => u());
  subs.delete(id);
}

let stopReact: Unsub | null = null;
let stopLog: Unsub | null = null;

export function startEventRouter() {
  if (stopReact) return;

  // Subscribe for whatever tabs are present right now.
  for (const t of get(tabs)) void subscribe(t.id);

  stopReact = tabs.subscribe((list: Tab[]) => {
    const present = new Set(list.map((t) => t.id));
    // Add subs for new tabs.
    for (const t of list) if (!subs.has(t.id)) void subscribe(t.id);
    // Drop subs for closed tabs.
    for (const id of [...subs.keys()]) if (!present.has(id)) unsubscribe(id);
  });

  void (async () => {
    stopLog = await onScriptLog((e) => {
      const id = get(activeTabId);
      pushLog(id, "rx", new TextEncoder().encode(`[lua] ${e.payload.msg}`), e.payload.ts);
    });
  })();
}

export function stopEventRouter() {
  if (stopReact) {
    stopReact();
    stopReact = null;
  }
  if (stopLog) {
    stopLog();
    stopLog = null;
  }
  for (const id of [...subs.keys()]) unsubscribe(id);
}
