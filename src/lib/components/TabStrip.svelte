<script lang="ts">
  let {
    label = "COM —",
    active = true,
  }: { label?: string; active?: boolean } = $props();
</script>

<div class="strip">
  <div class="tab" class:active>{label}</div>
  <div class="tab add" title="Multi-tab arrives in Plan 2">+</div>
</div>

<style>
  .strip {
    display: flex;
    gap: 2px;
    padding: 6px 8px 0;
    background: var(--bg);
    border-bottom: 1px solid var(--border);
  }
  .tab {
    padding: 5px 12px;
    font-size: 11px;
    border-radius: var(--radius-md) var(--radius-md) 0 0;
    color: var(--fg-muted);
    background: transparent;
    cursor: default;
  }
  .tab.active { background: var(--bg-elevated); color: var(--fg); }
  .tab.add { color: var(--fg-subtle); cursor: not-allowed; opacity: 0.6; }
</style>
