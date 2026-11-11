// Long calculations: progress, time, Cancel and Pause in one place. A step of a later ticket calls
// `runJob("Align", async (job) => { ... job.progress(done, total, "Loading frame: 4") ... return "Alignment finished." })`.

import { invoke } from "@tauri-apps/api/core";
import { timeLeft } from "./logic/workflow";
import { status } from "./state";

export interface JobContext {
  /** Progress of the running step; `action` replaces the text of the second status field. */
  progress(done: number, total: number, action?: string): void;
  /** True after Cancel; a job that polls this stops itself. */
  readonly cancelled: boolean;
}

// Align, Stack and the Wavelet tab run in different commands of the backend, each with its own flags for Cancel and
// Pause; Save Aligned and Save Integrated share the flags of Stack. "Wavelet" is Do All and Process.
const commands = (name: string) => {
  if (name === "Stack" || name === "Save Aligned" || name === "Save Integrated") {
    return { cancel: "cancel_stacking", pause: "pause_stacking" };
  }
  if (name === "Wavelet") return { cancel: "cancel_wavelet", pause: "pause_wavelet" };
  return { cancel: "cancel_processing", pause: "pause_processing" };
};

let timer: number | undefined;
let startedAt = 0;
let pausedAt: number | null = null;
let pausedTotal = 0;

function tick() {
  const now = performance.now();
  const paused = pausedTotal + (pausedAt === null ? 0 : now - pausedAt);
  status.timeUsed = (now - startedAt - paused) / 1000;
  status.timeLeft = timeLeft(status.timeUsed, status.progress);
}

/** Runs one calculation. Only one at a time; the returned text is the result shown in the status bar. */
export async function runJob(name: string, job: (context: JobContext) => Promise<string | void>): Promise<void> {
  if (status.busy) return;
  const busy = { name, paused: false, cancelled: false };
  status.busy = busy;
  status.progress = 0;
  status.action = "";
  status.result = "";
  startedAt = performance.now();
  pausedAt = null;
  pausedTotal = 0;
  tick();
  timer = window.setInterval(tick, 250);
  try {
    const result = await job({
      progress(done, total, action) {
        status.progress = total > 0 ? Math.min(1, done / total) : 0;
        if (action !== undefined) status.action = action;
      },
      get cancelled() {
        return busy.cancelled;
      },
    });
    if (busy.cancelled) {
      status.result = "Cancelled";
    } else {
      status.progress = 1;
      status.result = result ?? "";
    }
  } catch (e) {
    status.result = busy.cancelled ? "Cancelled" : `Error: ${String(e)}`;
  } finally {
    window.clearInterval(timer);
    tick();
    status.busy = null;
  }
}

/** Cancel in the toolbar. The backend commands stop the calculation that runs in Rust. */
export async function cancelJob() {
  const busy = status.busy;
  if (!busy) return;
  busy.cancelled = true;
  if (busy.paused) resume(busy);
  await invoke(commands(busy.name).cancel).catch(() => undefined);
}

function resume(busy: { paused: boolean }) {
  busy.paused = false;
  if (pausedAt !== null) pausedTotal += performance.now() - pausedAt;
  pausedAt = null;
}

/** Pause in the toolbar; the button reads Resume while paused. */
export async function togglePause() {
  const busy = status.busy;
  if (!busy) return;
  if (busy.paused) {
    resume(busy);
  } else {
    busy.paused = true;
    pausedAt = performance.now();
  }
  await invoke(commands(busy.name).pause, { paused: busy.paused }).catch(() => undefined);
}

