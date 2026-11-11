// What the Stack button sends to the backend, collected from the window state (docs/stacking.md).

import type { settings as settingsValue, session as sessionValue } from "../state";

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
}

type Session = Pick<typeof sessionValue, "referenceFrame" | "alignpoints" | "frames" | "roi">;
type Settings = Pick<typeof settingsValue, "align" | "stack">;

// The number fields accept any typed value (docs/ui.md); the backend takes whole numbers from zero (one point at least).
const whole = (value: number, min: number): number => (Number.isFinite(value) ? Math.max(min, Math.round(value)) : min);

/** The checked alignpoints and checked frames together with the options of the Stack tab. */
export function buildStackRequest(session: Session, settings: Settings): StackRequest {
  return {
    referenceFrame: session.referenceFrame,
    points: session.alignpoints.filter((p) => p.checked).map(({ x, y }) => ({ x, y })),
    frames: session.frames.filter((f) => f.checked).map((f) => f.index),
    boxSize: whole(settings.align.boxSize, 1),
    maxMovement: whole(settings.align.maxMovement, 0),
    nearestPoints: whole(settings.stack.nearestPoints, 1),
    correctGeometry: settings.stack.correctGeometry,
    normalize: settings.stack.normalize,
    edgeSkip: whole(settings.stack.edgePixels, 0),
    roi: session.roi,
  };
}
