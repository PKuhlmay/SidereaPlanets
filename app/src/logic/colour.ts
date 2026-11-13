// Pure helpers of the colour and tone cards (RK-008): the values sent to the engine, the histogram graph and the
// Copy To views. Talking to the backend is in ../colour.ts.

import { contrastField, type ChannelShifts, type ToneSettings } from "./settings";

/** The cards that exist so far; the other Functions buttons only report that they are not available yet. */
export const toneCards = ["Histogram", "Gamma", "RGB Align", "RGB Balance"] as const;
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
}

export interface HistogramData {
  channels: number[][];
  luma: number[];
  bins: number;
}

/** The values of all cards plus the Contrast/Brightness sliders of the Wavelet tab. */
export function colourPayload(wavelet: { tone: ToneSettings; contrast: number; brightness: number }): ColourPayload {
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
  };
}

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
    v.brightness === 0
  );
}

/**
 * Identifies the histogram a request would return. The one before the stretch does not depend on the stretch,
 * gamma, contrast and brightness, so moving those does not need a new one.
 */
export function histogramKey(v: ColourPayload, after: boolean): string {
  const before = { shifts: v.shifts, red: v.red, green: v.green, blue: v.blue };
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

/** How much the difference is amplified, so that small changes show. Our choice, the original was not observed. */
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

/** Whether an Adjustments button does something; the others are dimmed and locked (MAC-014). */
export const isFunctionBuilt = (name: string): boolean => isToneCard(name);

const functionHelps: Record<ToneCard, string> = {
  Histogram: "Black point, white point and brightness of the image.",
  Gamma: "Brightens or darkens the midtones.",
  "RGB Align": "Shifts red, green and blue so that they lie on top of each other.",
  "RGB Balance": "Changes the share of red, green and blue.",
};

/** The tooltip of an Adjustments button: what its card does, or that the button is locked. */
export const functionHelp = (name: string): string => (isToneCard(name) ? functionHelps[name] : "Not available yet");
