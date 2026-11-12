// Open project and the Wavelets layers: a new stacked image puts the layers back to their defaults unless Hold
// Wavelet Setting is on. The Wavelets step stays locked until Stack, so the values a project file brought along have
// to survive exactly that one stack. The watcher that does the reset only runs inside the Tauri window, so these tests
// stand in for it (`__TAURI_INTERNALS__`).

import { beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick } from "vue";

type Args = Record<string, unknown> | undefined;

const backend = vi.hoisted(() => ({ handlers: {} as Record<string, (args: Args) => unknown> }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: async (command: string, args?: Args) => {
    const handler = backend.handlers[command];
    if (!handler) throw new Error(`unexpected command ${command}`);
    return handler(args);
  },
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: async () => () => undefined }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(), save: vi.fn() }));

class FakeImageData {
  constructor(
    readonly data: Uint8ClampedArray,
    readonly width: number,
    readonly height: number,
  ) {}
}
vi.stubGlobal("ImageData", FakeImageData);
vi.stubGlobal("window", { __TAURI_INTERNALS__: {}, setInterval, clearInterval, setTimeout, clearTimeout });

import { defaultLayer } from "./logic/settings";
import { openProject } from "./saving";
import { session, settings } from "./state";
import "./wavelet";

const info = { format: "AVI", codec: "DIB", width: 2, height: 1, bitDepth: 8, frameCount: 1, color: "Rgb", fps: null, observer: null, instrument: null, telescope: null };

function frameBuffer(): ArrayBuffer {
  const buffer = new ArrayBuffer(8 + 2 * 4);
  const view = new DataView(buffer);
  view.setUint32(0, 2, true);
  view.setUint32(4, 1, true);
  return buffer;
}

const stackedImage = () => new FakeImageData(new Uint8ClampedArray(8), 2, 1) as unknown as ImageData;

/** A project file whose first layer is set to 10.0 (slider position 100) and whose hold flag is as given. */
function projectWith(hold: boolean) {
  const layers = Array.from({ length: 6 }, defaultLayer);
  layers[0].level = 100;
  return {
    path: "C:/data/JUPITER.rkproj",
    project: { formatVersion: 1, recording: ["C:/data/JUPITER.AVI"], sections: { wavelet: { hold, layers } } },
    missingFiles: [],
  };
}

beforeEach(() => {
  backend.handlers = { open_recording: () => info, read_frame: () => frameBuffer() };
  settings.wavelet.hold = false;
  settings.wavelet.layers.forEach((layer) => Object.assign(layer, defaultLayer()));
  session.stackImage = null;
});

describe("layer values from a project file", () => {
  it("survive the first stacked image after Open project, although Hold Wavelet Setting is off", async () => {
    backend.handlers.project_open = () => projectWith(false);
    await openProject();
    expect(settings.wavelet.layers[0].level).toBe(100);

    session.stackImage = stackedImage();
    await nextTick();
    expect(settings.wavelet.layers[0].level).toBe(100);
  });

  it("go back to the defaults with the second stacked image: the exception is used up", async () => {
    backend.handlers.project_open = () => projectWith(false);
    await openProject();
    session.stackImage = stackedImage();
    await nextTick();

    session.stackImage = stackedImage();
    await nextTick();
    expect(settings.wavelet.layers[0].level).toBe(10);
  });

  it("are not touched by the empty stack of the new recording that Open project opens", async () => {
    backend.handlers.project_open = () => projectWith(false);
    session.stackImage = stackedImage();
    await nextTick();
    await openProject();
    await nextTick();
    expect(session.stackImage).toBeNull();

    session.stackImage = stackedImage();
    await nextTick();
    expect(settings.wavelet.layers[0].level).toBe(100);
  });

  it("do not change the rule without a project: a new stack resets the layers", async () => {
    settings.wavelet.layers[0].level = 100;
    session.stackImage = stackedImage();
    await nextTick();
    expect(settings.wavelet.layers[0].level).toBe(10);
  });

  it("keep Hold Wavelet Setting as the project file says, and not switch it on", async () => {
    backend.handlers.project_open = () => projectWith(false);
    await openProject();
    expect(settings.wavelet.hold).toBe(false);
  });
});
