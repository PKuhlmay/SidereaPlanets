// Dev builds only (main.ts): jumps into a state of the window with generated data, so every state of
// docs/design/screenshots can be looked at without a recording and without the computing steps.
//
//   ?step=loaded|points|aligned|limited|stacked  ?tab=align|stack|wavelet  ?theme=light  ?busy=40
//   ?show=framelist,graph,resampling,drizzling,procarea,alignpoints,nosections  ?layers=32,22,14,10,10,10
//   ?functions=Histogram,Gamma  (cards open in the Wavelet tab; their values come from the engine, so the cards
//   need the app, not the plain browser)

import { setFrameReader, showFrame, startSession } from "./recording";
import { LAYER_COUNT } from "./logic/settings";
import { atLeast, type Step, type Tab } from "./logic/workflow";
import { applyTheme } from "./theme";
import { session, settings, status, type Alignpoint, type RecordingInfo } from "./state";

const W = 1024;
const H = 680;
const QUALITY = [92.1, 88.4, 97.3, 85.0, 90.6, 99.2, 83.7, 95.8, 87.2, 100.0];
const DIFFERENCE = [31, 44, 18, 52, 37, 12, 58, 22, 47, 9];
const STEPS: Step[] = ["loaded", "points", "aligned", "limited", "stacked"];
const TABS: Tab[] = ["align", "stack", "wavelet"];

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

export async function applyPreview(search: string) {
  const params = new URLSearchParams(search);
  const stepName = params.get("step") as Step | null;
  const tab = TABS.find((t) => t === params.get("tab")) ?? "align";
  const step: Step = STEPS.find((s) => s === stepName) ?? (tab === "align" ? "loaded" : "stacked");
  if (!params.has("step") && !params.has("tab")) return;

  if (params.get("theme") === "light") applyTheme("light");
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
  session.tab = tab;

  const result: Record<Step, [string, string]> = {
    empty: ["", ""],
    loaded: ["", "Loaded 10 frames"],
    points: ["", "82 alignpoints set"],
    aligned: ["Loading frame: 10", "Alignment finished."],
    limited: ["Processing limited to 10 frames", "Stacksize: 10  Low quality: 100.00%"],
    stacked: ["Stack size (n=10) - (1024x680)", "Stacking Range : 0-255"],
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
  const busy = Number(params.get("busy"));
  if (busy > 0) {
    status.busy = { name: "Align", paused: false, cancelled: false };
    status.progress = busy / 100;
    status.action = `Loading frame: ${Math.ceil(busy / 10)}`;
    status.result = "";
    status.timeUsed = 4;
    status.timeLeft = 6;
  }
}
