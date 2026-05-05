<script lang="ts">
  import { activeTab, activeTabId, clearBuffer, patchActiveTab } from "$lib/stores/tabs";
  import { bytesToText, formatHex } from "$lib/format";
  import { applyFilter } from "$lib/filter";
  import { findMatches, type MatchRange } from "$lib/search";
  import SearchOverlay from "$lib/components/SearchOverlay.svelte";

  let scrollEl: HTMLDivElement | undefined = $state();

  let display = $derived.by(() => {
    const t = $activeTab;
    const filter = (text: string) => applyFilter(text, t.filterPattern, t.filterEnabled).text;
    if (t.viewMode === "ascii") return filter(bytesToText(t.buffer, t.stripAnsi));
    if (t.viewMode === "hex") return formatHex(t.buffer);
    const ascii = filter(bytesToText(t.buffer, t.stripAnsi));
    const hex = formatHex(t.buffer);
    return `=== HEX ===\n${hex}\n=== ASCII ===\n${ascii}`;
  });

  $effect(() => {
    if (scrollEl && $activeTab.autoScroll) {
      // Touch derived value so the effect re-runs on display change.
      void display;
      requestAnimationFrame(() => {
        if (scrollEl) scrollEl.scrollTop = scrollEl.scrollHeight;
      });
    }
  });

  let matches = $derived(
    $activeTab.searchOpen
      ? findMatches(display, $activeTab.searchPattern, $activeTab.searchUseRegex).matches
      : []
  );

  function renderHighlighted(text: string, ranges: MatchRange[], current: number): string {
    if (ranges.length === 0) return escapeHtml(text);
    let out = "";
    let pos = 0;
    ranges.forEach((r, i) => {
      if (r.start > pos) out += escapeHtml(text.slice(pos, r.start));
      const cls = i === current ? "match current" : "match";
      out += `<mark class="${cls}">${escapeHtml(text.slice(r.start, r.end))}</mark>`;
      pos = r.end;
    });
    if (pos < text.length) out += escapeHtml(text.slice(pos));
    return out;
  }

  function escapeHtml(s: string): string {
    return s
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }
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
  <button onclick={() => clearBuffer($activeTabId)}>Clear</button>
</div>

<div class="term-wrap">
  <SearchOverlay displayText={display} />
  <div class="term" bind:this={scrollEl}>
    <pre>{@html renderHighlighted(display, matches, $activeTab.searchMatchIndex)}</pre>
  </div>
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
  .term-wrap { position: relative; flex: 1; display: flex; flex-direction: column; min-height: 0; }
  .term-wrap .term { flex: 1; }
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
  pre :global(mark.match) {
    background: var(--warn-bg);
    color: var(--warn);
    padding: 0 1px;
    border-radius: 2px;
  }
  pre :global(mark.match.current) {
    background: var(--accent);
    color: var(--accent-fg);
  }
</style>
