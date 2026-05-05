<script lang="ts">
  let { connected = false }: { connected?: boolean } = $props();
</script>

<header>
  <span class="brand">RustCOM</span>
  {#if connected}
    <span class="badge ok">● Connected</span>
  {:else}
    <span class="badge muted">Disconnected</span>
  {/if}
</header>

<style>
  header {
    height: var(--titlebar-h);
    background: var(--bg-titlebar);
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    padding: 0 12px;
    gap: 10px;
    font-size: 12px;
  }
  .brand { font-weight: 600; color: var(--fg); }
  .badge {
    font-size: 10px;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    letter-spacing: 0.04em;
  }
  .ok { background: var(--ok-bg); color: var(--ok); }
  .muted { background: var(--bg-input); color: var(--fg-subtle); }
</style>
