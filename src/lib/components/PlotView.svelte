<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { activeTab, activeTabId } from "$lib/stores/tabs";
  import { plotTick, ringFor } from "$lib/plotState";

  type UPlotInstance = InstanceType<typeof import("uplot")>;
  type UPlotOptions = import("uplot").Options;

  let chartEl: HTMLDivElement | undefined = $state();
  let chart: UPlotInstance | undefined;
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

  $effect(() => {
    void $plotTick;
    void $activeTab.plotPattern;
    void $activeTab.plotCapacity;
    void $activeTabId;
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
</script>

<div class="plot-view">
  <div class="chart" bind:this={chartEl}></div>
  {#if $activeTab.plotStartedAt === 0}
    <div class="hint-overlay">
      <p>Open the <strong>Plot</strong> panel on the left, enter a regex with named capture groups (e.g. <code>{"Battery at (?<voltage>\\d+) mV"}</code>), and press <strong>Start</strong>.</p>
    </div>
  {/if}
</div>

<style>
  .plot-view {
    flex: 1;
    min-height: 0;
    position: relative;
    background: var(--bg-elevated);
  }
  .chart {
    position: absolute;
    inset: 8px;
    overflow: hidden;
  }
  .hint-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
    padding: 0 32px;
  }
  .hint-overlay p {
    max-width: 480px;
    text-align: center;
    color: var(--fg-subtle);
    font-size: 12px;
    line-height: 1.6;
  }
  .hint-overlay code {
    font-family: var(--font-mono);
    background: var(--bg-input);
    padding: 1px 4px;
    border-radius: 3px;
    color: var(--fg-muted);
  }
</style>
