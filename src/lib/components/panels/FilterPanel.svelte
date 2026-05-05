<script lang="ts">
  import { activeTab, patchActiveTab } from "$lib/stores/tabs";
  import { applyFilter } from "$lib/filter";

  let liveError = $derived(applyFilter("", $activeTab.filterPattern, $activeTab.filterEnabled).invalid);
</script>

<div class="panel">
  <h4>Filter</h4>

  <label class="toggle">
    <input
      type="checkbox"
      checked={$activeTab.filterEnabled}
      onchange={(e) => patchActiveTab({ filterEnabled: (e.currentTarget as HTMLInputElement).checked })}
    />
    Enable regex filter
  </label>

  <label class="lbl">
    Pattern
    <input
      placeholder="e.g. ^READY|^ERROR"
      value={$activeTab.filterPattern}
      oninput={(e) => patchActiveTab({ filterPattern: (e.currentTarget as HTMLInputElement).value })}
      disabled={!$activeTab.filterEnabled}
    />
  </label>

  {#if liveError}
    <p class="err">{liveError}</p>
  {/if}

  <p class="hint">Filter is applied at display time. Lines not matching are hidden; underlying buffer is preserved.</p>
</div>

<style>
  .panel { padding: 12px; height: 100%; box-sizing: border-box; display: flex; flex-direction: column; gap: 6px; }
  h4 { font-size: 10px; letter-spacing: 0.1em; text-transform: uppercase; color: var(--fg-subtle); margin: 0 0 6px; font-weight: 600; }
  .toggle { display: flex; align-items: center; gap: 6px; font-size: 11px; color: var(--fg-muted); }
  .lbl { font-size: 9px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--fg-subtle); display: flex; flex-direction: column; gap: 4px; }
  input { width: 100%; font-family: var(--font-mono); }
  .err { color: var(--err); font-size: 11px; margin: 4px 0 0; }
  .hint { color: var(--fg-subtle); font-size: 10px; margin-top: 8px; }
</style>
