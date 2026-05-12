<script lang="ts">
  import { tabs, activeTabId, addTab, closeTab, setActiveTab } from "$lib/stores/tabs";
  import { ipc } from "$lib/ipc";
  import type { TabId } from "$lib/ipc";

  function labelFor(id: TabId): string {
    const t = $tabs.find((x) => x.id === id);
    if (!t) return "—";
    if (t.state === "connected" && t.config.port_name) return t.config.port_name;
    return `Tab ${id}`;
  }

  async function onClose(e: MouseEvent, id: TabId) {
    e.stopPropagation();
    // Best-effort disconnect; ignore errors (port may not be open).
    await ipc.closePort(id).catch(() => undefined);
    closeTab(id);
  }
</script>

<div class="strip">
  {#each $tabs as t (t.id)}
    <div
      class="tab"
      class:active={t.id === $activeTabId}
      onclick={() => setActiveTab(t.id)}
      role="tab"
      tabindex="0"
      onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") setActiveTab(t.id); }}
    >
      {#if t.state === "connected"}
        <span class="dot ok"></span>
      {:else if t.state === "connecting" || t.state === "reconnecting"}
        <span class="dot warn"></span>
      {:else}
        <span class="dot muted"></span>
      {/if}
      <span class="label">
        {labelFor(t.id)}
        {#if t.state === "reconnecting" && t.reconnectAttempts > 1}
          <span class="attempt"> · retry {t.reconnectAttempts}</span>
        {/if}
      </span>
      <button class="close" onclick={(e) => onClose(e, t.id)} title="Close tab">×</button>
    </div>
  {/each}
  <button class="add" onclick={addTab} title="New tab">+</button>
</div>

<style>
  .strip {
    display: flex;
    gap: 2px;
    padding: 6px 8px 0;
    background: var(--bg);
    border-bottom: 1px solid var(--border);
    align-items: center;
    overflow-x: auto;
  }
  .tab {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px 4px 10px;
    font-size: 11px;
    border-radius: var(--radius-md) var(--radius-md) 0 0;
    color: var(--fg-muted);
    background: transparent;
    cursor: pointer;
    user-select: none;
  }
  .tab:hover { background: var(--bg-input); color: var(--fg); }
  .tab.active { background: var(--bg-elevated); color: var(--fg); }
  .label { white-space: nowrap; }
  .attempt { color: var(--warn); font-size: 9px; }
  .dot { width: 6px; height: 6px; border-radius: 50%; display: inline-block; }
  .dot.ok { background: var(--ok); }
  .dot.warn { background: var(--warn); }
  .dot.muted { background: var(--fg-subtle); }
  .close {
    background: transparent;
    color: var(--fg-subtle);
    padding: 0 4px;
    font-size: 13px;
    line-height: 1;
    border-radius: 3px;
  }
  .close:hover { background: var(--err-bg); color: var(--err); }
  .add {
    background: transparent;
    color: var(--fg-subtle);
    padding: 4px 8px;
    font-size: 12px;
    border-radius: var(--radius-md);
  }
  .add:hover { background: var(--bg-input); color: var(--fg); }
</style>
