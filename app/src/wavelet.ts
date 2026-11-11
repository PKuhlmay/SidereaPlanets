// The Wavelet tab: live preview of the Processing Area, Process, Do All and schemes, through the commands of
// src-tauri/src/wavelet_commands.rs (docs/wavelets.md). The image to sharpen lives in the backend; the stack
// step (RK-006) hands it over with `WaveletState::set_source` and puts the same image into `session.stackImage`.

import { invoke } from "@tauri-apps/api/core";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { watch } from "vue";
import { runJob } from "./jobs";
import { defaultLayer } from "./logic/settings";
import { processingArea } from "./logic/viewport";
import { colourMode, fromBackend, parsePreview, toBackend, writePreview, type BackendSettings } from "./logic/wavelet";
import { session, settings, status } from "./state";

// The design preview of dev builds runs in a plain browser; there is no backend to ask.
const inTauri = (): boolean => "__TAURI_INTERNALS__" in window;

// The pixels the image area shows: a copy of the stack with the preview of the Processing Area written into it.
// The ImageData objects share this buffer, so a new preview costs the area, not the whole image.
let working: { stack: ImageData; pixels: Uint8ClampedArray<ArrayBuffer> } | null = null;

function workingPixels(stack: ImageData): Uint8ClampedArray<ArrayBuffer> {
  if (!working || working.stack !== stack) {
    const pixels = new Uint8ClampedArray(stack.data.length);
    pixels.set(stack.data);
    working = { stack, pixels };
  }
  return working.pixels;
}

/** Colour and LRGB decide how the stack is sharpened: per channel, on the luminance only, or as grayscale. */
const mode = () => colourMode(settings.colour, settings.lrgb);

const area = (stack: ImageData) => {
  const a = processingArea({ w: stack.width, h: stack.height }, settings.menu.processingArea);
  return { x: a.x, y: a.y, width: a.w, height: a.h };
};

/** Calculates the Processing Area with the current values and shows it in the image. */
async function renderPreview(cancelled?: () => boolean): Promise<string> {
  const stack = session.stackImage;
  if (!stack) throw new Error("no stacked image");
  // Time from the request to the finished drawing in the image area, shown in the status bar (docs/gui-toolkit.md).
  const started = performance.now();
  const answer = await invoke<ArrayBuffer>("wavelet_preview", { settings: toBackend(settings.wavelet), area: area(stack), mode: mode() });
  // Process was cancelled while the area was being calculated: the old picture stays.
  if (cancelled?.()) return "";
  const pixels = workingPixels(stack);
  writePreview(pixels, stack.width, parsePreview(answer));
  session.processedImage = new ImageData(pixels, stack.width, stack.height);
  status.action = `Preview: ${Math.round(performance.now() - started)} ms`;
  return "Calculating wavelets done";
}

// Only one request runs at a time; while it runs, further changes are folded into one more request.
let rendering = false;
let again = false;

async function refreshPreview(): Promise<void> {
  if (rendering) {
    again = true;
    return;
  }
  rendering = true;
  try {
    do {
      again = false;
      status.result = await renderPreview();
    } while (again);
  } catch (e) {
    status.result = `Error: ${String(e)}`;
  } finally {
    rendering = false;
  }
}

/**
 * Process: calculates the Processing Area now. The area is small, so the backend does not stop in the middle; Cancel
 * drops the result when it arrives.
 */
export async function process(): Promise<void> {
  await runJob("Wavelet", (job) => renderPreview(() => job.cancelled));
}

/**
 * Do All: applies the values to the whole image. Cancel and Pause reach the backend between two layers; a cancelled
 * run leaves the earlier result, here and in the backend (what Save image writes), as it was.
 */
export async function doAll(): Promise<void> {
  await runJob("Wavelet", async (job) => {
    job.progress(0, 1, "Do_all processing");
    // The backend reports each finished layer.
    const unlisten = await listen<{ done: number; total: number }>("wavelet-progress", ({ payload }) =>
      job.progress(payload.done, payload.total),
    );
    let buffer: ArrayBuffer;
    try {
      buffer = await invoke<ArrayBuffer>("wavelet_do_all", { settings: toBackend(settings.wavelet), mode: mode() });
    } finally {
      unlisten();
    }
    if (job.cancelled) return;
    // Binary layout: width (u32 LE), height (u32 LE), then RGBA bytes.
    const view = new DataView(buffer);
    const width = view.getUint32(0, true);
    const height = view.getUint32(4, true);
    session.processedImage = new ImageData(new Uint8ClampedArray(buffer, 8, width * height * 4), width, height);
    // The next preview starts from the stack again, so that its area is not painted over this result.
    working = null;
    return "Do All finished";
  });
}

// Load Scheme and Save Scheme. The dialog is the one place RK-009 replaces with its `pick_scheme_path` command.
const schemeFilter = [{ name: "the reference program Wavelet scheme settings", extensions: ["rwv"] }];

async function pickSchemePath(mode: "open" | "save"): Promise<string | null> {
  if (mode === "open") {
    const picked = await openDialog({ multiple: false, directory: false, filters: schemeFilter });
    return typeof picked === "string" ? picked : null;
  }
  return saveDialog({ filters: schemeFilter, defaultPath: "scheme.rwv" });
}

export async function loadScheme(): Promise<void> {
  const path = await pickSchemePath("open");
  if (!path) return;
  try {
    const loaded = fromBackend(await invoke<BackendSettings>("load_wavelet_scheme", { path }));
    Object.assign(settings.wavelet, { scheme: loaded.scheme, filter: loaded.filter, initialLayer: loaded.initialLayer, stepIncrement: loaded.stepIncrement });
    loaded.layers.forEach((layer, i) => Object.assign(settings.wavelet.layers[i], layer));
    status.result = `Scheme loaded: ${path}`;
  } catch (e) {
    status.result = `Error: ${String(e)}`;
  }
}

export async function saveScheme(): Promise<void> {
  const path = await pickSchemePath("save");
  if (!path) return;
  try {
    await invoke("save_wavelet_scheme", { path, settings: toBackend(settings.wavelet) });
    status.result = `Scheme saved: ${path}`;
  } catch (e) {
    status.result = `Error: ${String(e)}`;
  }
}

// Hold Wavelet Setting keeps the values when a new stacked image arrives; without it they start over.
// (The original's behaviour is a guess, see docs/offene-fragen.md.)
watch(
  () => session.stackImage,
  (stack) => {
    if (!inTauri() || !stack || settings.wavelet.hold) return;
    settings.wavelet.layers.forEach((layer) => Object.assign(layer, defaultLayer()));
  },
);

// Every change of a value that matters, of the Processing Area, or a new stack recalculates the preview.
const AUTOMATIC_DELAY_MS = 600;
let automatic: number | undefined;

watch(
  [() => JSON.stringify(toBackend(settings.wavelet)), mode, () => settings.menu.processingArea, () => session.stackImage, () => session.tab],
  () => {
    if (!inTauri() || session.tab !== "wavelet" || !session.stackImage || status.busy) return;
    status.action = "layer setting changed";
    void refreshPreview();
    // Automatic: once the values stop changing, apply them to the whole image as well.
    window.clearTimeout(automatic);
    if (settings.wavelet.automatic) automatic = window.setTimeout(() => void doAll(), AUTOMATIC_DELAY_MS);
  },
);
