<script lang="ts">
  import { activeTab, activeTabId, clearBuffer, patchActiveTab } from "$lib/stores/tabs";
  import { bytesToText, formatHex } from "$lib/format";
  import { applyFilter } from "$lib/filter";
  import { findMatches, type MatchRange } from "$lib/search";
  import SearchOverlay from "$lib/components/SearchOverlay.svelte";

  let scrollEl: HTMLDivElement | undefined = $state();

  let asciiText = $derived.by(() => {
    const t = $activeTab;
    return applyFilter(bytesToText(t.buffer, t.stripAnsi), t.filterPattern, t.filterEnabled).text;
  });
  let hexText = $derived.by(() => formatHex($activeTab.buffer));

  // For search overlay + single-pane rendering. In "both" view we search the
  // ASCII column only (HEX dumps aren't a useful search target).
  let display = $derived.by(() => {
    const t = $activeTab;
    if (t.viewMode === "ascii") return asciiText;
    if (t.viewMode === "hex") return hexText;
    return asciiText;
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
  <div class="term" class:both={$activeTab.viewMode === "both"} bind:this={scrollEl}>
    {#if $activeTab.viewMode === "both"}
      <div class="col hex-col">
        <div class="col-label">HEX</div>
        <pre class="hex-pane">{hexText}</pre>
      </div>
      <div class="col ascii-col">
        <div class="col-label">ASCII</div>
        <pre>{@html renderHighlighted(asciiText, matches, $activeTab.searchMatchIndex)}</pre>
      </div>
    {:else}
      <pre>{@html renderHighlighted(display, matches, $activeTab.searchMatchIndex)}</pre>
    {/if}
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
  .term.both {
    display: grid;
    grid-template-columns: minmax(440px, max-content) minmax(0, 1fr);
    gap: 12px;
    padding: 0;
  }
  .col { min-width: 0; display: flex; flex-direction: column; }
  .col.hex-col {
    border-right: 1px solid var(--border);
    background: var(--bg);
  }
  .col-label {
    position: sticky;
    top: 0;
    z-index: 1;
    background: var(--bg-titlebar);
    color: var(--fg-subtle);
    font-size: 9px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    padding: 4px 12px;
    border-bottom: 1px solid var(--border);
  }
  .col pre { padding: 6px 12px; }
  .hex-pane { white-space: pre; word-break: normal; }
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
