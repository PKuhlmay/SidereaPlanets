// Load Scheme and Save Scheme report in the status bar. Loading changes the layer values, and in the Wavelets step
// that starts a new preview, whose own message ("Preview updated") must not replace the one of Load Scheme.
// The preview watcher only runs inside the Tauri window, so these tests stand in for it (`__TAURI_INTERNALS__`).

import { beforeEach, describe, expect, it, vi } from "vitest";

type Args = Record<string, unknown> | undefined;

const backend = vi.hoisted(() => ({
  handlers: {} as Record<string, (args: Args) => unknown>,
  dialogs: { open: "" as string | null, save: "" as string | null },
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: async (command: string, args?: Args) => {
    const handler = backend.handlers[command];
    if (!handler) throw new Error(`unexpected command ${command}`);
    return handler(args);
  },
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: async () => () => undefined }));
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: async () => backend.dialogs.open,
  save: async () => backend.dialogs.save,
}));

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
import { session, settings, status } from "./state";
import { loadScheme, saveScheme } from "./wavelet";

/** x, y, width, height (u32 LE), then RGBA: what wavelet_preview answers for a 2 x 1 pixel area. */
function previewBuffer(): ArrayBuffer {
  const buffer = new ArrayBuffer(16 + 2 * 4);
  const view = new DataView(buffer);
  view.setUint32(8, 2, true);
  view.setUint32(12, 1, true);
  return buffer;
}

/** A scheme with layer 1 at 10.0 (gain 10). */
const scheme = () => ({
  scheme: "dyadic",
  filter: "default",
  initial: 1,
  step: 0,
  layers: [{ enabled: true, gain: 10, denoise: 0, sharpen: 0.1 }, ...Array.from({ length: 5 }, () => ({ enabled: true, gain: 1, denoise: 0, sharpen: 0.1 }))],
});

const pause = () => new Promise((resolve) => setTimeout(resolve, 30));

beforeEach(async () => {
  backend.handlers = { wavelet_preview: () => previewBuffer(), load_wavelet_scheme: () => scheme(), save_wavelet_scheme: () => undefined };
  backend.dialogs = { open: "C:/schemes/sharp.rwv", save: "C:/schemes/mine.rwv" };
  settings.wavelet.layers.forEach((layer) => Object.assign(layer, defaultLayer()));
  settings.wavelet.hold = true;
  session.view = "open";
  session.stackImage = null;
  status.busy = null;
  status.result = "";
  status.action = "";
  await pause();
});

/** The Wavelets step with a stack: every change of a value starts a preview. */
async function inWaveletsStep(): Promise<void> {
  session.stackImage = new FakeImageData(new Uint8ClampedArray(8), 2, 1) as unknown as ImageData;
  session.view = "wavelets";
  await pause();
  status.result = "";
  status.action = "";
}

describe("Load Scheme", () => {
  it("says which scheme was loaded and puts the values in", async () => {
    await loadScheme();
    expect(settings.wavelet.layers[0].level).toBe(100);
    expect(settings.wavelet.scheme).toBe("dyadic");
    expect(status.result).toBe("Scheme loaded: sharp.rwv");
  });

  it("still says so after the new values started a preview in the Wavelets step", async () => {
    await inWaveletsStep();
    await loadScheme();
    await pause();
    // The preview ran with the loaded values ...
    expect(status.action).toMatch(/^Preview calculated in \d+ ms$/);
    // ... and did not replace the message.
    expect(status.result).toBe("Scheme loaded: sharp.rwv");
  });

  it("says so as well when the loaded values are the ones that were set already", async () => {
    await inWaveletsStep();
    settings.wavelet.layers[0].level = 100;
    settings.wavelet.scheme = "dyadic";
    settings.wavelet.filter = "default";
    await pause();
    status.result = "";
    await loadScheme();
    await pause();
    expect(status.result).toBe("Scheme loaded: sharp.rwv");
  });

  it("reports a file that cannot be read and keeps the values", async () => {
    backend.handlers.load_wavelet_scheme = () => {
      throw new Error("not a scheme file");
    };
    await loadScheme();
    expect(status.result).toBe("Load Scheme failed: not a scheme file");
    expect(settings.wavelet.layers[0].level).toBe(10);
  });

  it("says nothing when the dialog is cancelled", async () => {
    backend.dialogs.open = null;
    await loadScheme();
    expect(status.result).toBe("");
  });
});

describe("Save Scheme", () => {
  it("says where the scheme was saved", async () => {
    await saveScheme();
    expect(status.result).toBe("Scheme saved: mine.rwv");
  });

  it("reports a file that cannot be written", async () => {
    backend.handlers.save_wavelet_scheme = () => {
      throw new Error("disk full");
    };
    await saveScheme();
    expect(status.result).toBe("Save Scheme failed: disk full");
  });
});
