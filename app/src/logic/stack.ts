// What the Stack button sends to the backend, collected from the window state (docs/stacking.md).

import type { settings as settingsValue, session as sessionValue } from "../state";
import { centreOfGravityThreshold } from "./align";
import { derotationRequest, type DerotationRequest } from "./derotation";
import { wholeNumber } from "./numbers";

/** Matches `StackSettings` of the Tauri command `stack` (app/src-tauri/src/stack_commands.rs). */
export interface StackRequest {
  referenceFrame: number;
  points: { x: number; y: number }[];
  frames: number[];
  boxSize: number;
  maxMovement: number;
  nearestPoints: number;
  correctGeometry: boolean;
  normalize: boolean;
  edgeSkip: number;
  roi: { x: number; y: number; width: number; height: number } | null;
  /** Brightness threshold of Align by center of gravity, null while it is off. */
  centreOfGravity: number | null;
  /** De-rotate images (RK-023); left out while the check box is off. */
  derotation?: DerotationRequest;
}

type Session = Pick<typeof sessionValue, "referenceFrame" | "alignpoints" | "frames" | "roi">;
type Settings = Pick<typeof settingsValue, "align" | "stack">;

/** The checked alignpoints and checked frames together with the options of the Stack tab. */
export function buildStackRequest(session: Session, settings: Settings): StackRequest {
  const derotation = derotationRequest(settings.stack);
  return {
    ...(derotation ? { derotation } : {}),
    referenceFrame: session.referenceFrame,
    points: session.alignpoints.filter((p) => p.checked).map(({ x, y }) => ({ x, y })),
    frames: session.frames.filter((f) => f.checked).map((f) => f.index),
    boxSize: wholeNumber(settings.align.boxSize, 1),
    maxMovement: wholeNumber(settings.align.maxMovement, 0),
    nearestPoints: wholeNumber(settings.stack.nearestPoints, 1),
    correctGeometry: settings.stack.correctGeometry,
    normalize: settings.stack.normalize,
    edgeSkip: wholeNumber(settings.stack.edgePixels, 0),
    roi: session.roi,
    centreOfGravity: centreOfGravityThreshold(settings.align),
  };
}

/** Matches `SequenceSettings` of the Tauri commands `save_aligned` and `save_integrated` (src-tauri/src/sequence_commands.rs). */
export interface SequenceRequest {
  stack: StackRequest;
  framerate: number;
  sortOnQuality: boolean;
  averageFrames: number;
}

/** What *Save Aligned* and *Save Integrated* send: the request of Stack plus the options of *Create AVI*. */
export function buildSequenceRequest(session: Session, settings: Settings): SequenceRequest {
  return {
    stack: buildStackRequest(session, settings),
    framerate: wholeNumber(settings.stack.framerate, 1),
    sortOnQuality: settings.stack.sortOnQuality,
    averageFrames: wholeNumber(settings.stack.averageFrames, 1),
  };
}

/** The status bar after Stack: the frames and the size ("Stacked 10 frames · 1024×680"), then the range of the values. */
export function stackMessages(frames: number, width: number, height: number): { action: string; result: string } {
  return { action: `Stacked ${frames} ${frames === 1 ? "frame" : "frames"} · ${width}×${height}`, result: "Value range 0–255" };
}
