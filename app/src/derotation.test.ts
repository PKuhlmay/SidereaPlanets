// De-rotate images in the window (RK-023), on a fake backend: what a new recording suggests, when span, rotation and disc
// are asked for again, and what the panel and the image area get from the answer. Ported from the Mac's DerotationModelTests.

import { beforeEach, describe, expect, it, vi } from "vitest";

type Args = Record<string, unknown> | undefined;

const backend = vi.hoisted(() => ({
  calls: [] as { command: string; args: Args }[],
  handlers: {} as Record<string, (args: Args) => unknown>,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: async (command: string, args?: Args) => {
    backend.calls.push({ command, args });
    const handler = backend.handlers[command];
    if (!handler) throw new Error(`unexpected command ${command}`);
    return handler(args);
  },
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: async () => () => undefined }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(), save: vi.fn() }));
vi.mock("@tauri-apps/api/webview", () => ({ getCurrentWebview: () => ({ onDragDropEvent: async () => () => undefined }) }));

class FakeImageData {
  constructor(
    readonly data: Uint8ClampedArray,
    readonly width: number,
    readonly height: number,
  ) {}
}
vi.stubGlobal("ImageData", FakeImageData);
vi.stubGlobal("window", { setInterval, clearInterval, setTimeout, clearTimeout });

import { startDerotation } from "./derotation";
import type { DerotationInfo } from "./logic/derotation";
import { startSession } from "./recording";
import { session, settings, status, type RecordingInfo } from "./state";

const info = (more: Partial<RecordingInfo> = {}): RecordingInfo => ({
  format: "AVI",
  codec: "DIB",
  width: 4,
  height: 3,
  bitDepth: 8,
  frameCount: 10,
  color: "Rgb",
  fps: 25,
  observer: null,
  instrument: null,
  telescope: null,
  hasTimestamps: false,
  startTime: null,
  startTimeFromFileDate: false,
  ...more,
});

const disc = { x: 2, y: 1.5, semiMajor: 1.2, semiMinor: 1.1, angle: 0, ring: null, north: [0, -1] as [number, number] };
let answer: DerotationInfo = { spanSeconds: 192, rotationDegrees: 5.84, disc, problem: null };

function frameBuffer(): ArrayBuffer {
  const buffer = new ArrayBuffer(8 + 4 * 3 * 4);
  const view = new DataView(buffer);
  view.setUint32(0, 4, true);
  view.setUint32(4, 3, true);
  return buffer;
}

const calls = () => backend.calls.filter((c) => c.command === "derotation_info");
const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
async function eventually(condition: () => boolean): Promise<boolean> {
  for (let i = 0; i < 100; i++) {
    if (condition()) return true;
    await wait(5);
  }
  return false;
}

/** Points set on frame 4 (index 3), limited to frames 2, 5 and 9; the Stack step is open and shows frame 8. */
function limitedRecording() {
  session.referenceFrame = 3;
  session.frames.forEach((f) => (f.checked = [2, 5, 9].includes(f.index)));
  session.frame = 7;
  session.view = "stack";
  session.step = "limited";
}

startDerotation();

beforeEach(async () => {
  backend.calls = [];
  answer = { spanSeconds: 192, rotationDegrees: 5.84, disc, problem: null };
  backend.handlers = { derotation_info: () => answer, read_frame: () => frameBuffer() };
  Object.assign(settings.stack, { derotate: false, planet: "jupiter", cameraAngle: 0, mirrored: false, startTime: null });
  startSession(info(), "JUPITER.AVI");
  status.result = "";
  await wait(150);
  backend.calls = [];
});

describe("suggestions of a new recording", () => {
  it("takes the start time from the file and the planet from the file name, but never turns the option on", () => {
    startSession(info({ startTime: 1_678_482_000, startTimeFromFileDate: true }), "SATURN_2023.ser");
    expect(settings.stack.startTime).toBe(1_678_482_000);
    expect(settings.stack.planet).toBe("saturn");
    expect(settings.stack.derotate).toBe(false);
  });

  it("keeps the planet when the name has none, and drops the start time of the last recording", () => {
    settings.stack.planet = "mars";
    settings.stack.startTime = 7;
    startSession(info(), "capture.avi");
    expect(settings.stack.planet).toBe("mars");
    expect(settings.stack.startTime).toBeNull();
  });
});

describe("asking the core", () => {
  it("asks nothing while the option is off", async () => {
    limitedRecording();
    await wait(200);
    expect(calls()).toHaveLength(0);
    expect(session.derotation).toBeNull();
  });

  it("asks for the kept frames and the reference frame once the option is on, and shows the reference frame", async () => {
    limitedRecording();
    settings.stack.cameraAngle = 20;
    settings.stack.derotate = true;
    expect(await eventually(() => calls().length === 1)).toBe(true);
    expect(calls()[0].args).toEqual({
      settings: { planet: "jupiter", cameraAngleDeg: 20, mirrored: false, startTimeUtc: null, useFileTimestamps: true },
      referenceFrame: 3,
      frames: [2, 5, 9],
    });
    expect(await eventually(() => session.derotation !== null)).toBe(true);
    expect(session.derotation?.disc).toEqual(disc);
    // The disc was measured in the reference frame, so the Stack step shows it.
    expect(await eventually(() => session.frame === 3)).toBe(true);
    expect(backend.calls.some((c) => c.command === "read_frame" && c.args?.index === 3)).toBe(true);
  });

  it("asks again after every setting that matters and after the frame choice, but not for another field", async () => {
    limitedRecording();
    settings.stack.derotate = true;
    expect(await eventually(() => calls().length === 1)).toBe(true);
    settings.stack.planet = "mars";
    expect(await eventually(() => calls().length === 2)).toBe(true);
    settings.stack.mirrored = true;
    expect(await eventually(() => calls().length === 3)).toBe(true);
    settings.stack.startTime = 1_700_000_000;
    expect(await eventually(() => calls().length === 4)).toBe(true);
    session.frames[2].checked = false;
    expect(await eventually(() => calls().length === 5)).toBe(true);
    settings.stack.nearestPoints = 9;
    await wait(200);
    expect(calls()).toHaveLength(5);
  });

  it("measures once after a fast series of changes, with the last value", async () => {
    limitedRecording();
    settings.stack.derotate = true;
    for (let angle = 1; angle <= 6; angle++) {
      settings.stack.cameraAngle = angle;
      await wait(10);
    }
    await wait(250);
    expect(calls()).toHaveLength(1);
    expect((calls()[0].args?.settings as { cameraAngleDeg: number }).cameraAngleDeg).toBe(6);
  });

  it("takes the numbers and the disc away when the option is turned off", async () => {
    limitedRecording();
    settings.stack.derotate = true;
    expect(await eventually(() => session.derotation !== null)).toBe(true);
    settings.stack.derotate = false;
    expect(await eventually(() => session.derotation === null)).toBe(true);
  });

  it("says in the status bar when the core fails", async () => {
    backend.handlers.derotation_info = () => {
      throw new Error("no recording");
    };
    limitedRecording();
    settings.stack.derotate = true;
    expect(await eventually(() => status.result !== "")).toBe(true);
    expect(status.result).toBe("De-rotation failed: no recording");
    expect(session.derotation).toBeNull();
  });
});
