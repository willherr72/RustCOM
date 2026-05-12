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
        stroke: palette[i % palette.length],
        width: 1.5,
      })),
    ],
  };
}

export { uPlot };
