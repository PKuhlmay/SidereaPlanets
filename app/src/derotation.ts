// De-rotate images in the Stack step (SP-023): asks the core for the span, the rotation and the disc measured in the reference
// frame whenever something they depend on changes, after a short pause so that typing in a field does not measure per key.
// The calculation is `Session::derotation_info`; stacking itself sends the settings with Stack (logic/stack.ts).

import { invoke } from "@tauri-apps/api/core";
import { watch } from "vue";
import { failureText } from "./jobs";
import { derotationRequest, type DerotationInfo } from "./logic/derotation";
import { atLeast } from "./logic/workflow";
import { showFrame } from "./recording";
import { session, settings, status } from "./state";

const DELAY_MS = 120;

let timer: ReturnType<typeof setTimeout> | undefined;
// Answers to an older request are dropped.
let generation = 0;

function refresh(): void {
  clearTimeout(timer);
  const request = derotationRequest(settings.stack);
  const current = ++generation;
  if (!request || !atLeast(session.step, "limited")) {
    session.derotation = null;
    return;
  }
  const referenceFrame = session.referenceFrame;
  const frames = session.frames.filter((f) => f.checked).map((f) => f.index);
  timer = setTimeout(async () => {
    try {
      const info = await invoke<DerotationInfo>("derotation_info", { settings: request, referenceFrame, frames });
      if (current !== generation) return;
      session.derotation = info;
      // The disc was measured in the reference frame: the Stack step shows that frame until the stack is there.
      if (info.disc && session.view === "stack" && session.step !== "stacked" && session.frame !== referenceFrame) void showFrame(referenceFrame);
    } catch (e) {
      if (current !== generation) return;
      session.derotation = null;
      status.result = failureText("De-rotation", e);
    }
  }, DELAY_MS);
}

let started = false;

/** Watches what De-rotate images depends on: its settings, the step, the reference frame and the kept frames. */
export function startDerotation(): void {
  if (started) return;
  started = true;
  watch(
    () => [
      settings.stack.derotate,
      settings.stack.planet,
      settings.stack.cameraAngle,
      settings.stack.mirrored,
      settings.stack.startTime,
      session.info,
      session.step,
      session.referenceFrame,
      session.frames.map((f) => (f.checked ? f.index : -1)).join(","),
    ],
    refresh,
  );
}
