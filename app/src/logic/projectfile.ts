// The project file (.rkproj, docs/formats.md): the recording and the settings of the steps. Results of Align and Stack
// are not stored, they are calculated again.

import type { createSettings } from "./settings";

type Settings = ReturnType<typeof createSettings>;

export interface ProjectFile {
  formatVersion: number;
  recording: string[];
  sections: Record<string, unknown>;
}

const isRecord = (v: unknown): v is Record<string, unknown> => typeof v === "object" && v !== null && !Array.isArray(v);

/** A copy that no longer shares anything with the reactive settings. */
const copy = <T>(value: T): T => JSON.parse(JSON.stringify(value)) as T;

export function buildProject(recording: string[], settings: Settings): ProjectFile {
  // The threshold belongs to the points of the current frame, which are not stored.
  const align = Object.fromEntries(Object.entries(settings.align).filter(([key]) => key !== "threshold"));
  return {
    formatVersion: 1,
    recording: [...recording],
    sections: {
      align: copy(align),
      stack: copy(settings.stack),
      wavelet: copy(settings.wavelet),
      colour: { colour: settings.colour, lrgb: settings.lrgb },
    },
  };
}

/**
 * Copies the values of `source` into `target`, but only where `target` has the key and the value has the same type.
 * A hand-edited or older file therefore cannot put wrong types into the settings; keys it lacks keep their value.
 */
export function mergeKnown(target: Record<string, unknown>, source: unknown): void {
  if (!isRecord(source)) return;
  for (const key of Object.keys(target)) {
    if (!(key in source)) continue;
    const have = target[key];
    const want = source[key];
    if (Array.isArray(have)) {
      if (Array.isArray(want)) mergeArray(have, want);
    } else if (isRecord(have)) {
      mergeKnown(have, want);
    } else if (typeof want === typeof have && (typeof want !== "number" || Number.isFinite(want))) {
      target[key] = want;
    }
  }
}

function mergeArray(have: unknown[], want: unknown[]): void {
  have.forEach((item, i) => {
    if (i >= want.length) return;
    if (isRecord(item)) mergeKnown(item, want[i]);
    else if (typeof want[i] === typeof item) have[i] = want[i];
  });
}

/** Puts the settings of a project into the current ones. */
export function applyProject(settings: Settings, project: ProjectFile): void {
  const { align, stack, wavelet, colour } = project.sections;
  mergeKnown(settings.align, align);
  mergeKnown(settings.stack, stack);
  mergeKnown(settings.wavelet, wavelet);
  if (isRecord(colour)) {
    if (typeof colour.colour === "boolean") settings.colour = colour.colour;
    if (typeof colour.lrgb === "boolean") settings.lrgb = colour.lrgb;
  }
}
