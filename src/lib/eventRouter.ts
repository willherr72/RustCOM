import { get } from "svelte/store";
import { tabs, appendBytes, type Tab } from "$lib/stores/tabs";
import {
  ipc,
  onRx,
  onTx,
  onDisconnect,
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
    appendBytes(id, new Uint8Array(p.bytes), "rx");
  });
  const untx = await onTx(id, (event) => {
    const p = event.payload as TxPayload;
    appendBytes(id, new Uint8Array(p.bytes), "tx");
  });
  const undisc = await onDisconnect(id, (event) => {
    const p = event.payload as DisconnectPayload;
    // Patch any tab matching the id, not just the active one.
    tabs.update((list) =>
      list.map((t) =>
        t.id === id
          ? { ...t, state: "disconnected", errorMessage: p.reason, dtr: false, rts: false }
          : t
      )
    );
    ipc.closePort(id).catch(() => undefined);
  });

  subs.set(id, [unrx, untx, undisc]);
}

function unsubscribe(id: TabId) {
  const list = subs.get(id);
  if (!list) return;
  list.forEach((u) => u());
  subs.delete(id);
}

let stopReact: Unsub | null = null;

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
}

export function stopEventRouter() {
  if (stopReact) {
    stopReact();
    stopReact = null;
  }
  for (const id of [...subs.keys()]) unsubscribe(id);
}
