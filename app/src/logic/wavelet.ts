// Pure helpers of the Wavelet tab: the values of the panel as the backend wants them, linked sliders, and
// putting a preview of the Processing Area into the image. The calls to the backend are in ../wavelet.ts.

import { wholeNumber } from "./numbers";
import { defaultLayer, LAYER_COUNT, LAYER_RANGE, type WaveletFilter, type WaveletLayer, type WaveletScheme } from "./settings";

/** The values of the left panel that decide the result (`settings.wavelet` without the window state). */
export interface WaveletValues {
  scheme: WaveletScheme;
  filter: WaveletFilter;
  initialLayer: number;
  stepIncrement: number;
  layers: WaveletLayer[];
}

/** `WaveletSettings` of crates/core (camelCase JSON). */
export interface BackendSettings {
  scheme: WaveletScheme;
  filter: WaveletFilter;
  initial: number;
  step: number;
  layers: { enabled: boolean; gain: number; denoise: number; sharpen: number }[];
}

/** `ColourMode` of crates/core: how the colour of the stack is treated while sharpening. */
export type ColourMode = "colour" | "lrgb" | "gray";

/** The Colour and LRGB checkboxes as the mode of the backend. LRGB only counts together with Colour. */
export function colourMode(colour: boolean, lrgb: boolean): ColourMode {
  if (!colour) return "gray";
  return lrgb ? "lrgb" : "colour";
}

/**
 * What a Do All result depends on: the values of the layers and the colour mode. The Processing Area only decides
 * what the preview shows, and the colour and tone cards are applied when the image is saved, so neither belongs here.
 */
export function doAllKey(values: WaveletValues, mode: ColourMode): string {
  return JSON.stringify([toBackend(values), mode]);
}

/** Whether the last Do All (`done`) was made for this stacked image with these values. */
export function doAllIsCurrent<T>(done: { stack: T; key: string } | null, stack: T | null, key: string): boolean {
  return done !== null && stack !== null && done.stack === stack && done.key === key;
}

/**
 * Whether the preview of the Processing Area is calculated now. Only in the Wavelets step, where the values are changed:
 * the Save step shows what Do All made, and entering a step with a finished Do All for these values must not paint the
 * preview over that image or replace the status bar text of that run.
 */
export function previewWanted(state: { view: string; hasStack: boolean; busy: boolean; doAllCurrent: boolean }): boolean {
  return state.view === "wavelets" && state.hasStack && !state.busy && !state.doAllCurrent;
}

/** The slider position is ten times the factor of the layer (10 = 1.0). */
export function toBackend(values: WaveletValues): BackendSettings {
  return {
    scheme: values.scheme,
    filter: values.filter,
    initial: wholeNumber(values.initialLayer, 1),
    step: wholeNumber(values.stepIncrement, 0),
    layers: values.layers.map((l) => ({ enabled: l.enabled, gain: l.level / 10, denoise: l.denoise, sharpen: l.sharpen })),
  };
}

export function fromBackend(settings: BackendSettings): WaveletValues {
  return {
    scheme: settings.scheme,
    filter: settings.filter,
    initialLayer: settings.initial,
    stepIncrement: settings.step,
    layers: Array.from({ length: LAYER_COUNT }, (_, i) => {
      const l = settings.layers[i];
      return l ? { enabled: l.enabled, denoise: l.denoise, sharpen: l.sharpen, level: Math.round(l.gain * 10) } : defaultLayer();
    }),
  };
}

/**
 * Use Linked Wavelets: moving one slider moves all others by the same amount. All layers are linked;
 * this is the simplest rule. Returns the new slider positions.
 */
export function linkedLevels(levels: number[], index: number, level: number, linked: boolean): number[] {
  const clamp = (v: number) => Math.min(LAYER_RANGE.max, Math.max(LAYER_RANGE.min, Math.round(v)));
  const next = clamp(level);
  if (!linked) return levels.map((l, i) => (i === index ? next : l));
  const delta = next - levels[index];
  return levels.map((l, i) => (i === index ? next : clamp(l + delta)));
}

/** The answer of `wavelet_preview`: the area that was rendered, then its pixels. */
export interface PreviewAnswer {
  x: number;
  y: number;
  width: number;
  height: number;
  /** Four bytes per pixel, or one byte per pixel when `mono` (an image without colour). */
  rgba: Uint8ClampedArray;
  mono: boolean;
}

const PREVIEW_HEADER = 16;

/**
 * Binary layout: x, y, width, height (u32 LE each), then width * height * 4 bytes RGBA, or width * height bytes when the
 * image has no colour (`WaveletPreview::wire` in crates/session). The length tells which.
 */
export function parsePreview(buffer: ArrayBuffer): PreviewAnswer {
  const view = new DataView(buffer);
  const width = view.getUint32(8, true);
  const height = view.getUint32(12, true);
  const mono = buffer.byteLength - PREVIEW_HEADER === width * height;
  return {
    x: view.getUint32(0, true),
    y: view.getUint32(4, true),
    width,
    height,
    rgba: new Uint8ClampedArray(buffer, PREVIEW_HEADER, width * height * (mono ? 1 : 4)),
    mono,
  };
}

/** Copies the preview rows into the RGBA pixels of the whole image; a preview without colour gets red, green and blue alike and full alpha. */
export function writePreview(target: Uint8ClampedArray, imageWidth: number, preview: PreviewAnswer): void {
  const { rgba, width } = preview;
  for (let row = 0; row < preview.height; row++) {
    const to = ((preview.y + row) * imageWidth + preview.x) * 4;
    if (!preview.mono) {
      target.set(rgba.subarray(row * width * 4, (row + 1) * width * 4), to);
      continue;
    }
    for (let i = 0, from = row * width, at = to; i < width; i++, from++, at += 4) {
      const v = rgba[from];
      target[at] = v;
      target[at + 1] = v;
      target[at + 2] = v;
      target[at + 3] = 255;
    }
  }
}
