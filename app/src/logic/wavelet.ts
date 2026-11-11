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
 * Use Linked Wavelets: moving one slider moves all others by the same amount. Which layers the original links
 * is unobserved (docs/offene-fragen.md); this is the simplest reading. Returns the new slider positions.
 */
export function linkedLevels(levels: number[], index: number, level: number, linked: boolean): number[] {
  const clamp = (v: number) => Math.min(LAYER_RANGE.max, Math.max(LAYER_RANGE.min, Math.round(v)));
  const next = clamp(level);
  if (!linked) return levels.map((l, i) => (i === index ? next : l));
  const delta = next - levels[index];
  return levels.map((l, i) => (i === index ? next : clamp(l + delta)));
}

/** The answer of `wavelet_preview`: the area that was rendered, then its RGBA bytes. */
export interface PreviewAnswer {
  x: number;
  y: number;
  width: number;
  height: number;
  rgba: Uint8ClampedArray;
}

const PREVIEW_HEADER = 16;

/** Binary layout: x, y, width, height (u32 LE each), then width * height * 4 bytes RGBA. */
export function parsePreview(buffer: ArrayBuffer): PreviewAnswer {
  const view = new DataView(buffer);
  const width = view.getUint32(8, true);
  const height = view.getUint32(12, true);
  return {
    x: view.getUint32(0, true),
    y: view.getUint32(4, true),
    width,
    height,
    rgba: new Uint8ClampedArray(buffer, PREVIEW_HEADER, width * height * 4),
  };
}

/** Copies the preview rows into the RGBA pixels of the whole image. */
export function writePreview(target: Uint8ClampedArray, imageWidth: number, preview: PreviewAnswer): void {
  const rowBytes = preview.width * 4;
  for (let row = 0; row < preview.height; row++) {
    const to = ((preview.y + row) * imageWidth + preview.x) * 4;
    target.set(preview.rgba.subarray(row * rowBytes, (row + 1) * rowBytes), to);
  }
}
