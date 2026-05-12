<script lang="ts" module>
  export function formatBytes(n: number): string {
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
    return `${(n / 1024 / 1024).toFixed(2)} MB`;
  }
</script>

<script lang="ts">
  let {
    connected = false,
    portName = "",
    rxBytes = 0,
    txBytes = 0,
    settingsLabel = "",
  }: {
    connected?: boolean;
    portName?: string;
    rxBytes?: number;
    txBytes?: number;
    settingsLabel?: string;
  } = $props();
</script>

<footer>
  {#if connected}
    <span class="ok">● {portName} connected</span>
  {:else}
    <span class="muted">○ disconnected</span>
  {/if}
  <span>RX {formatBytes(rxBytes)}</span>
  <span>TX {formatBytes(txBytes)}</span>
  <span class="spacer"></span>
  <span class="muted">{settingsLabel}</span>
</footer>

<style>
  footer {
    height: var(--statusbar-h);
    background: var(--bg-titlebar);
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    padding: 0 12px;
    gap: 14px;
    font-size: 11px;
    color: var(--fg-subtle);
  }
  .ok { color: var(--ok); }
  .muted { color: var(--fg-subtle); }
  .spacer { flex: 1; }
</style>
