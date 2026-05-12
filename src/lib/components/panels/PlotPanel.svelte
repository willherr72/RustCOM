<script lang="ts">
  import { activeTab, activeTabId, setPlotPattern, startPlot, pausePlot, setPlotCapacity } from "$lib/stores/tabs";
  import { ringFor, clearRing } from "$lib/plotState";
  import { extract } from "$lib/plotExtractor";
  import { save } from "@tauri-apps/plugin-dialog";
  import { writeTextFile } from "@tauri-apps/plugin-fs";

  let panelError = $state<string | null>(null);

  // Validate the regex live (passing empty text just exercises the RegExp compile).
  let validation = $derived(extract("", $activeTab.plotPattern, 0, 0));

  function start() {
    panelError = validation.invalid;
    if (panelError) return;
    if (!$activeTab.plotPattern) {
      panelError = "Set a regex pattern with named groups first.";
      return;
    }
    clearRing($activeTabId);
    startPlot($activeTabId);
  }

  function clearAll() {
    panelError = null;
    clearRing($activeTabId);
  }

  async function exportCsv() {
    const ring = ringFor($activeTabId, $activeTab.plotCapacity);
    if (ring.size === 0) {
      panelError = "Nothing to export yet.";
      return;
    }
    const path = await save({
      title: "Save plot data as CSV",
      defaultPath: `rustcom_plot_${new Date().toISOString().replace(/[:.]/g, "-")}.csv`,
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    if (!path) return;
    try {
      const csv = ring.toCsv(ring.seriesNames());
      await writeTextFile(path, csv);
    } catch (e) {
      panelError = `Save failed: ${String(e)}`;
    }
  }
</script>

<div class="panel">
  <h4>Plot</h4>
  <p class="hint">Match a regex with named capture groups against incoming lines — each group's number becomes a series in the chart on the right.</p>

  <label class="lbl">
    Pattern
    <input
      placeholder={"Battery at (?<voltage>\\d+) mV"}
      value={$activeTab.plotPattern}
      oninput={(e) => setPlotPattern($activeTabId, (e.currentTarget as HTMLInputElement).value)}
    />
  </label>

  <label class="lbl">
    Window (points)
    <input
      type="number"
      min="50"
      step="50"
      value={$activeTab.plotCapacity}
      onchange={(e) => setPlotCapacity($activeTabId, Number((e.currentTarget as HTMLInputElement).value))}
    />
  </label>

  {#if validation.invalid}
    <p class="err">{validation.invalid}</p>
  {/if}
  {#if panelError && !validation.invalid}
    <p class="err">{panelError}</p>
  {/if}

  <div class="actions">
    {#if $activeTab.plotStartedAt === 0}
      <button class="primary" onclick={start}>Start</button>
    {:else if $activeTab.plotPaused}
      <button class="primary" onclick={() => pausePlot($activeTabId, false)}>Resume</button>
    {:else}
      <button onclick={() => pausePlot($activeTabId, true)}>Pause</button>
    {/if}
    <button onclick={clearAll}>Clear</button>
    <button onclick={exportCsv}>Save CSV…</button>
  </div>

  <p class="status">
    {#if $activeTab.plotStartedAt === 0}
      Not running — press Start.
    {:else if $activeTab.plotPaused}
      Paused.
    {:else}
      Running.
    {/if}
  </p>
</div>

<style>
  .panel { padding: 12px; height: 100%; box-sizing: border-box; display: flex; flex-direction: column; gap: 8px; min-height: 0; }
  h4 { font-size: 10px; letter-spacing: 0.1em; text-transform: uppercase; color: var(--fg-subtle); margin: 0; font-weight: 600; }
  .hint { color: var(--fg-subtle); font-size: 10px; margin: 0; }
  .lbl { font-size: 9px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--fg-subtle); display: flex; flex-direction: column; gap: 4px; }
  .lbl input { width: 100%; font-family: var(--font-mono); }
  .err { color: var(--err); font-size: 11px; margin: 0; }
  .actions { display: flex; gap: 4px; }
  .actions button { padding: 4px 10px; font-size: 10px; flex: 1 1 auto; }
  .status { color: var(--fg-subtle); font-size: 11px; margin: 4px 0 0; }
</style>
