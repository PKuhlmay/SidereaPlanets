// The Alignpoints, Align and Limit steps: Set Alignpoints, Align and Limit call the backend commands (app/src-tauri/src/align_commands.rs)
// and write their results into `session`. Imported once by main.ts, which registers the handlers.
//
// Not available yet, because the reference program was never observed doing them (docs/offene-fragen.md, question 21):
// No Align, Scan Frames and Empty List have no handler; the status bar says so.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { handlers } from "./actions";
import { failureText, runJob } from "./jobs";
import {
  alignArgs,
  limitArgs,
  limitMessages,
  qualityPercent,
  roomForPoints,
  setAlignpointsArgs,
  type AlignedAnswer,
  type AlignpointsAnswer,
  type AlignProgress,
  type LimitedAnswer,
} from "./logic/align";
import { atLeast } from "./logic/workflow";
import { session, settings, status } from "./state";
import { advance } from "./steps";

function showPoints(answer: AlignpointsAnswer) {
  session.alignpoints = answer.points.map((p) => ({ x: p.x, y: p.y, checked: true }));
  session.alignpointTotal = answer.total;
  settings.align.threshold = answer.threshold;
}

/**
 * A new Set Alignpoints or Align replaces the alignment, so everything built on the old one goes: Limit (frame choice
 * and point shifts), Stack, the sharpened image and the colour adjustment. The backend drops its copies at the same
 * time (`discard_results` in lib.rs). The steps after it are locked again; a step panel showing one of them moves to the
 * next open step (steps.ts).
 */
function discardResults() {
  session.frames.forEach((f) => Object.assign(f, { checked: true, quality: null, difference: null }));
  session.alignpoints.forEach((p) => Object.assign(p, { shiftX: undefined, shiftY: undefined }));
  session.stackImage = null;
  session.processedImage = null;
  session.adjustedImage = null;
  session.stackSize = null;
  if (atLeast(session.step, "aligned")) session.step = "points";
}

async function setAlignpoints() {
  const roi = session.roi;
  if (roi && !roomForPoints(roi, settings.align.edgeDistance)) {
    status.result = "The region is too small for the distance from the edge; draw a larger region";
    return;
  }
  // The frame is read now: the Frame bar may move on while the points are searched, but they belong to this one.
  const frame = session.frame;
  await runJob("Set Alignpoints", async () => {
    const answer = await invoke<AlignpointsAnswer>("set_alignpoints", setAlignpointsArgs(frame, settings.align, roi));
    showPoints(answer);
    // Align and Stack register the other frames against this one (the backend keeps the same number).
    session.referenceFrame = frame;
    // Points that are set again start a new alignment.
    discardResults();
    if (answer.points.length === 0) {
      session.step = "loaded";
      return "No alignpoints found: lower the distances or widen Min./Max. brightness under Advanced";
    }
    session.step = "points";
    advance();
    return `${answer.points.length} alignpoints set`;
  });
}

/** The Threshold slider: the weakest points are dropped. */
export async function setThreshold(threshold: number) {
  settings.align.threshold = threshold;
  try {
    showPoints(await invoke<AlignpointsAnswer>("set_alignpoint_threshold", { threshold }));
  } catch (e) {
    status.result = failureText("Threshold", e);
  }
}

async function align() {
  await runJob("Align", async (job) => {
    // The backend drops its results when Align starts, so the window does the same.
    discardResults();
    const stop = await listen<AlignProgress>("align-progress", (event) =>
      job.progress(event.payload.done, event.payload.total, `Aligning frame: ${event.payload.done}`),
    );
    let answer: AlignedAnswer;
    try {
      answer = await invoke<AlignedAnswer>("align", alignArgs(settings.align, session.roi));
    } finally {
      stop();
    }
    const percent = qualityPercent(answer.frames.map((f) => f.quality));
    for (const frame of answer.frames) {
      const entry = session.frames[frame.index];
      if (entry) Object.assign(entry, { quality: percent[frame.index], difference: frame.difference });
    }
    session.step = "aligned";
    advance();
    return "Alignment finished";
  });
}

async function limit() {
  await runJob("Keep Best Frames", async () => {
    const answer = await invoke<LimitedAnswer>("limit", limitArgs(settings.align));
    const kept = new Set(answer.frames);
    session.frames.forEach((f) => (f.checked = kept.has(f.index)));
    answer.pointShifts.forEach((shift, i) => {
      const point = session.alignpoints[i];
      if (point) Object.assign(point, { shiftX: shift?.[0], shiftY: shift?.[1] });
    });
    // Limiting again after Stack makes that stack out of date.
    if (atLeast(session.step, "stacked")) {
      session.stackImage = null;
      session.processedImage = null;
      session.adjustedImage = null;
      session.stackSize = null;
    }
    session.step = "limited";
    advance();
    const qualities = answer.frames.flatMap((index) => session.frames[index]?.quality ?? []);
    const messages = limitMessages(answer.frames.length, session.frames.length, qualities);
    status.action = messages.action;
    return messages.result;
  });
}

/** R.o.Interest: the next drag in the image draws the region; pressed again, the region is dropped. */
function regionOfInterest() {
  if (session.roiDrawing) {
    session.roiDrawing = false;
    session.roi = null;
    settings.align.showRoi = false;
  } else {
    session.roiDrawing = true;
  }
}

handlers.setAlignpoints = setAlignpoints;
handlers.align = align;
handlers.limit = limit;
handlers.regionOfInterest = regionOfInterest;
