<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { activeTab, activeTabId, setPlotPattern, startPlot, pausePlot, setPlotCapacity } from "$lib/stores/tabs";
  import { plotTick, ringFor, clearRing } from "$lib/plotState";
  import { extract } from "$lib/plotExtractor";
  import { save } from "@tauri-apps/plugin-dialog";
  import { writeTextFile } from "@tauri-apps/plugin-fs";

  type UPlotInstance = InstanceType<typeof import("uplot")>;
  type UPlotOptions = import("uplot").Options;

  let chartEl: HTMLDivElement | undefined = $state();
  let chart: UPlotInstance | undefined;
  let panelError = $state<string | null>(null);
  let resizeObserver: ResizeObserver | undefined;
  let lastWidth = 0;
  let lastHeight = 0;
  let lastSeries: string[] = [];

  onMount(async () => {
    if (!chartEl) return;
    const { uPlot, buildOptions } = await import("$lib/plot");
    const rect = chartEl.getBoundingClientRect();
    lastWidth = Math.max(200, rect.width);
    lastHeight = Math.max(160, rect.height);
    chart = new uPlot(buildOptions([], lastWidth, lastHeight) as UPlotOptions, [[], []] as never, chartEl);
    resizeObserver = new ResizeObserver(() => {
      if (!chartEl || !chart) return;
      const r = chartEl.getBoundingClientRect();
      const w = Math.max(200, r.width);
      const h = Math.max(160, r.height);
      if (w !== lastWidth || h !== lastHeight) {
        lastWidth = w;
        lastHeight = h;
        chart.setSize({ width: w, height: h });
      }
    });
    resizeObserver.observe(chartEl);
    void redraw();
  });

  onDestroy(() => {
    chart?.destroy();
    resizeObserver?.disconnect();
  });

  // Validate regex live so the user gets feedback as they type.
  let validation = $derived(extract("", $activeTab.plotPattern, 0, 0));

  $effect(() => {
    void $plotTick;
    void $activeTab.plotPattern;
    void $activeTab.plotCapacity;
    void redraw();
  });

  async function redraw() {
    if (!chart || !chartEl) return;
    const ring = ringFor($activeTabId, $activeTab.plotCapacity);
    const names = ring.seriesNames();
    const data = ring.toUPlotData(names);

    const sameSeries = names.length === lastSeries.length && names.every((n, i) => n === lastSeries[i]);
    if (!sameSeries) {
      const { uPlot, buildOptions } = await import("$lib/plot");
      chart.destroy();
      chart = new uPlot(buildOptions(names, lastWidth, lastHeight) as UPlotOptions, data as never, chartEl);
      lastSeries = names;
      return;
    }

    chart.setData(data as never);
  }

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

  <label class="lbl">
    Pattern
    <input
      placeholder={"e.g. temp:(?<temp>\\d+\\.\\d+) hum:(?<hum>\\d+\\.\\d+)"}
      value={$activeTab.plotPattern}
      oninput={(e) => setPlotPattern($activeTabId, (e.currentTarget as HTMLInputElement).value)}
    />
  </label>

  <label class="lbl">
    Window
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

  <div class="chart" bind:this={chartEl}></div>
</div>

<style>
  .panel { padding: 12px; height: 100%; box-sizing: border-box; display: flex; flex-direction: column; gap: 8px; min-height: 0; }
  h4 { font-size: 10px; letter-spacing: 0.1em; text-transform: uppercase; color: var(--fg-subtle); margin: 0; font-weight: 600; }
  .lbl { font-size: 9px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--fg-subtle); display: flex; flex-direction: column; gap: 4px; }
  .lbl input { width: 100%; font-family: var(--font-mono); }
  .err { color: var(--err); font-size: 11px; margin: 0; }
  .actions { display: flex; gap: 4px; }
  .actions button { padding: 4px 10px; font-size: 10px; flex: 1 1 auto; }
  .chart { flex: 1; min-height: 200px; background: var(--bg-elevated); border-radius: var(--radius-md); overflow: hidden; }
</style>
