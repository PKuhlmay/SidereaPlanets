// The Stack button: stacks the checked frames in the backend and shows the result in the Stack step.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { clearCrop } from "./cropTool";
import { runJob } from "./jobs";
import { buildStackRequest, stackMessages } from "./logic/stack";
import { session, settings } from "./state";
import { advance } from "./steps";

interface Stacked {
  width: number;
  height: number;
  channels: number;
  frames: number;
  elapsedMs: number;
}

interface StackProgress {
  done: number;
  total: number;
}

// Same binary layout as a frame: width (u32 LE), height (u32 LE), then RGBA bytes.
async function fetchStackImage(): Promise<ImageData> {
  const buffer = await invoke<ArrayBuffer>("stacked_image");
  const view = new DataView(buffer);
  const width = view.getUint32(0, true);
  const height = view.getUint32(4, true);
  return new ImageData(new Uint8ClampedArray(buffer, 8, width * height * 4), width, height);
}

export async function stack(): Promise<void> {
  const request = buildStackRequest(session, settings);
  await runJob("Stack", async (job) => {
    job.progress(0, request.frames.length, "Stacking frame: 0");
    const unlisten = await listen<StackProgress>("stack-progress", ({ payload }) =>
      job.progress(payload.done, payload.total, `Stacking frame: ${payload.done}`),
    );
    let stacked: Stacked;
    try {
      stacked = await invoke<Stacked>("stack", { settings: request });
    } finally {
      unlisten();
    }
    session.stackImage = await fetchStackImage();
    // A crop rectangle belongs to the stack it was set on.
    clearCrop();
    session.stackSize = stacked.frames;
    // A new stack makes an earlier sharpened image obsolete.
    session.processedImage = null;
    session.step = "stacked";
    advance();
    const messages = stackMessages(stacked.frames, stacked.width, stacked.height);
    job.progress(1, 1, messages.action);
    return messages.result;
  });
}
