// Dive profile: depth over time, surface at the top.

import { depthUnit, depthValue, formatDepth, formatDuration } from "../format";
import { t } from "../i18n";
import type { Sample, Units } from "../types";

const W = 470;
const H = 150;
const LEFT = 30;
const TOP = 10;
const BOTTOM = 20;
const PLOT_W = W - LEFT - 4;
const PLOT_H = H - TOP - BOTTOM;

/** Rounds the max depth up to a readable scale (multiples of 5 m / 10 ft). */
function niceMax(v: number, units: Units): number {
  const step = units.depth === "ft" ? 10 : 5;
  return Math.max(step, Math.ceil(v / step) * step);
}

export function ProfileChart({ samples, units }: { samples: Sample[]; units: Units }) {
  const maxT = samples[samples.length - 1].time_s || 1;
  const maxD = Math.max(...samples.map((s) => s.depth_m));
  const scale = niceMax(depthValue(maxD, units), units);
  const x = (s: Sample) => ((s.time_s / maxT) * PLOT_W).toFixed(1);
  const y = (s: Sample) => ((depthValue(s.depth_m, units) / scale) * PLOT_H).toFixed(1);
  const points = samples.map((s) => `${x(s)},${y(s)}`).join(" ");
  const area = `M0,0 ${points} ${PLOT_W},0Z`;
  const grid = [0, scale / 2, scale];
  const label = t("detail.chart", { depth: formatDepth(maxD, units), duration: formatDuration(maxT) });

  return (
    <svg viewBox={`0 0 ${W} ${H}`} className="profile-chart" role="img" aria-label={label}>
      <g transform={`translate(${LEFT},${TOP})`}>
        <path
          d={grid.map((g) => `M0 ${((g / scale) * PLOT_H).toFixed(1)}H${PLOT_W}`).join("")}
          className="profile-grid"
        />
        <path d={area} className="profile-area" />
        <polyline points={points} className="profile-line" />
      </g>
      {grid.map((g, i) => (
        <text key={g} x={0} y={TOP + (g / scale) * PLOT_H + 4} className="profile-text">
          {i === grid.length - 1 ? `${g} ${depthUnit(units)}` : g}
        </text>
      ))}
      <text x={LEFT} y={H - 4} className="profile-text">
        0
      </text>
      <text x={W - 4} y={H - 4} textAnchor="end" className="profile-text">
        {formatDuration(maxT)}
      </text>
    </svg>
  );
}
