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
  return {
    settings: {
      boxSize: wholeNumber(align.boxSize, 1),
      maxMovement: wholeNumber(align.maxMovement, 0),
      roi,
      centreOfGravity: centreOfGravityThreshold(align),
    },
  };
}

/** The Brightness threshold of Align by center of gravity, held to 0–255; null while the option is off. */
export function centreOfGravityThreshold(align: Pick<AlignSettings, "centreOfGravity" | "brightnessThreshold">): number | null {
  return align.centreOfGravity ? Math.min(255, wholeNumber(align.brightnessThreshold, 0)) : null;
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

/** The frames in the order of the Frame List: after Align best first, equal quality by the smaller number; before Align as they come. */
export function rankedFrames<T extends { index: number; quality: number | null }>(frames: T[]): T[] {
  if (!frames.some((f) => f.quality !== null)) return [...frames];
  return [...frames].sort((a, b) => (b.quality ?? -1) - (a.quality ?? -1) || a.index - b.index);
}

/** The first frame of the ranking (0-based), null before Align. */
export function bestFrame(frames: { index: number; quality: number | null }[]): number | null {
  return frames.some((f) => f.quality !== null) ? (rankedFrames(frames)[0]?.index ?? null) : null;
}

/** The tooltip of Go to Best Frame. */
export const goToBestFrameHelp = "Show the best frame and open Alignpoints, to line all frames up on it.";

/** The note in the step Alignpoints after Go to Best Frame; the number counts from 1 like the frame bar. */
export const bestFrameHint = (frame: number): string =>
  `Best frame: ${frame + 1}. Run Set Alignpoints and Align again to line all frames up on this one.`;
