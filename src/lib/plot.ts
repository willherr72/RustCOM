import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";

export type UPlotInstance = uPlot;
export type UPlotOptions = uPlot.Options;
export type UPlotData = uPlot.AlignedData;

/**
 * Build uPlot options matching our Zed Cool theme. `seriesNames` is the list
 * of series in order; their colors come from `palette` cyclically.
 */
const palette = ["#7a86c8", "#8fc4a8", "#c4a78c", "#d4b483", "#c87a86", "#b8c4d8"];

function colorFor(i: number): string {
  return palette[i % palette.length];
}

/**
 * A floating tooltip that follows the cursor and shows the x value plus each
 * visible series' value at the hovered sample. uPlot ships a docked "live
 * legend" but no cursor-following box, so this fills that in. The tooltip div
 * is created inside `u.over` on init and removed on destroy.
 */
function tooltipPlugin(seriesNames: string[]): uPlot.Plugin {
  let tt: HTMLDivElement | null = null;
  return {
    hooks: {
      init: (u) => {
        tt = document.createElement("div");
        tt.className = "u-tooltip";
        tt.style.display = "none";
        u.over.appendChild(tt);
      },
      setCursor: (u) => {
        if (!tt) return;
        const idx = u.cursor.idx;
        const left = u.cursor.left;
        const top = u.cursor.top;
        if (idx == null || left == null || top == null || left < 0 || top < 0) {
          tt.style.display = "none";
          return;
        }
        const xs = u.data[0] as readonly (number | null)[];
        const xv = xs[idx];
        let rows = `<div class="u-tt-x">${xv == null ? "" : `${(xv as number).toFixed(2)} s`}</div>`;
        let any = false;
        for (let s = 1; s < u.series.length; s++) {
          if (u.series[s].show === false) continue;
          const col = u.data[s] as readonly (number | null)[];
          const yv = col[idx];
          const name = seriesNames[s - 1] ?? `s${s}`;
          const disp = yv == null || Number.isNaN(yv) ? "—" : String(yv);
          rows +=
            `<div class="u-tt-row">` +
            `<span class="u-tt-dot" style="background:${colorFor(s - 1)}"></span>` +
            `<span class="u-tt-name">${name}</span>` +
            `<span class="u-tt-val">${disp}</span>` +
            `</div>`;
          any = true;
        }
        if (!any) {
          tt.style.display = "none";
          return;
        }
        tt.innerHTML = rows;
        tt.style.display = "block";
        // Keep the box inside the plot rect; flip sides near the edges.
        const r = tt.getBoundingClientRect();
        const w = u.over.clientWidth;
        const h = u.over.clientHeight;
        let x = left + 12;
        if (x + r.width > w) x = left - r.width - 12;
        let y = top + 12;
        if (y + r.height > h) y = top - r.height - 12;
        tt.style.left = `${Math.max(0, x)}px`;
        tt.style.top = `${Math.max(0, y)}px`;
      },
      destroy: () => {
        tt?.remove();
        tt = null;
      },
    },
  };
}

export function buildOptions(
  seriesNames: string[],
  width: number,
  height: number,
): UPlotOptions {
  return {
    width,
    height,
    pxAlign: false,
    cursor: { drag: { x: true, y: false }, points: { size: 6 } },
    legend: { show: true, live: true },
    plugins: [tooltipPlugin(seriesNames)],
    scales: {
      x: { time: false },
      y: { auto: true },
    },
    axes: [
      { stroke: "#8a8d92", grid: { stroke: "#1f2124" }, ticks: { stroke: "#1f2124" }, label: "seconds", labelSize: 18 },
      { stroke: "#8a8d92", grid: { stroke: "#1f2124" }, ticks: { stroke: "#1f2124" } },
    ],
    series: [
      { label: "t (s)" },
      ...seriesNames.map((name, i) => ({
        label: name,
        stroke: colorFor(i),
        width: 1.5,
      })),
    ],
  };
}

export { uPlot };
