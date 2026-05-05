<script lang="ts">
  import { toasts, dismissToast } from "$lib/stores/toasts";
</script>

<div class="stack">
  {#each $toasts as t (t.id)}
    <button type="button" class="toast {t.level}" onclick={() => dismissToast(t.id)}>
      <span>{t.msg}</span>
    </button>
  {/each}
</div>

<style>
  .stack { position: fixed; bottom: calc(var(--statusbar-h) + 8px); right: 12px; display: flex; flex-direction: column; gap: 6px; z-index: 100; pointer-events: none; }
  .toast { pointer-events: auto; padding: 8px 12px; border-radius: var(--radius-md); font-size: 11px; cursor: pointer; box-shadow: 0 4px 12px rgba(0,0,0,0.4); max-width: 320px; border: none; text-align: left; font-family: inherit; }
  .toast.info { background: var(--bg-input-hover); color: var(--fg); }
  .toast.warn { background: var(--warn-bg); color: var(--warn); }
  .toast.err, .toast.error { background: var(--err-bg); color: var(--err); }
</style>
