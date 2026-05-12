<script lang="ts">
  import { activeTab, patchActiveTab } from "$lib/stores/tabs";
  import { findMatches } from "$lib/search";

  let { displayText = "" }: { displayText?: string } = $props();

  let result = $derived(findMatches(displayText, $activeTab.searchPattern, $activeTab.searchUseRegex));
  let total = $derived(result.matches.length);
  let current = $derived(total === 0 ? -1 : Math.max(0, Math.min($activeTab.searchMatchIndex, total - 1)));

  function close() {
    patchActiveTab({ searchOpen: false, searchMatchIndex: -1 });
  }

  function next() {
    if (total === 0) return;
    const i = current === -1 ? 0 : (current + 1) % total;
    patchActiveTab({ searchMatchIndex: i });
  }

  function prev() {
    if (total === 0) return;
    const i = current === -1 ? total - 1 : (current - 1 + total) % total;
    patchActiveTab({ searchMatchIndex: i });
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") { e.preventDefault(); close(); }
    else if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); next(); }
    else if (e.key === "Enter" && e.shiftKey) { e.preventDefault(); prev(); }
  }
</script>

{#if $activeTab.searchOpen}
  <div class="overlay">
    <!-- svelte-ignore a11y_autofocus -->
    <input
      autofocus
      placeholder="Find…"
      value={$activeTab.searchPattern}
      oninput={(e) => patchActiveTab({ searchPattern: (e.currentTarget as HTMLInputElement).value, searchMatchIndex: -1 })}
      onkeydown={onKey}
    />
    <label class="re">
      <input
        type="checkbox"
        checked={$activeTab.searchUseRegex}
        onchange={(e) => patchActiveTab({ searchUseRegex: (e.currentTarget as HTMLInputElement).checked, searchMatchIndex: -1 })}
      />
      .* regex
    </label>
    <span class="count">{total === 0 ? "0" : `${current + 1} / ${total}`}</span>
    <button onclick={prev} disabled={total === 0}>↑</button>
    <button onclick={next} disabled={total === 0}>↓</button>
    <button onclick={close}>×</button>
    {#if result.invalid}
      <span class="err">{result.invalid}</span>
    {/if}
  </div>
{/if}

<style>
  .overlay {
    position: absolute;
    top: 8px;
    right: 12px;
    background: var(--bg-titlebar);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    font-size: 11px;
    z-index: 10;
  }
  .overlay input { font-family: var(--font-mono); width: 200px; }
  .re { display: inline-flex; align-items: center; gap: 4px; color: var(--fg-muted); }
  .count { color: var(--fg-subtle); min-width: 50px; text-align: right; }
  .overlay button { padding: 2px 8px; font-size: 11px; }
  .err { color: var(--err); }
</style>
