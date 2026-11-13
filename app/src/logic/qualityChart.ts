// The quality chart of the Limit step (docs/design.md, "4 Limit"): one bar per frame, best first, the kept frames
// marked, and where the limit sits. Mirrors `limit_frames` in crates/core/src/align.rs so that the chart follows the
// limit value before Limit runs.

import type { LimitMode } from "./settings";

export interface QualityBar {
  /** Quality as a share of the best frame, 0..1. */
  height: number;
  kept: boolean;
}

export type LimitLine =
  /** Lowest Quality (%): a horizontal line at this share of the best quality, 0..1. */
  | { kind: "level"; at: number }
  /** The other modes: a vertical line after the last kept frame, as a share of the width, 0..1. */
  | { kind: "after"; at: number };

export interface QualityChart {
  bars: QualityBar[];
  frames: number;
  kept: number;
  line: LimitLine | null;
  /** The limit value next to the line. */
  value: string;
  /** Text alternative of the chart. */
  label: string;
}

export interface RatedFrame {
  quality: number | null;
  /** Kept by the last Limit. */
  checked: boolean;
}

export interface LimitValues {
  limitMode: LimitMode;
  limitPercent: number;
  limitCount: number;
}

/**
 * How many of the frames (qualities best first) a limit keeps; at least one if there is any. Frames/Apoint depends on
 * the quality of every point in every frame, which only the backend knows, so it has no count here.
 */
export function keptCount(sorted: number[], limit: LimitValues): number | null {
  const n = sorted.length;
  if (n === 0) return 0;
  let keep: number;
  switch (limit.limitMode) {
    case "lowestQuality": {
      const floor = (sorted[0] * limit.limitPercent) / 100;
      keep = sorted.filter((q) => q >= floor).length;
      break;
    }
    case "bestPercent":
      keep = Math.ceil((n * limit.limitPercent) / 100);
      break;
    case "bestCount":
      keep = Math.round(limit.limitCount);
      break;
    case "framesPerPoint":
      return null;
  }
  return Math.min(n, Math.max(1, keep));
}

function limitValue(limit: LimitValues): string {
  return limit.limitMode === "lowestQuality" || limit.limitMode === "bestPercent"
    ? `${limit.limitPercent}%`
    : String(Math.round(limit.limitCount));
}

/**
 * The bars of the chart. With more frames than `maxBars` one bar stands for several neighbouring frames: its height is
 * their mean quality and it counts as kept when at least half of them are kept.
 */
export function qualityChart(frames: RatedFrame[], limit: LimitValues, maxBars: number): QualityChart {
  const rated = frames
    .filter((f): f is { quality: number; checked: boolean } => f.quality !== null)
    .sort((a, b) => b.quality - a.quality);
  const n = rated.length;
  if (n === 0) return { bars: [], frames: 0, kept: 0, line: null, value: "", label: "No frame is rated yet." };

  const best = rated[0].quality;
  const count = keptCount(
    rated.map((f) => f.quality),
    limit,
  );
  // Frames/Apoint: the frames the last Limit kept.
  const kept = rated.map((f, i) => (count === null ? f.checked : i < count));
  const keptTotal = kept.filter(Boolean).length;
  const share = (q: number) => (best > 0 ? Math.min(1, Math.max(0, q / best)) : 0);

  const size = Math.max(1, Math.ceil(n / Math.max(1, Math.floor(maxBars))));
  const bars: QualityBar[] = [];
  for (let start = 0; start < n; start += size) {
    const group = rated.slice(start, start + size);
    const keptInGroup = kept.slice(start, start + size).filter(Boolean).length;
    const mean = group.reduce((sum, f) => sum + f.quality, 0) / group.length;
    bars.push({ height: share(mean), kept: keptInGroup * 2 >= group.length });
  }

  const lastKept = kept.lastIndexOf(true);
  const line: LimitLine =
    limit.limitMode === "lowestQuality"
      ? { kind: "level", at: Math.min(1, Math.max(0, limit.limitPercent / 100)) }
      : { kind: "after", at: (lastKept + 1) / n };

  return {
    bars,
    frames: n,
    kept: keptTotal,
    line,
    value: limitValue(limit),
    label: `Quality of ${n} frames, best first. ${keptTotal} ${keptTotal === 1 ? "is" : "are"} kept.`,
  };
}
