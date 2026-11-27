// The project file (.sidereaplanets, docs/formats.md): the recording and the settings of the steps. Results of Align and Stack
// are not stored, they are calculated again.

import { applyDerotationSection, DEROTATION_KEYS, derotationSection } from "./derotation";
import { applyGeometrySection, geometrySection } from "./geometry";
import { IDENTITY_MIX, REFINE_LIMITS, TONE_LIMITS, type createSettings } from "./settings";

type Settings = ReturnType<typeof createSettings>;

export interface ProjectFile {
  formatVersion: number;
  recording: string[];
  sections: Record<string, unknown>;
}

const isRecord = (v: unknown): v is Record<string, unknown> => typeof v === "object" && v !== null && !Array.isArray(v);

/** A copy that no longer shares anything with the reactive settings. */
const copy = <T>(value: T): T => JSON.parse(JSON.stringify(value)) as T;

/** The Stack settings without De-rotate images, which has a section of its own (`derotation`, as in the Mac app). */
const withoutDerotation = (stack: Record<string, unknown>): Record<string, unknown> =>
  Object.fromEntries(Object.entries(stack).filter(([key]) => !(DEROTATION_KEYS as readonly string[]).includes(key)));

export function buildProject(recording: string[], settings: Settings): ProjectFile {
  // The threshold belongs to the points of the current frame, which are not stored.
  const align = Object.fromEntries(Object.entries(settings.align).filter(([key]) => key !== "threshold"));
  return {
    formatVersion: 1,
    recording: [...recording],
    sections: {
      align: copy(align),
      stack: copy(withoutDerotation(settings.stack)),
      wavelet: copy(settings.wavelet),
      colour: { colour: settings.colour, lrgb: settings.lrgb },
      refine: copy(settings.refine),
      geometry: geometrySection(settings.geometry),
      derotation: derotationSection(settings.stack),
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
  const { align, stack, wavelet, colour, refine, derotation, geometry } = project.sections;
  mergeKnown(settings.align, align);
  settings.align.brightnessThreshold = clamp(settings.align.brightnessThreshold, { min: 0, max: 255 });
  // Older files carry the locked `derotate: false` in `stack`; De-rotate images is read from its own section only.
  if (isRecord(stack)) mergeKnown(settings.stack, withoutDerotation(stack));
  applyDerotationSection(derotation, settings.stack);
  // The mixing matrix is taken only as a whole: nine finite numbers, each held to the limits of a weight.
  const matrix = [...settings.wavelet.tone.mix.matrix];
  mergeKnown(settings.wavelet, wavelet);
  settings.wavelet.tone.mix.matrix = mixMatrix(wavelet) ?? matrix;
  mergeKnown(settings.refine, refine);
  applyGeometrySection(geometry, settings.geometry);
  const r = settings.refine;
  r.denoise = clamp(r.denoise, REFINE_LIMITS.percent);
  r.deringing = clamp(r.deringing, REFINE_LIMITS.percent);
  r.radius = clamp(r.radius, REFINE_LIMITS.radius);
  r.threshold = clamp(r.threshold, REFINE_LIMITS.threshold);
  r.feather = clamp(r.feather, REFINE_LIMITS.feather);
  if (isRecord(colour)) {
    if (typeof colour.colour === "boolean") settings.colour = colour.colour;
    if (typeof colour.lrgb === "boolean") settings.lrgb = colour.lrgb;
  }
}

const clamp = (value: number, range: { min: number; max: number }): number => Math.min(range.max, Math.max(range.min, Math.round(value)));

/** `wavelet.tone.mix.matrix` of a project when it is nine finite numbers, held to the limits of a weight; null otherwise. */
function mixMatrix(wavelet: unknown): number[] | null {
  const tone = isRecord(wavelet) ? wavelet.tone : undefined;
  const mix = isRecord(tone) ? tone.mix : undefined;
  const matrix = isRecord(mix) ? mix.matrix : undefined;
  if (!Array.isArray(matrix) || matrix.length !== IDENTITY_MIX.length) return null;
  if (!matrix.every((w): w is number => typeof w === "number" && Number.isFinite(w))) return null;
  return matrix.map((w) => clamp(w, TONE_LIMITS.mix));
}
