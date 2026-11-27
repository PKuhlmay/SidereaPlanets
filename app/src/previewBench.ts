// Measures the live preview of the Wavelets step in the real window, with the real commands, the real viewer and the
// real webview (WKWebView, WebView2 or WebKitGTK). Start the app with
//
//   PLANETS_PREVIEW_BENCH=<recording> PLANETS_PREVIEW_BENCH_OUT=<file.json> <app>
//
// It opens the recording, takes frame 1 as the stacked image, runs Do All and then moves the first layer slider: one
// step after the other (latency of one step) and as fast as a hand drags it (frames per second). The result goes to the
// file and the app quits. Commands: src-tauri/src/bench_commands.rs; the numbers are discussed in docs/wavelets.md.

import { invoke } from "@tauri-apps/api/core";
import { nextTick } from "vue";
import { openRecording } from "./recording";
import { previewTiming, type PreviewSample } from "./previewTiming";
import { session, settings } from "./state";
import { previewIdle, runDoAll } from "./wavelet";

const STEPS = 100;
const DRAG_MS = 3000;
const THROUGHPUT_MS = 2000;
/** A mouse reports every few milliseconds; the browser hands the slider one `input` event per frame. */
const DRAG_EVENT_MS = 8;

/** Median, 95th percentile and extremes of a list of numbers, rounded to a tenth. */
export function summarize(values: number[]): { median: number; p95: number; min: number; max: number } {
  const sorted = [...values].sort((a, b) => a - b);
  const at = (p: number) => sorted[Math.round((sorted.length - 1) * p)] ?? 0;
  const r = (v: number) => Math.round(v * 10) / 10;
  return { median: r(at(0.5)), p95: r(at(0.95)), min: r(sorted[0] ?? 0), max: r(sorted[sorted.length - 1] ?? 0) };
}

const sleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

/** Resolves after the browser has painted the frame that follows now. */
const painted = () => new Promise<void>((resolve) => requestAnimationFrame(() => setTimeout(resolve, 0)));

const level = (i: number) => 10 + (i % 50);

/** Moves the slider and waits until the preview of that value is in the image. */
async function step(i: number): Promise<void> {
  settings.wavelet.layers[0].level = level(i);
  await nextTick();
  await previewIdle();
}

/**
 * One slider step after the other. `ready` is the time until the preview is written and the viewer has drawn it;
 * `painted` adds the wait for the next frame of the browser, so it never is below one frame (16.7 ms at 60 Hz).
 */
async function sequential(): Promise<Record<string, unknown>> {
  previewTiming.samples = [];
  const ready: number[] = [];
  const total: number[] = [];
  for (let i = 0; i < STEPS; i++) {
    const started = performance.now();
    await step(i);
    await nextTick();
    ready.push(performance.now() - started);
    await painted();
    total.push(performance.now() - started);
  }
  const samples: PreviewSample[] = previewTiming.samples;
  return {
    steps: STEPS,
    ready: summarize(ready),
    painted: summarize(total),
    invoke: summarize(samples.map((s) => s.invoke)),
    write: summarize(samples.map((s) => s.write)),
    draw: summarize(samples.map((s) => s.draw)),
  };
}

/** Steps one after the other without waiting for a frame: how many previews per second the chain can deliver. */
async function throughput(): Promise<number> {
  const started = performance.now();
  let n = 0;
  while (performance.now() - started < THROUGHPUT_MS) await step(n++);
  return Math.round((n / ((performance.now() - started) / 1000)) * 10) / 10;
}

async function drag(): Promise<Record<string, unknown>> {
  previewTiming.samples = [];
  const started = performance.now();
  let events = 0;
  let frames = 0;
  let counting = true;
  // Counts the frames the browser really drew while the slider moved.
  const tick = () => {
    if (!counting) return;
    frames++;
    requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
  while (performance.now() - started < DRAG_MS) {
    settings.wavelet.layers[0].level = level(events++);
    await sleep(DRAG_EVENT_MS);
  }
  const last = performance.now();
  await nextTick();
  await previewIdle();
  await painted();
  counting = false;
  const seconds = (last - started) / 1000;
  return {
    seconds: Math.round(seconds * 10) / 10,
    sliderEvents: events,
    previews: previewTiming.samples.length,
    previewsPerSecond: Math.round((previewTiming.samples.length / seconds) * 10) / 10,
    framesPerSecond: Math.round((frames / seconds) * 10) / 10,
    lastEventToPainted: Math.round(performance.now() - last),
  };
}

/** The same steps for three cases: the Processing Area of 512 pixels, the whole image (2048 reaches over it), and no colour. */
async function cases(): Promise<Record<string, unknown>> {
  const result: Record<string, unknown> = {};
  for (const [name, area, colour] of [
    ["area512Colour", 512, true],
    ["wholeImageColour", 2048, true],
    ["area512Gray", 512, false],
  ] as const) {
    settings.menu.processingArea = area;
    settings.colour = colour;
    // The first preview of a case builds the layers of its area; the steps after it only recombine them.
    await step(1);
    await painted();
    result[name] = { sequential: await sequential(), previewsPerSecondUnthrottled: await throughput() };
  }
  settings.menu.processingArea = 512;
  settings.colour = true;
  await step(1);
  await painted();
  return result;
}

/** Runs the measurement if the environment asks for it (`preview_bench_config`); a normal start returns at once. */
export async function runPreviewBench(): Promise<void> {
  const config = await invoke<{ recording: string } | null>("preview_bench_config");
  if (!config) return;
  let report: Record<string, unknown>;
  try {
    await openRecording([config.recording]);
    const frame = session.frameImage;
    if (!frame) throw new Error("no frame to take as the stacked image");
    await invoke("wavelet_set_source_frame", { index: 0 });
    session.stackImage = frame;
    session.step = "stacked";
    session.view = "wavelets";
    previewTiming.on = true;
    await nextTick();
    await previewIdle();
    await runDoAll();
    const measured = await cases();
    const dragged = await drag();
    report = {
      image: `${frame.width}x${frame.height}`,
      area: settings.menu.processingArea,
      devicePixelRatio: window.devicePixelRatio,
      userAgent: navigator.userAgent,
      ...measured,
      drag: dragged,
    };
  } catch (e) {
    report = { error: String(e) };
  }
  await invoke("preview_bench_report", { report: JSON.stringify(report, null, 2) });
}
