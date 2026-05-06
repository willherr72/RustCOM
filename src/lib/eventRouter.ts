import { get } from "svelte/store";
import { tabs, activeTabId, appendBytes, pushLog, getTab, type Tab } from "$lib/stores/tabs";
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
import { pushToast } from "$lib/stores/toasts";

type Unsub = () => void;
type Direction = "rx" | "tx";

const subs = new Map<TabId, Unsub[]>();

// rAF-batched buffer accumulator. Per-event store updates were thrashing the
// webview on chatty devices because every appendBytes triggered an O(n)
// re-decode of the whole buffer. Now we queue chunks and flush at most once
// per animation frame (~60 Hz cap on the heavy work).
type PendingBucket = { rxChunks: Uint8Array[]; rxTotal: number; txChunks: Uint8Array[]; txTotal: number };
const pending = new Map<TabId, PendingBucket>();
let rafScheduled = false;

function queueChunk(id: TabId, direction: Direction, bytes: Uint8Array) {
  let bucket = pending.get(id);
  if (!bucket) {
    bucket = { rxChunks: [], rxTotal: 0, txChunks: [], txTotal: 0 };
    pending.set(id, bucket);
  }
  if (direction === "rx") {
    bucket.rxChunks.push(bytes);
    bucket.rxTotal += bytes.length;
  } else {
    bucket.txChunks.push(bytes);
    bucket.txTotal += bytes.length;
  }
  if (!rafScheduled) {
    rafScheduled = true;
    requestAnimationFrame(flushPending);
  }
}

function concatChunks(chunks: Uint8Array[], total: number): Uint8Array {
  const out = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    out.set(chunk, offset);
    offset += chunk.length;
  }
  return out;
}

function flushPending() {
  rafScheduled = false;
  if (pending.size === 0) return;
  for (const [id, bucket] of pending) {
    if (bucket.rxTotal > 0) appendBytes(id, concatChunks(bucket.rxChunks, bucket.rxTotal), "rx");
    if (bucket.txTotal > 0) appendBytes(id, concatChunks(bucket.txChunks, bucket.txTotal), "tx");
  }
  pending.clear();
}

async function subscribe(id: TabId) {
  if (subs.has(id)) return;

  const unrx = await onRx(id, (event) => {
    const p = event.payload as RxPayload;
    const bytes = new Uint8Array(p.bytes);
    queueChunk(id, "rx", bytes);
    pushLog(id, "rx", bytes, p.ts);
  });
  const untx = await onTx(id, (event) => {
    const p = event.payload as TxPayload;
    const bytes = new Uint8Array(p.bytes);
    queueChunk(id, "tx", bytes);
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
  pending.delete(id);
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
    let lastWarnAt = 0;
    stopLog = await onScriptLog((e) => {
      const id = get(activeTabId);
      const tab = getTab(id);
      if (tab && !tab.loggingEnabled) {
        const now = Date.now();
        if (now - lastWarnAt > 30_000) {
          lastWarnAt = now;
          pushToast(
            "warn",
            "Lua log() output is dropping — enable Capture in the Logs panel to record it.",
          );
        }
        return;
      }
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
