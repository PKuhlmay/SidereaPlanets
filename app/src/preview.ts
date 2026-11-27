// Dev builds only (main.ts): jumps into a state of the window with generated data, so every state
// of the design can be looked at without a recording and without the computing steps.
//
//   ?step=empty|loaded|points|aligned|limited|stacked|processed|saved
//   ?view=open|alignpoints|align|limit|stack|wavelets|save  (without it the next open step)  ?advanced=1
//   ?theme=light  ?busy=40  ?show=framelist,graph,resampling,drizzling,procarea,alignpoints,nosections
//   ?layers=32,22,14,10,10,10  ?functions=Histogram,Gamma  (cards open in the Wavelets step; their values come from
//   the engine, so the cards need the app, not the plain browser)
//   ?update=notes|nonotes|deb|error  (the dialog of a new version, also without any step)

import { setFrameReader, showFrame, startSession } from "./recording";
import { LAYER_COUNT } from "./logic/settings";
import { atLeast, viewSteps, type Step, type ViewStep } from "./logic/workflow";
import { applyTheme } from "./theme";
import { session, settings, status, type Alignpoint, type RecordingInfo } from "./state";
import { advance, advancedOpen } from "./steps";
import { updates } from "./updater";
import { markDoAllCurrent } from "./wavelet";

const W = 1024;
const H = 680;
const QUALITY = [92.1, 88.4, 97.3, 85.0, 90.6, 99.2, 83.7, 95.8, 87.2, 100.0];
const DIFFERENCE = [31, 44, 18, 52, 37, 12, 58, 22, 47, 9];
// The states of the design (docs/design.md, "Zustände in der App ansehen"): the steps of the calculation plus
// "processed" (Do All ran with the values on screen) and "saved" (Save image wrote the result).
type PreviewStep = Step | "processed" | "saved";
const STEPS: PreviewStep[] = ["empty", "loaded", "points", "aligned", "limited", "stacked", "processed", "saved"];
// Without `step` a `view` gets the first state in which it can be opened.
const NEEDS: Record<ViewStep, Step> = {
  open: "loaded",
  alignpoints: "loaded",
  align: "points",
  limit: "aligned",
  stack: "limited",
  wavelets: "stacked",
  save: "stacked",
};

const info: RecordingInfo = {
  format: "AVI",
  codec: "DIB",
  width: W,
  height: H,
  bitDepth: 8,
  frameCount: QUALITY.length,
  color: "Rgb",
  fps: 25,
  observer: null,
  instrument: null,
  telescope: null,
  hasTimestamps: false,
  startTime: null,
  startTimeFromFileDate: false,
};

// A banded disc on black, a little blurrier for frames of lower quality.
function drawPlanet(index: number): Promise<ImageData> {
  const canvas = document.createElement("canvas");
  canvas.width = W;
  canvas.height = H;
  const ctx = canvas.getContext("2d");
  if (!ctx) return Promise.reject(new Error("no 2d context"));
  ctx.fillStyle = "#000";
  ctx.fillRect(0, 0, W, H);
  ctx.filter = `blur(${(0.4 + (100 - QUALITY[index % QUALITY.length]) * 0.09).toFixed(2)}px)`;
  ctx.save();
  ctx.translate(((index * 7) % 11) - 5, ((index * 5) % 9) - 4);
  ctx.beginPath();
  ctx.ellipse(540, 345, 236, 222, 0, 0, Math.PI * 2);
  ctx.clip();
  const bands = ctx.createLinearGradient(0, 123, 0, 567);
  [["#b39aae", 0], ["#cbb5c7", 0.2], ["#8b5a61", 0.29], ["#d9c7d6", 0.43], ["#875459", 0.58], ["#ccb6c6", 0.73], ["#8a7484", 1]].forEach(
    ([color, at]) => bands.addColorStop(at as number, color as string),
  );
  ctx.fillStyle = bands;
  ctx.fillRect(0, 0, W, H);
  const limb = ctx.createRadialGradient(540, 345, 0, 540, 345, 236);
  limb.addColorStop(0.7, "rgba(0,0,0,0)");
  limb.addColorStop(1, "rgba(0,0,0,0.85)");
  ctx.fillStyle = limb;
  ctx.fillRect(0, 0, W, H);
  ctx.restore();
  return Promise.resolve(ctx.getImageData(0, 0, W, H));
}

// 91 points on a hexagonal grid over the disc, the weakest dropped by the default threshold (9) leaves 82.
function makePoints(): Alignpoint[] {
  const points: Alignpoint[] = [];
  for (let y = 137, row = 0; y <= 553; y += 38, row++) {
    for (let x = 318 + (row % 2 ? 19 : 0); x <= 762; x += 38) {
      if (((x - 540) / 222) ** 2 + ((y - 345) / 208) ** 2 <= 1) {
        const s = Math.abs(Math.sin(x * 12.9898 + y * 78.233) * 43758.5453) % 1;
        points.push({ x, y, shiftX: (s - 0.5) * 3.5, shiftY: (0.5 - s) * 2.5, checked: true });
      }
    }
  }
  return points.slice(0, 91);
}

/** The dialog of a new version (PLANETS-13) with made-up release notes: `notes`, `nonotes`, `deb` (cannot update itself), `error`. */
function previewUpdate(kind: string): void {
  const notes = [
    "Siderea Planets 0.2.0",
    "",
    "- Check for Updates… in the app menu and Settings.",
    "- Better quality estimate for frames with a very small planet.",
    "- Fixed a crash when saving a project to a folder without write access.",
    "",
    "Known issues",
    "- Resize Image does not remember its last value.",
    "- Drizzle is still locked.",
    "- The Linux .deb does not update itself.",
  ].join("\n");
  updates.state.update = { version: "0.2.0", notes: kind === "nonotes" ? "" : notes, canInstall: kind !== "deb" };
  if (kind === "error") updates.state.error = "Could not install the update: the download was interrupted";
}

export async function applyPreview(search: string) {
  const params = new URLSearchParams(search);
  const view = viewSteps.find((s) => s.id === params.get("view"))?.id ?? null;
  const named = STEPS.find((s) => s === params.get("step"));
  if (params.get("theme") === "light") applyTheme("light");
  const update = params.get("update");
  if (update) previewUpdate(update);
  if (!named && !view) return;
  const state: PreviewStep = named ?? (view ? NEEDS[view] : "loaded");
  const step: Step = state === "processed" || state === "saved" ? "stacked" : state;

  if (params.get("advanced") === "1") viewSteps.forEach((s) => (advancedOpen[s.id] = true));
  if (step === "empty") return;
  setFrameReader(drawPlanet);
  startSession(info, "JUPITER.AVI");
  await showFrame(0);

  session.step = step;
  if (atLeast(step, "points")) {
    session.alignpoints = makePoints().slice(0, 82);
    session.alignpointTotal = 91;
    settings.align.threshold = 9;
  }
  if (atLeast(step, "aligned")) {
    session.frames.forEach((f, i) => {
      f.quality = QUALITY[i];
      f.difference = DIFFERENCE[i];
    });
  }
  if (atLeast(step, "stacked")) {
    session.stackImage = await drawPlanet(9);
    session.stackSize = 10;
  }

  const result: Record<Step, [string, string]> = {
    empty: ["", ""],
    loaded: ["", "Loaded 10 frames"],
    points: ["", "82 alignpoints set"],
    aligned: ["Aligning frame: 10", "Alignment finished"],
    limited: ["Keeping 10 of 10 frames", "10 frames will be stacked · weakest kept: 100%"],
    stacked: ["Stacked 10 frames · 1024×680", "Value range 0–255"],
  };
  [status.action, status.result] = result[step];
  status.progress = atLeast(step, "aligned") ? 1 : 0;

  const show = (params.get("show") ?? "").split(",");
  settings.align.showFramelist = show.includes("framelist");
  settings.align.showRegistrationgraph = show.includes("graph");
  settings.stack.resampling = show.includes("resampling");
  settings.stack.drizzling = show.includes("drizzling");
  settings.wavelet.showProcessingArea = show.includes("procarea");
  settings.wavelet.showAlignpoints = show.includes("alignpoints");
  if (show.includes("nosections")) settings.stack.showAlignSections = false;

  (params.get("layers") ?? "").split(",").slice(0, LAYER_COUNT).forEach((value, i) => {
    if (value !== "") settings.wavelet.layers[i].level = Number(value);
  });
  settings.wavelet.functions = (params.get("functions") ?? "").split(",").filter((name) => name !== "");

  // Do All and Save image after the layers are set: the result belongs to these values.
  if (state === "processed" || state === "saved") {
    session.processedImage = await drawPlanet(9);
    markDoAllCurrent();
  }
  if (state === "saved") {
    session.saved = true;
    session.savedName = "JUPITER.tif";
    status.result = "Saved JUPITER.tif";
  }
  if (view) session.view = view;
  else advance();
  const busy = Number(params.get("busy"));
  if (busy > 0) {
    status.busy = { name: "Align", paused: false, cancelled: false };
    status.progress = busy / 100;
    status.action = `Aligning frame: ${Math.ceil(busy / 10)}`;
    status.result = "";
    status.timeUsed = 4;
    status.timeLeft = 6;
  }
}
