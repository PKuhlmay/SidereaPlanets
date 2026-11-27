// Pure helpers of the colour and tone cards: the values sent to the engine, the histogram graph and the
// Copy To views. Talking to the backend is in ../colour.ts.

import { contrastField, IDENTITY_MIX, TONE_LIMITS, type ChannelShifts, type RefineValues, type ToneSettings } from "./settings";

/** The cards that exist so far; the other Functions buttons only report that they are not available yet. */
export const toneCards = [
  "Histogram",
  "Gamma",
  "Color Mixing",
  "RGB Align",
  "RGB Balance",
  "Denoise/Deringing",
  "Masking",
  "Flip and Rotate",
  "Resize Image",
] as const;
export type ToneCard = (typeof toneCards)[number];
export const isToneCard = (name: string): name is ToneCard => (toneCards as readonly string[]).includes(name);

/** What `apply_colors` and the histogram commands take (src-tauri/src/color_commands.rs). */
export interface ColourPayload {
  shifts: ChannelShifts;
  red: number;
  green: number;
  blue: number;
  black: number;
  white: number;
  gamma: number;
  contrast: number;
  brightness: number;
  mix: { matrix: number[]; preserveBrightness: boolean };
  refine: RefinePayload;
}

/** `RefineSettings` of crates/core (camelCase JSON). */
export interface RefinePayload {
  denoise: number;
  deringing: number;
  deringingRadius: number;
  mask: { enabled: boolean; threshold: number; feather: number; invert: boolean };
}

export interface HistogramData {
  channels: number[][];
  luma: number[];
  bins: number;
}

/** The values of all cards plus the Contrast/Brightness sliders of the Wavelet tab. */
export function colourPayload(wavelet: { tone: ToneSettings; contrast: number; brightness: number }, refine: RefineValues): ColourPayload {
  const { tone } = wavelet;
  const { red, green, blue } = tone.shifts;
  return {
    shifts: { red: { ...red }, green: { ...green }, blue: { ...blue } },
    red: tone.red,
    green: tone.green,
    blue: tone.blue,
    black: tone.black,
    white: tone.white,
    gamma: tone.gamma,
    contrast: contrastField(wavelet.contrast),
    brightness: wavelet.brightness,
    mix: { matrix: tone.mix.matrix.map(clampWeight), preserveBrightness: tone.mix.preserveBrightness },
    refine: refinePayload(refine),
  };
}

/** Denoise/Deringing and Masking as `RefineSettings` of the core take them. */
export function refinePayload(refine: RefineValues): RefinePayload {
  return {
    denoise: refine.denoise,
    deringing: refine.deringing,
    deringingRadius: refine.radius,
    mask: { enabled: refine.maskOn, threshold: refine.threshold, feather: refine.feather, invert: refine.invert },
  };
}

/** One weight of Color Mixing, a whole percent within -200 … 200. */
export const clampWeight = (value: number): number =>
  Math.min(TONE_LIMITS.mix.max, Math.max(TONE_LIMITS.mix.min, Math.round(value)));

/** The ready-made matrices of the Color Mixing card; Preserve brightness stays as it is. */
export const mixPresets: { name: string; matrix: number[] }[] = [
  { name: "Neutral", matrix: [...IDENTITY_MIX] },
  { name: "Swap Red and Blue", matrix: [0, 0, 100, 0, 100, 0, 100, 0, 0] },
  // The weights of the luminance, so that all three channels show the brightness of the color image.
  { name: "Luminance Gray", matrix: [30, 59, 11, 30, 59, 11, 30, 59, 11] },
  // For infrared-pass and other single-filter data: the red channel drives all three outputs.
  { name: "Red to All Channels", matrix: [100, 0, 0, 100, 0, 0, 100, 0, 0] },
];

const isIdentity = (matrix: number[]): boolean => matrix.every((w, i) => w === IDENTITY_MIX[i]);

/** True when the values leave the image as it is, so the base image can be shown without a round trip. */
export function isNeutral(v: ColourPayload): boolean {
  const zero = (s: { x: number; y: number }) => s.x === 0 && s.y === 0;
  return (
    zero(v.shifts.red) &&
    zero(v.shifts.green) &&
    zero(v.shifts.blue) &&
    v.red === 100 &&
    v.green === 100 &&
    v.blue === 100 &&
    v.black === 0 &&
    v.white === 255 &&
    v.gamma === 1 &&
    v.contrast === 100 &&
    v.brightness === 0 &&
    isIdentity(v.mix.matrix) &&
    v.refine.denoise <= 0 &&
    v.refine.deringing <= 0 &&
    !v.refine.mask.enabled
  );
}

/**
 * Identifies the histogram a request would return. The one before the stretch does not depend on the stretch,
 * gamma, contrast and brightness, so moving those does not need a new one.
 */
export function histogramKey(v: ColourPayload, after: boolean): string {
  const before = { shifts: v.shifts, mix: v.mix, red: v.red, green: v.green, blue: v.blue, refine: v.refine };
  return JSON.stringify(after ? v : before);
}

/** Height of a bar on a logarithmic scale, 0..1: a few bright pixels stay visible next to a black sky. */
export const logHeight = (count: number, max: number): number => (max <= 0 ? 0 : Math.log1p(count) / Math.log1p(max));

export function histogramMax(h: HistogramData): number {
  return Math.max(0, ...h.luma, ...h.channels.flat());
}

/** The outline of one series as an SVG path. `closed` runs it down to the axis, for the filled area. */
export function histogramPath(counts: number[], max: number, width: number, height: number, closed = false): string {
  if (counts.length === 0) return "";
  const x = (i: number) => (counts.length === 1 ? 0 : (i / (counts.length - 1)) * width);
  const y = (c: number) => height - logHeight(c, max) * height;
  const line = counts.map((c, i) => `${i === 0 ? "M" : "L"}${x(i).toFixed(1)} ${y(c).toFixed(1)}`).join("");
  return closed ? `${line}L${width} ${height}L0 ${height}Z` : line;
}

/** Position on the graph (0..width) to a level on the 0..255 scale. */
export const xToLevel = (x: number, width: number): number => Math.min(255, Math.max(0, Math.round((x / width) * 255)));
export const levelToX = (level: number, width: number): number => (level / 255) * width;

/** The black marker stays at least one level below white, the white marker one above black. */
export const moveBlack = (level: number, white: number): number => Math.min(Math.max(0, level), white - 1);
export const moveWhite = (level: number, black: number): number => Math.max(Math.min(255, level), black + 1);

export type ToneView = "current" | "clipboard" | "difference";

/** The image the Wavelet tab shows: the current one, the copied one or the difference between them. */
export function pickToneImage<T>(view: ToneView, images: { current: T | null; clipboard: T | null; difference: T | null }): T | null {
  if (view === "clipboard") return images.clipboard ?? images.current;
  if (view === "difference") return images.difference ?? images.current;
  return images.current;
}

export interface Pixels {
  width: number;
  height: number;
  data: Uint8ClampedArray<ArrayBuffer>;
}

/** How much the difference is amplified, so that small changes show. A design decision. */
export const DIFFERENCE_GAIN = 4;

/** |a - b| per colour channel times DIFFERENCE_GAIN, opaque; null if the sizes differ. */
export function differencePixels(a: Pixels, b: Pixels): Pixels | null {
  if (a.width !== b.width || a.height !== b.height) return null;
  const data = new Uint8ClampedArray(a.data.length);
  for (let i = 0; i < data.length; i += 4) {
    for (let c = 0; c < 3; c++) data[i + c] = Math.abs(a.data[i + c] - b.data[i + c]) * DIFFERENCE_GAIN;
    data[i + 3] = 255;
  }
  return { width: a.width, height: a.height, data };
}

/** The Adjustments buttons that switch a view or open a tool instead of a card. */
const viewFunctions = ["View Compare", "Crop"];

/** Whether an Adjustments button does something; the others are dimmed and locked. */
export const isFunctionBuilt = (name: string): boolean => isToneCard(name) || viewFunctions.includes(name);

const functionHelps: Record<ToneCard, string> = {
  Histogram: "Black point, white point and brightness of the image.",
  Gamma: "Brightens or darkens the midtones.",
  "Color Mixing": "Builds each color channel from a mix of red, green and blue.",
  "Denoise/Deringing": "Smooths noise without blurring edges and softens the dark ring next to bright edges.",
  Masking: "Limits the sharpening to the planet, so the sky stays untouched.",
  "Flip and Rotate": "Mirrors the picture or turns it, by 90° or by any angle.",
  "Resize Image": "Makes the picture bigger or smaller when it is saved, by a percentage or to a size in pixels.",
  "RGB Align": "Shifts red, green and blue so that they lie on top of each other.",
  "RGB Balance": "Changes the share of red, green and blue.",
};

const viewHelps: Record<string, string> = {
  "View Compare": "Splits the image: the stack on the left, the result on the right.",
  Crop: "Cuts the picture to a rectangle when it is saved.",
};

/** The tooltip of an Adjustments button: what its card does, or that the button is locked. */
export const functionHelp = (name: string): string => (isToneCard(name) ? functionHelps[name] : (viewHelps[name] ?? "Not available yet"));
