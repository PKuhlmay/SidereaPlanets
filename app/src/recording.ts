// Opening a recording and reading its frames through the Tauri commands of the backend.

import { reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { clearCrop } from "./cropTool";
import { failureText } from "./jobs";
import { planetFromName } from "./logic/derotation";
import { pushRecent, fileName } from "./logic/recent";
import { session, settings, status, type RecordingInfo } from "./state";

// Same filters and order as the "Open file(s)" dialog of the reference program (docs/formats.md).
const fileFilters = [
  { name: "Video (*.avi *.ser)", extensions: ["avi", "ser"] },
  { name: "Still frames (*.bmp *.jpg *.tif *.png)", extensions: ["bmp", "jpg", "jpeg", "tif", "tiff", "png"] },
  { name: "FITS frames (*.fit *.fits *.fts)", extensions: ["fit", "fits", "fts"] },
  { name: "All files", extensions: ["*"] },
];

const RECENT_KEY = "rk-recent";

function loadRecent(): string[][] {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(RECENT_KEY) ?? "[]");
    return Array.isArray(parsed) ? parsed.filter((e): e is string[] => Array.isArray(e) && e.every((p) => typeof p === "string")) : [];
  } catch {
    return [];
  }
}

/** The files opened last, newest first (menu MRU). */
export const recent = reactive({ list: loadRecent() });

// Where frames come from. The default asks the backend; the design preview in dev builds swaps it for a generated image.
let readFrameImage = async (index: number): Promise<ImageData> => {
  // Binary layout: width (u32 LE), height (u32 LE), then RGBA bytes.
  const buffer = await invoke<ArrayBuffer>("read_frame", { index });
  const view = new DataView(buffer);
  const width = view.getUint32(0, true);
  const height = view.getUint32(4, true);
  return new ImageData(new Uint8ClampedArray(buffer, 8, width * height * 4), width, height);
};

export function setFrameReader(reader: (index: number) => Promise<ImageData>) {
  readFrameImage = reader;
}

// Drops answers that arrive after a newer frame was requested.
let latestRequest = 0;

/** The frame before or after the one shown (frame bar, ← and →); stays at the first and the last frame. */
export function showNextFrame(by: number): void {
  const last = Math.max(0, (session.info?.frameCount ?? 0) - 1);
  const next = Math.min(Math.max(session.frame + by, 0), last);
  if (next !== session.frame) void showFrame(next);
}

export async function showFrame(index: number) {
  const request = ++latestRequest;
  session.frame = index;
  try {
    const image = await readFrameImage(index);
    if (request === latestRequest) session.frameImage = image;
  } catch (e) {
    if (request === latestRequest) status.result = failureText("Showing the frame", e);
  }
}

/** Select: asks for files and opens them. */
export async function chooseFiles() {
  // Several still frames together form one sequence, so multiple selection is allowed.
  const picked = await openDialog({ multiple: true, directory: false, filters: fileFilters });
  if (picked && picked.length > 0) await openRecording(picked);
}

export async function openRecording(paths: string[]) {
  let info: RecordingInfo;
  try {
    info = await invoke<RecordingInfo>("open_recording", { paths });
  } catch (e) {
    status.result = failureText("Open", e);
    return;
  }
  startSession(info, fileName(paths[0] ?? ""));
  session.paths = [...paths];
  recent.list = pushRecent(recent.list, paths);
  try {
    localStorage.setItem(RECENT_KEY, JSON.stringify(recent.list));
  } catch {
    // The list then lasts for this run only.
  }
  await showFrame(0);
  status.result = `Loaded ${info.frameCount} frames`;
}

/** Puts the window into the state "recording loaded". */
export function startSession(info: RecordingInfo, name: string) {
  session.info = info;
  session.fileName = name;
  session.paths = [];
  session.step = "loaded";
  // After Select the next open step is Alignpoints.
  session.view = "alignpoints";
  session.saved = false;
  session.savedName = "";
  session.frame = 0;
  session.frameImage = null;
  session.stackImage = null;
  clearCrop();
  session.processedImage = null;
  session.adjustedImage = null;
  session.clipboardImage = null;
  session.differenceImage = null;
  session.toneView = "current";
  session.stackSize = null;
  session.alignpoints = [];
  session.referenceFrame = 0;
  session.alignpointTotal = 0;
  session.roi = null;
  session.bestFrameHint = null;
  session.maskImage = null;
  session.roiDrawing = false;
  session.derotation = null;
  // De-rotate images: the start time of this file (or none) and the planet its name gives; the check box stays as it is.
  settings.stack.startTime = info.startTime;
  settings.stack.planet = planetFromName(name) ?? settings.stack.planet;
  session.frames = Array.from({ length: info.frameCount }, (_, index) => ({ index, checked: true, quality: null, difference: null }));
  settings.align.threshold = 0;
  // The original asks "process in colour?"; the clone decides from the recording (Settings > Autodetect Colour/BW).
  settings.colour = info.color !== "Mono";
  status.progress = 0;
  status.action = "";
  status.position = "";
}

/**
 * Files dropped onto the window open like the files picked with Select Recording… (several still frames form one
 * sequence). Nothing opens during a calculation. Outside the Tauri window (browser preview) there is nothing to listen to.
 */
export function openDroppedFiles(): void {
  try {
    void getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === "drop" && event.payload.paths.length > 0 && status.busy === null) void openRecording(event.payload.paths);
      })
      .catch(() => undefined);
  } catch {
    // Not running inside Tauri.
  }
}
