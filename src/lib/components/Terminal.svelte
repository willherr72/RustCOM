<script lang="ts">
  import { activeTab, appendBytes, clearBuffer, patchActiveTab, SOLE_TAB_ID } from "$lib/stores/tabs";
  import { ipc, onRx, onDisconnect, onTx, type RxPayload, type TxPayload, type DisconnectPayload } from "$lib/ipc";
  import { bytesToText, formatHex } from "$lib/format";
  import { onMount } from "svelte";

  let scrollEl: HTMLDivElement | undefined = $state();
  let mounted = $state(false);

  // Subscriptions
  onMount(() => {
    let unrxs: Array<() => void> = [];

    (async () => {
      const unrx = await onRx(SOLE_TAB_ID, (event) => {
        const payload = event.payload as RxPayload;
        const bytes = new Uint8Array(payload.bytes);
        appendBytes(SOLE_TAB_ID, bytes);
      });
      const untx = await onTx(SOLE_TAB_ID, (event) => {
        const payload = event.payload as TxPayload;
        // Echo TX into the buffer with direction tag so txBytes counter is correct.
        appendBytes(SOLE_TAB_ID, new Uint8Array(payload.bytes), "tx");
      });
      const undisc = await onDisconnect(SOLE_TAB_ID, (event) => {
        const payload = event.payload as DisconnectPayload;
        patchActiveTab({ state: "disconnected", errorMessage: payload.reason, dtr: false, rts: false });
        // Defensive: ensure the backend manager entry is reaped even if the worker
        // didn't self-evict for some reason.
        ipc.closePort(SOLE_TAB_ID).catch(() => undefined);
      });
      unrxs = [unrx, untx, undisc];
      mounted = true;
    })();

    return () => unrxs.forEach((u) => u());
  });

  let display = $derived.by(() => {
    const t = $activeTab;
    if (t.viewMode === "ascii") return bytesToText(t.buffer, t.stripAnsi);
    if (t.viewMode === "hex") return formatHex(t.buffer);
    const ascii = bytesToText(t.buffer, t.stripAnsi);
    const hex = formatHex(t.buffer);
    return `=== HEX ===\n${hex}\n=== ASCII ===\n${ascii}`;
  });

  $effect(() => {
    if (mounted && scrollEl && $activeTab.autoScroll) {
      // Touch derived value so the effect re-runs on display change.
      void display;
      requestAnimationFrame(() => {
        if (scrollEl) scrollEl.scrollTop = scrollEl.scrollHeight;
      });
    }
  });
</script>

<div class="bar">
  <div class="modes">
    {#each ["ascii", "hex", "both"] as const as m}
      <button
        class:active={$activeTab.viewMode === m}
        onclick={() => patchActiveTab({ viewMode: m })}
      >
        {m === "ascii" ? "ASCII" : m === "hex" ? "HEX" : "Both"}
      </button>
    {/each}
  </div>
  <label class="check">
    <input
      type="checkbox"
      checked={$activeTab.stripAnsi}
      onchange={(e) => patchActiveTab({ stripAnsi: (e.currentTarget as HTMLInputElement).checked })}
    />
    Strip ANSI
  </label>
  <label class="check">
    <input
      type="checkbox"
      checked={$activeTab.autoScroll}
      onchange={(e) => patchActiveTab({ autoScroll: (e.currentTarget as HTMLInputElement).checked })}
    />
    Auto-scroll
  </label>
  <span class="spacer"></span>
  <button onclick={() => clearBuffer(SOLE_TAB_ID)}>Clear</button>
</div>

<div class="term" bind:this={scrollEl}>
  <pre>{display}</pre>
</div>

<style>
  .bar {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 12px;
    background: var(--bg-elevated);
    border-bottom: 1px solid var(--border);
    font-size: 11px;
  }
  .modes { display: flex; gap: 4px; }
  .modes button { padding: 4px 8px; font-size: 10px; }
  .modes button.active { background: var(--bg-input-hover); color: var(--accent); }
  .check { display: inline-flex; align-items: center; gap: 4px; color: var(--fg-muted); cursor: pointer; }
  .spacer { flex: 1; }
  .bar button { padding: 4px 10px; font-size: 10px; }
  .term {
    flex: 1;
    background: var(--bg-elevated);
    overflow: auto;
    padding: 8px 12px;
  }
  pre {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.55;
    color: var(--rx);
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
