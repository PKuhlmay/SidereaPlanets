// Pure helpers of the Align tab: what is sent to the backend commands (app/src-tauri/src/align_commands.rs)
// and how their answers become what the window shows. The calls themselves are in ../align.ts.

import { wholeNumber } from "./numbers";
import type { createSettings } from "./settings";
import type { Size } from "./viewport";

export type AlignSettings = ReturnType<typeof createSettings>["align"];

/** A rectangle in image pixels, from the top left corner. */
export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

// Answers of the backend commands.
export interface AlignpointsAnswer {
  points: { x: number; y: number }[];
  /** Number of points before the threshold: the upper end of the Threshold slider. */
  total: number;
  threshold: number;
}

export interface AlignedAnswer {
  /** One rating per frame, in frame order. */
  frames: { index: number; quality: number; shiftX: number; shiftY: number; difference: number }[];
  ranking: number[];
}

export interface LimitedAnswer {
  frames: number[];
  /** Per point in use: mean distance from the shift of its frame, `null` where it was never found. */
  pointShifts: ([number, number] | null)[];
}

export interface AlignProgress {
  done: number;
  total: number;
  elapsedMs: number;
}

export function setAlignpointsArgs(frame: number, align: AlignSettings, roi: Rect | null) {
  return {
    frame,
    settings: {
      minDistance: wholeNumber(align.minDistance, 0),
      edgeDistance: wholeNumber(align.edgeDistance, 0),
      maxMovement: wholeNumber(align.maxMovement, 0),
      roi,
      intensity: align.intensity,
      lo: align.lo,
      hi: align.hi,
      boxSize: wholeNumber(align.boxSize, 1),
    },
  };
}

export function alignArgs(align: AlignSettings, roi: Rect | null) {
  return { settings: { boxSize: wholeNumber(align.boxSize, 1), maxMovement: wholeNumber(align.maxMovement, 0), roi } };
}

/** Lowest Quality (%) and Best Frames (%) take the percent field, the two count modes the number field. */
export function limitArgs(align: AlignSettings) {
  const percent = align.limitMode === "lowestQuality" || align.limitMode === "bestPercent";
  return { settings: { mode: align.limitMode, value: percent ? align.limitPercent : wholeNumber(align.limitCount, 1) } };
}

/** Quality in percent of the best frame, as the Framelist shows it. */
export function qualityPercent(qualities: number[]): number[] {
  const best = Math.max(...qualities, 0);
  return qualities.map((q) => (best > 0 ? (q / best) * 100 : 0));
}

/** The rectangle spanned by two corners, kept inside the image and with whole pixels. */
export function rectBetween(a: { x: number; y: number }, b: { x: number; y: number }, image: Size): Rect {
  const clamp = (v: number, max: number) => Math.min(max, Math.max(0, Math.round(v)));
  const x0 = clamp(Math.min(a.x, b.x), image.w);
  const y0 = clamp(Math.min(a.y, b.y), image.h);
  const x1 = clamp(Math.max(a.x, b.x), image.w);
  const y1 = clamp(Math.max(a.y, b.y), image.h);
  return { x: x0, y: y0, width: x1 - x0, height: y1 - y0 };
}

/** A rectangle dragged by accident (a click) is not a region. */
export const MIN_ROI_SIDE = 8;
export const isRegion = (rect: Rect): boolean => rect.width >= MIN_ROI_SIDE && rect.height >= MIN_ROI_SIDE;

/** Whether a region leaves room for any point: it must be wider and higher than twice Min distance from edge. */
export const roomForPoints = (rect: Rect, edgeDistance: number): boolean =>
  rect.width > 2 * edgeDistance && rect.height > 2 * edgeDistance;

/**
 * The status bar after Keep Best Frames: how many frames stay and the quality of the weakest of them in percent of the
 * best frame (`keptQualities`, the percentages of the Frame List). That meaning is a conjecture (docs/workflow.md).
 */
export function limitMessages(kept: number, total: number, keptQualities: number[]): { action: string; result: string } {
  const action = `Keeping ${kept} of ${total} frames`;
  const stacked = `${kept} ${kept === 1 ? "frame" : "frames"} will be stacked`;
  if (keptQualities.length === 0) return { action, result: stacked };
  return { action, result: `${stacked} · weakest kept: ${Math.min(...keptQualities).toFixed(0)}%` };
}
