<script lang="ts">
  import { activeTab, activeTabId, navigateHistory, patchActiveTab, pushHistory } from "$lib/stores/tabs";
  import { ipc, type LineEnding, type SendMode } from "$lib/ipc";

  let value = $state("");
  let sending = $state(false);

  async function submit() {
    const tab = $activeTab;
    if (tab.state !== "connected" || !value) return;
    sending = true;
    try {
      if (tab.sendMode === "ascii") {
        await ipc.sendText($activeTabId, value, tab.lineEnding);
      } else {
        await ipc.sendHex($activeTabId, value);
      }
      // The backend emits tx://<tab_id> after the actual write succeeds —
      // Terminal.svelte's onTx handler updates txBytes and the visible buffer
      // from there. We never bump counters client-side; backend is the source of truth.
      pushHistory($activeTabId, value);
      value = "";
    } catch (e) {
      patchActiveTab({ errorMessage: String(e) });
    } finally {
      sending = false;
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void submit();
      return;
    }
    if (e.key === "ArrowUp") {
      const r = navigateHistory($activeTabId, "up");
      if (r !== null) {
        e.preventDefault();
        value = r;
      }
    } else if (e.key === "ArrowDown") {
      const r = navigateHistory($activeTabId, "down");
      if (r !== null) {
        e.preventDefault();
        value = r;
      }
    }
  }

  function setMode(m: SendMode) { patchActiveTab({ sendMode: m }); }
  function setLineEnding(le: LineEnding) { patchActiveTab({ lineEnding: le }); }
</script>

<div class="row">
  <div class="modes">
    <button class:active={$activeTab.sendMode === "ascii"} onclick={() => setMode("ascii")}>ASCII</button>
    <button class:active={$activeTab.sendMode === "hex"} onclick={() => setMode("hex")}>Hex</button>
  </div>

  {#if $activeTab.sendMode === "ascii"}
    <select
      value={$activeTab.lineEnding}
      onchange={(e) => setLineEnding((e.currentTarget as HTMLSelectElement).value as LineEnding)}
    >
      <option value="none">None</option>
      <option value="cr">\r</option>
      <option value="lf">\n</option>
      <option value="crlf">\r\n</option>
    </select>
  {/if}

  <input
    bind:value
    onkeydown={onKey}
    placeholder={$activeTab.sendMode === "ascii" ? "Type message…" : "AA BB 0D 0A …"}
    disabled={$activeTab.state !== "connected" || sending}
  />

  <button class="primary" onclick={submit} disabled={$activeTab.state !== "connected" || !value || sending}>
    Send
  </button>
</div>

<style>
  .row {
    display: flex;
    gap: 6px;
    padding: 8px 10px;
    background: var(--bg-elevated);
    border-top: 1px solid var(--border);
    align-items: center;
  }
  .modes { display: flex; gap: 2px; }
  .modes button { padding: 5px 10px; font-size: 10px; }
  .modes button.active { background: var(--bg-input-hover); color: var(--accent); }
  input { flex: 1; font-family: var(--font-mono); }
  select { width: 70px; }
  button.primary { padding: 6px 14px; }
</style>
