// The window and the backend together: the handlers of the step buttons run against a fake backend that answers
// with the shapes of app/src-tauri/src/*_commands.rs and checks what it is sent. The pure logic is tested in
// logic/*.test.ts; this file is about the seams between the steps (which frame the points belong to, the state
// after Cancel or an error, what a new stack makes obsolete).

import { beforeEach, describe, expect, it, vi } from "vitest";

type Args = Record<string, unknown> | undefined;

const backend = vi.hoisted(() => ({
  calls: [] as { command: string; args: Args }[],
  handlers: {} as Record<string, (args: Args) => unknown>,
  listeners: {} as Record<string, (event: { payload: unknown }) => void>,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: async (command: string, args?: Args) => {
    backend.calls.push({ command, args });
    const handler = backend.handlers[command];
    if (!handler) throw new Error(`unexpected command ${command}`);
    return handler(args);
  },
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: async (name: string, callback: (event: { payload: unknown }) => void) => {
    backend.listeners[name] = callback;
    return () => {
      delete backend.listeners[name];
    };
  },
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(), save: vi.fn() }));

// The tests run in node: the few browser objects the code touches are replaced.
class FakeImageData {
  constructor(
    readonly data: Uint8ClampedArray,
    readonly width: number,
    readonly height: number,
  ) {}
}
vi.stubGlobal("ImageData", FakeImageData);
vi.stubGlobal("window", { setInterval, clearInterval, setTimeout, clearTimeout });

import "./align";
import { runAction } from "./actions";
import { autoAlign } from "./colour";
import { cancelJob, togglePause } from "./jobs";
import { process } from "./wavelet";
import { openRecording, startSession } from "./recording";
import { session, settings, status, type RecordingInfo } from "./state";

const W = 4;
const H = 3;
const info: RecordingInfo = {
  format: "AVI",
  codec: "DIB",
  width: W,
  height: H,
  bitDepth: 8,
  frameCount: 4,
  color: "Rgb",
  fps: null,
  observer: null,
  instrument: null,
  telescope: null,
};

/** Width, height (u32 LE), then RGBA: what read_frame, stacked_image and wavelet_do_all return. */
function imageBuffer(): ArrayBuffer {
  const buffer = new ArrayBuffer(8 + W * H * 4);
  const view = new DataView(buffer);
  view.setUint32(0, W, true);
  view.setUint32(4, H, true);
  new Uint8Array(buffer, 8).fill(200);
  return buffer;
}

const calls = (command: string) => backend.calls.filter((c) => c.command === command);

/** A request that stays open until the test settles it, as a long calculation does. */
function pending() {
  let resolve: (value: unknown) => void = () => undefined;
  let reject: (reason: unknown) => void = () => undefined;
  const promise = new Promise((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

// What the fake backend remembers, like AlignState in align_commands.rs.
let referenceFrame = -1;

function installBackend() {
  backend.handlers = {
    open_recording: () => info,
    read_frame: () => imageBuffer(),
    set_alignpoints: (args) => {
      referenceFrame = (args as { frame: number }).frame;
      return { points: [{ x: 10, y: 8 }, { x: 20, y: 16 }], total: 3, threshold: 1 };
    },
    align: () => ({
      frames: [0, 1, 2, 3].map((index) => ({ index, quality: 10 + index, shiftX: 0, shiftY: 0, difference: index })),
      ranking: [3, 2, 1, 0],
    }),
    limit: () => ({ frames: [3, 2, 1], pointShifts: [[0.5, -0.5], null] }),
    stack: (args) => {
      const { settings: request } = args as { settings: { referenceFrame: number; frames: number[] } };
      // The backend registers against the frame the points were found on; anything else gives a distorted stack.
      if (request.referenceFrame !== referenceFrame) {
        throw new Error(`stack: reference frame ${request.referenceFrame}, the points belong to frame ${referenceFrame}`);
      }
      return { width: W, height: H, channels: 3, frames: request.frames.length, elapsedMs: 1 };
    },
    stacked_image: () => imageBuffer(),
    wavelet_do_all: () => imageBuffer(),
    cancel_processing: () => undefined,
    cancel_stacking: () => undefined,
    cancel_wavelet: () => undefined,
    pause_wavelet: () => undefined,
  };
}

beforeEach(() => {
  backend.calls = [];
  backend.listeners = {};
  referenceFrame = -1;
  installBackend();
  startSession(info, "JUPITER.AVI");
  status.busy = null;
  status.result = "";
});

describe("Select", () => {
  it("opens the recording and shows its first frame", async () => {
    await openRecording(["C:\\data\\JUPITER.AVI"]);
    expect(calls("open_recording")[0]?.args).toEqual({ paths: ["C:\\data\\JUPITER.AVI"] });
    expect(session.step).toBe("loaded");
    expect(session.frames).toHaveLength(4);
    expect(session.frameImage).toMatchObject({ width: W, height: H });
    expect(status.result).toBe("Loaded 4 frames");
  });

  it("reports a file that cannot be opened", async () => {
    backend.handlers.open_recording = () => {
      throw new Error("not a recording");
    };
    await openRecording(["C:\\data\\x.txt"]);
    expect(status.result).toBe("Error: Error: not a recording");
    expect(session.step).toBe("loaded");
  });
});

describe("Set Alignpoints, Align, Limit, Stack", () => {
  it("runs the whole Align and Stack chain and moves through the steps", async () => {
    session.frame = 2;
    await runAction("setAlignpoints");
    expect(session.step).toBe("points");
    expect(session.alignpoints).toHaveLength(2);
    expect(session.alignpointTotal).toBe(3);
    expect(status.result).toBe("2 alignpoints set");

    await runAction("align");
    expect(session.step).toBe("aligned");
    // Quality is shown in percent of the best frame.
    expect(session.frames.map((f) => Math.round(f.quality ?? -1))).toEqual([77, 85, 92, 100]);

    await runAction("limit");
    expect(session.step).toBe("limited");
    expect(session.tab).toBe("stack");
    expect(session.frames.map((f) => f.checked)).toEqual([false, true, true, true]);
    expect(session.alignpoints[0]).toMatchObject({ shiftX: 0.5, shiftY: -0.5 });
    expect(session.alignpoints[1]?.shiftX).toBeUndefined();
    expect(status.action).toBe("Processing limited to 3 frames");

    await runAction("stack");
    expect(status.result).toBe("Stacking Range : 0-255");
    expect(session.step).toBe("stacked");
    expect(session.stackSize).toBe(3);
    expect(session.stackImage).toMatchObject({ width: W, height: H });
    expect(calls("stack")[0]?.args).toMatchObject({ settings: { referenceFrame: 2, frames: [1, 2, 3] } });
  });

  it("registers the stack against the frame the points were set on, not against frame 1", async () => {
    session.frame = 3;
    await runAction("setAlignpoints");
    expect(session.referenceFrame).toBe(3);
    // Setting the points again on another frame moves the reference with them.
    session.frame = 1;
    await runAction("setAlignpoints");
    expect(session.referenceFrame).toBe(1);
  });

  it("shows the progress events of Align in the status bar", async () => {
    await runAction("setAlignpoints");
    const seen: { progress: number; action: string }[] = [];
    backend.handlers.align = () => {
      backend.listeners["align-progress"]?.({ payload: { done: 2, total: 4, elapsedMs: 5 } });
      seen.push({ progress: status.progress, action: status.action });
      return { frames: [], ranking: [] };
    };
    await runAction("align");
    expect(seen).toEqual([{ progress: 0.5, action: "Loading frame: 2" }]);
    expect(backend.listeners["align-progress"]).toBeUndefined();
  });

  it("stays at the step before when Align fails, and says why", async () => {
    await runAction("setAlignpoints");
    backend.handlers.align = () => {
      throw new Error("set alignpoints first");
    };
    await runAction("align");
    expect(status.result).toBe("Error: Error: set alignpoints first");
    expect(session.step).toBe("points");
    expect(status.busy).toBeNull();
    expect(backend.listeners["align-progress"]).toBeUndefined();
  });

  it("Cancel stops Align, keeps the step and tells the backend", async () => {
    await runAction("setAlignpoints");
    const running = pending();
    backend.handlers.align = () => running.promise;
    const job = runAction("align");
    await vi.waitFor(() => expect(calls("align")).toHaveLength(1));
    expect(status.busy?.name).toBe("Align");

    const cancelling = cancelJob();
    running.reject("cancelled");
    await Promise.all([job, cancelling]);

    expect(calls("cancel_processing")).toHaveLength(1);
    expect(status.result).toBe("Cancelled");
    expect(session.step).toBe("points");
    expect(status.busy).toBeNull();
  });

  it("Cancel stops Stack through the stacking flag, not the align flag", async () => {
    await runAction("setAlignpoints");
    await runAction("align");
    await runAction("limit");
    const running = pending();
    backend.handlers.stack = () => running.promise;
    const job = runAction("stack");
    await vi.waitFor(() => expect(calls("stack")).toHaveLength(1));

    const cancelling = cancelJob();
    running.reject("cancelled");
    await Promise.all([job, cancelling]);

    expect(calls("cancel_stacking")).toHaveLength(1);
    expect(calls("cancel_processing")).toHaveLength(0);
    expect(status.result).toBe("Cancelled");
    expect(session.step).toBe("limited");
    expect(session.stackImage).toBeNull();
  });

  it("limiting again after Stack drops the stack and the sharpened image", async () => {
    await runAction("setAlignpoints");
    await runAction("align");
    await runAction("limit");
    await runAction("stack");
    await runAction("doAll");
    expect(session.processedImage).toMatchObject({ width: W, height: H });

    await runAction("limit");
    expect(session.step).toBe("limited");
    expect(session.stackImage).toBeNull();
    expect(session.processedImage).toBeNull();
    expect(session.stackSize).toBeNull();
  });
});

/** Runs the chain up to a sharpened image, with a colour adjustment shown on top of it. */
async function buildEverything() {
  await runAction("setAlignpoints");
  await runAction("align");
  await runAction("limit");
  await runAction("stack");
  await runAction("doAll");
  session.tab = "wavelet";
  session.adjustedImage = new FakeImageData(new Uint8ClampedArray(W * H * 4), W, H) as unknown as ImageData;
  expect(session.step).toBe("stacked");
}

describe("Align again after Limit, Stack and Wavelet", () => {
  it("Set Alignpoints drops Limit, Stack, the sharpened image, the colour adjustment and the frame choice", async () => {
    await buildEverything();
    expect(session.frames.map((f) => f.checked)).toEqual([false, true, true, true]);

    await runAction("setAlignpoints");

    expect(session.step).toBe("points");
    expect(session.tab).toBe("align");
    expect(session.stackImage).toBeNull();
    expect(session.processedImage).toBeNull();
    expect(session.adjustedImage).toBeNull();
    expect(session.stackSize).toBeNull();
    expect(session.frames.map((f) => f.checked)).toEqual([true, true, true, true]);
    expect(session.frames.every((f) => f.quality === null && f.difference === null)).toBe(true);
  });

  it("Align drops them as soon as it starts, so a Cancel leaves nothing of the old result", async () => {
    await buildEverything();
    const running = pending();
    backend.handlers.align = () => running.promise;
    const job = runAction("align");
    await vi.waitFor(() => expect(calls("align")).toHaveLength(2));

    expect(session.step).toBe("points");
    expect(session.stackImage).toBeNull();
    expect(session.processedImage).toBeNull();
    expect(session.frames.every((f) => f.quality === null && f.checked)).toBe(true);
    expect(session.alignpoints.every((p) => p.shiftX === undefined && p.shiftY === undefined)).toBe(true);

    const cancelling = cancelJob();
    running.reject("cancelled");
    await Promise.all([job, cancelling]);
    expect(status.result).toBe("Cancelled");
    expect(session.step).toBe("points");
    expect(session.tab).toBe("align");
  });

  it("a finished Align goes back to the step after Align, and Limit and Stack run again on the new result", async () => {
    await buildEverything();
    await runAction("align");
    expect(session.step).toBe("aligned");
    expect(session.tab).toBe("align");
    expect(session.stackImage).toBeNull();
    await runAction("limit");
    await runAction("stack");
    expect(session.step).toBe("stacked");
    expect(session.stackImage).toMatchObject({ width: W, height: H });
  });

  it("a failed Align after Limit stays at the step of the points", async () => {
    await buildEverything();
    backend.handlers.align = () => {
      throw new Error("no frame");
    };
    await runAction("align");
    expect(session.step).toBe("points");
    expect(session.stackImage).toBeNull();
  });
});

describe("Wavelet", () => {
  it("Do All sends the values of the panel and keeps the whole result", async () => {
    await runAction("setAlignpoints");
    await runAction("align");
    await runAction("limit");
    await runAction("stack");
    settings.wavelet.layers[0]!.level = 100;
    await runAction("doAll");
    const sent = calls("wavelet_do_all")[0]?.args as { settings: { layers: unknown[] } };
    expect(sent.settings).toMatchObject({ scheme: "linear", filter: "gaussian", initial: 1, step: 0 });
    expect(sent.settings.layers).toHaveLength(6);
    expect(sent.settings.layers[0]).toEqual({ enabled: true, gain: 10, denoise: 0, sharpen: 0.1 });
    expect(status.result).toBe("Do All finished");
    expect(session.processedImage).toMatchObject({ width: W, height: H });
    settings.wavelet.layers[0]!.level = 10;
  });

  it("a new stack makes the sharpened image obsolete", async () => {
    await runAction("setAlignpoints");
    await runAction("align");
    await runAction("limit");
    await runAction("stack");
    await runAction("doAll");
    await runAction("stack");
    expect(session.processedImage).toBeNull();
    expect(session.step).toBe("stacked");
  });
});

describe("Do All and Process: Cancel and Pause", () => {
  async function stacked() {
    await runAction("setAlignpoints");
    await runAction("align");
    await runAction("limit");
    await runAction("stack");
  }

  it("Cancel stops Do All through the wavelet flag, keeps the earlier result and shows nothing of the run", async () => {
    await stacked();
    await runAction("doAll");
    const earlier = session.processedImage;
    expect(earlier).not.toBeNull();

    const running = pending();
    backend.handlers.wavelet_do_all = () => running.promise;
    const job = runAction("doAll");
    await vi.waitFor(() => expect(calls("wavelet_do_all")).toHaveLength(2));
    expect(status.busy?.name).toBe("Wavelet");

    const cancelling = cancelJob();
    running.reject("cancelled");
    await Promise.all([job, cancelling]);

    expect(calls("cancel_wavelet")).toHaveLength(1);
    expect(calls("cancel_processing")).toHaveLength(0);
    expect(calls("cancel_stacking")).toHaveLength(0);
    expect(status.result).toBe("Cancelled");
    expect(session.processedImage).toBe(earlier);
    expect(status.busy).toBeNull();
  });

  it("drops a Do All result that arrives after Cancel", async () => {
    await stacked();
    const running = pending();
    backend.handlers.wavelet_do_all = () => running.promise;
    const job = runAction("doAll");
    await vi.waitFor(() => expect(calls("wavelet_do_all")).toHaveLength(1));
    const cancelling = cancelJob();
    running.resolve(imageBuffer());
    await Promise.all([job, cancelling]);
    expect(status.result).toBe("Cancelled");
    expect(session.processedImage).toBeNull();
  });

  it("Pause goes to the wavelet flag, not to the align flag", async () => {
    await stacked();
    backend.handlers.pause_wavelet = () => undefined;
    const running = pending();
    backend.handlers.wavelet_do_all = () => running.promise;
    const job = runAction("doAll");
    await vi.waitFor(() => expect(calls("wavelet_do_all")).toHaveLength(1));
    await togglePause();
    expect(calls("pause_wavelet")[0]?.args).toEqual({ paused: true });
    expect(calls("pause_processing")).toHaveLength(0);
    await togglePause();
    expect(calls("pause_wavelet")[1]?.args).toEqual({ paused: false });
    running.resolve(imageBuffer());
    await job;
  });

  it("shows the layers of a Do All as progress", async () => {
    await stacked();
    const seen: number[] = [];
    backend.handlers.wavelet_do_all = () => {
      backend.listeners["wavelet-progress"]?.({ payload: { done: 1, total: 4 } });
      seen.push(status.progress);
      return imageBuffer();
    };
    await runAction("doAll");
    expect(seen).toEqual([0.25]);
    expect(backend.listeners["wavelet-progress"]).toBeUndefined();
  });

  it("Cancel on Process drops the preview that is on its way", async () => {
    await stacked();
    expect(session.processedImage).toBeNull();
    const area = (x: number) => {
      const buffer = new ArrayBuffer(16 + 4 * 4);
      const view = new DataView(buffer);
      [x, 0, 2, 2].forEach((v, i) => view.setUint32(i * 4, v, true));
      return buffer;
    };
    const running = pending();
    backend.handlers.wavelet_preview = () => running.promise;
    const job = process();
    await vi.waitFor(() => expect(calls("wavelet_preview")).toHaveLength(1));
    const cancelling = cancelJob();
    running.resolve(area(0));
    await Promise.all([job, cancelling]);
    expect(calls("cancel_wavelet")).toHaveLength(1);
    expect(status.result).toBe("Cancelled");
    expect(session.processedImage).toBeNull();

    // Without Cancel the same preview is shown.
    backend.handlers.wavelet_preview = () => area(0);
    await process();
    expect(session.processedImage).toMatchObject({ width: W, height: H });
  });
});

describe("RGB Align, Auto", () => {
  it("asks the engine for whole pixels, whatever was typed in Max shift", async () => {
    const shifts = { red: { x: 0, y: 0 }, green: { x: 0, y: 0 }, blue: { x: 0, y: 0 } };
    backend.handlers.set_color_source = () => ({ width: W, height: H, channels: 3 });
    backend.handlers.estimate_rgb_align = () => shifts;
    session.stackImage = new FakeImageData(new Uint8ClampedArray(W * H * 4), W, H) as unknown as ImageData;
    settings.wavelet.tone.maxShift = 2.5;
    await autoAlign();
    settings.wavelet.tone.maxShift = 5;
    expect(calls("estimate_rgb_align")[0]?.args).toEqual({ maxShift: 3 });
    expect(settings.wavelet.tone.shifts).toEqual(shifts);
  });
});
