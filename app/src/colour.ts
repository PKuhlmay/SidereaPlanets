// The colour and tone cards and Copy To: sends the base image and the card values to the engine and shows the
// result in the Wavelet tab. The values are applied to the base image, never on top of each other.

import { invoke } from "@tauri-apps/api/core";
import { watch } from "vue";
import { handlers } from "./actions";
import { COMPARE, CROP, toggleCompare, toggleCrop } from "./cropTool";
import { failureText } from "./jobs";
import {
  colourPayload,
  differencePixels,
  refinePayload,
  isNeutral,
  isToneCard,
  type ColourPayload,
  type HistogramData,
} from "./logic/colour";
import { wholeNumber } from "./logic/numbers";
import { defaultTone, type ChannelShifts } from "./logic/settings";
import { session, settings, status } from "./state";

/** The image the cards start from: the sharpened one, or the stack before Wavelet has run. */
const baseImage = (): ImageData | null => session.processedImage ?? session.stackImage;

/** What the Wavelet tab shows without Before: the base with the cards applied. */
const currentImage = (): ImageData | null => session.adjustedImage ?? baseImage();

const payload = (): ColourPayload => colourPayload(settings.wavelet, settings.refine);

/** Grayscale recordings have no colour channels to align or balance. */
export const isMono = (): boolean => session.info?.color === "Mono";

function decode(buffer: ArrayBuffer): ImageData {
  // Binary layout: width (u32 LE), height (u32 LE), then RGBA bytes.
  const view = new DataView(buffer);
  const width = view.getUint32(0, true);
  const height = view.getUint32(4, true);
  return new ImageData(new Uint8ClampedArray(buffer, 8, width * height * 4), width, height);
}

// Which base image the engine holds. Sending it is the expensive part, so it only happens when the base changes.
let sentBase: ImageData | null = null;

async function ensureSource(): Promise<boolean> {
  const image = baseImage();
  if (!image) return false;
  if (sentBase !== image) {
    const bytes = new Uint8Array(image.data.buffer, image.data.byteOffset, image.data.byteLength);
    await invoke("set_color_source", bytes, { headers: { width: String(image.width), height: String(image.height) } });
    sentBase = image;
  }
  return true;
}

// Applies the values to the base. While one request runs, later changes only set a flag, so dragging a slider
// costs one request at a time and the last position always wins.
let running = false;
let dirty = false;

async function refresh(): Promise<void> {
  if (running) {
    dirty = true;
    return;
  }
  running = true;
  try {
    do {
      dirty = false;
      const values = payload();
      if (isNeutral(values) || !(await ensureSource())) {
        session.adjustedImage = null;
      } else {
        const base = baseImage();
        const buffer = await invoke<ArrayBuffer>("apply_colors", { settings: values });
        // The base may have changed while the engine worked; the next round then starts from the new one.
        if (base === baseImage()) session.adjustedImage = decode(buffer);
        else dirty = true;
      }
    } while (dirty);
  } catch (e) {
    session.adjustedImage = null;
    status.result = failureText("Color and tone", e);
  } finally {
    running = false;
  }
}

/** The histogram of the base with the card values applied: before the stretch, or the final result. */
export async function requestHistogram(after: boolean): Promise<HistogramData | null> {
  if (!(await ensureSource())) return null;
  return invoke<HistogramData>("color_histogram", { settings: payload(), after });
}

/** RGB Align, Auto: puts the shifts that move red and blue onto green into the fields. */
export async function autoAlign(): Promise<void> {
  try {
    if (!(await ensureSource())) {
      status.result = "RGB Align: no image yet";
      return;
    }
    const found = await invoke<ChannelShifts>("estimate_rgb_align", { maxShift: wholeNumber(settings.wavelet.tone.maxShift, 1) });
    settings.wavelet.tone.shifts = found;
    status.result = "RGB Align: shifts found";
  } catch (e) {
    status.result = failureText("RGB Align", e);
  }
}

/** Histogram, Auto: sets black and white point so that `clip` percent are cut off at each end. */
export async function autoLevels(): Promise<void> {
  try {
    if (!(await ensureSource())) {
      status.result = "Histogram: no image yet";
      return;
    }
    const tone = settings.wavelet.tone;
    const found = await invoke<{ black: number; white: number }>("auto_histogram_points", { settings: payload(), clip: tone.clip });
    tone.black = Math.round(found.black);
    tone.white = Math.round(found.white);
  } catch (e) {
    status.result = failureText("Histogram", e);
  }
}

/** Closes a card: the Adjustments button comes up, the values stay. */
export function closeFunction(name: string): void {
  const open = settings.wavelet.functions;
  const at = open.indexOf(name);
  if (at >= 0) open.splice(at, 1);
}

/** An Adjustments button: opens or closes its card. The buttons without a card are locked; the check is a safeguard. */
export function toggleFunction(name: string): void {
  if (name === COMPARE) return toggleCompare();
  if (name === CROP) return toggleCrop();
  if (!isToneCard(name)) {
    status.result = `${name}: not available yet`;
    return;
  }
  if (settings.wavelet.functions.includes(name)) closeFunction(name);
  else settings.wavelet.functions.push(name);
}

/** Puts all card values and Contrast/Brightness back to unchanged. */
export function resetTone(): void {
  // Assigned into the existing object: the open cards hold a reference to it.
  Object.assign(settings.wavelet.tone, defaultTone());
  settings.wavelet.contrast = 250;
  settings.wavelet.brightness = 0;
}

const copyOf = (image: ImageData): ImageData => new ImageData(new Uint8ClampedArray(image.data), image.width, image.height);

// Remember / Restore / Difference (the last group of the Adjustments panel).
function copyTo() {
  const image = currentImage();
  if (!image) {
    status.result = "Remember: no image yet";
    return;
  }
  session.clipboardImage = copyOf(image);
  session.differenceImage = null;
  session.toneView = "current";
  status.result = "Image remembered";
}

function loadTo() {
  if (!session.clipboardImage) {
    status.result = "Restore: nothing remembered yet";
    return;
  }
  // The copy already has the card values in it, so they go back to unchanged; applying them again would count twice.
  session.processedImage = copyOf(session.clipboardImage);
  resetTone();
  session.toneView = "current";
  status.result = "Remembered image restored";
}

function updateDifference() {
  const now = currentImage();
  const copy = session.clipboardImage;
  const diff = now && copy ? differencePixels(now, copy) : null;
  session.differenceImage = diff ? new ImageData(diff.data, diff.width, diff.height) : null;
  return diff !== null;
}

function difference() {
  if (!session.clipboardImage) {
    status.result = "Difference: nothing remembered yet";
    return;
  }
  if (!updateDifference()) {
    status.result = "Difference: the images differ in size";
    return;
  }
  session.toneView = "difference";
  status.result = "Difference shown";
}

/** Connects the cards and Remember to the window. Called once from App.vue. */
export function startColour(): void {
  handlers.copyTo = copyTo;
  handlers.loadTo = loadTo;
  handlers.difference = difference;

  watch([() => JSON.stringify(payload()), baseImage], () => void refresh(), { immediate: true });
  watch([maskWanted, () => JSON.stringify(refinePayload(settings.refine)), baseImage], () => void refreshMaskOverlay());
  // The difference follows the current image while it is shown.
  watch(currentImage, () => {
    if (session.toneView === "difference" && !updateDifference()) session.toneView = "current";
  });
}

/** Show mask counts while it is on, the Masking card is open and the Wavelets step is shown. */
const maskWanted = (): boolean =>
  session.showMask && settings.wavelet.functions.includes("Masking") && session.view === "wavelets";

// The newest request wins; an older answer that arrives late is dropped.
let maskRequest = 0;

/** Fetches the image with the tinted mask for Show mask, or drops it when Show mask does not count (only a view, never saved). */
export async function refreshMaskOverlay(): Promise<void> {
  const request = ++maskRequest;
  if (!maskWanted()) {
    session.maskImage = null;
    return;
  }
  try {
    if (!(await ensureSource())) {
      session.maskImage = null;
      return;
    }
    const buffer = await invoke<ArrayBuffer>("refine_mask_overlay", { settings: refinePayload(settings.refine) });
    if (request === maskRequest) session.maskImage = decode(buffer);
  } catch (e) {
    if (request === maskRequest) {
      session.maskImage = null;
      status.result = failureText("Show mask", e);
    }
  }
}
