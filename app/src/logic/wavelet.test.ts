import { describe, expect, it } from "vitest";
import { createSettings, defaultLayer } from "./settings";
import { colourMode, fromBackend, linkedLevels, parsePreview, previewWanted, toBackend, writePreview } from "./wavelet";

describe("values for the backend", () => {
  it("turns slider positions into factors and renames the fields", () => {
    const w = createSettings().wavelet;
    w.layers[0].level = 100;
    w.layers[1].level = -50;
    w.layers[2].denoise = 0.5;
    w.initialLayer = 2;
    w.stepIncrement = 3;
    w.scheme = "dyadic";
    w.filter = "default";
    const b = toBackend(w);
    expect(b).toMatchObject({ scheme: "dyadic", filter: "default", initial: 2, step: 3 });
    expect(b.layers).toHaveLength(6);
    expect(b.layers[0]).toEqual({ enabled: true, gain: 10, denoise: 0, sharpen: 0.1 });
    expect(b.layers[1].gain).toBe(-5);
    expect(b.layers[2].denoise).toBe(0.5);
  });

  it("the defaults of the panel are the defaults of the backend", () => {
    const b = toBackend(createSettings().wavelet);
    expect(b).toEqual({
      scheme: "linear",
      filter: "gaussian",
      initial: 1,
      step: 0,
      layers: Array.from({ length: 6 }, () => ({ enabled: true, gain: 1, denoise: 0, sharpen: 0.1 })),
    });
  });

  it("sends whole numbers for Initial Layer and Step Increment, which the backend reads as unsigned integers", () => {
    const w = createSettings().wavelet;
    w.initialLayer = 2.4;
    w.stepIncrement = -3;
    expect(toBackend(w)).toMatchObject({ initial: 2, step: 0 });
    w.initialLayer = 0;
    expect(toBackend(w).initial).toBe(1);
  });

  it("round-trips through the backend format", () => {
    const w = createSettings().wavelet;
    w.layers[3] = { enabled: false, denoise: 1.25, sharpen: 0.375, level: 77 };
    w.scheme = "dyadic";
    expect(fromBackend(toBackend(w))).toMatchObject({ scheme: "dyadic", layers: w.layers });
  });

  it("fills missing layers with the defaults", () => {
    const b = toBackend(createSettings().wavelet);
    b.layers.length = 2;
    expect(fromBackend(b).layers[5]).toEqual(defaultLayer());
  });
});

describe("linked sliders", () => {
  const levels = [10, 20, 990, -40, 10, 10];

  it("move only one slider when not linked", () => {
    expect(linkedLevels(levels, 0, 50, false)).toEqual([50, 20, 990, -40, 10, 10]);
  });

  it("move all sliders by the same amount when linked, within the range", () => {
    expect(linkedLevels(levels, 0, 30, true)).toEqual([30, 40, 1000, -20, 30, 30]);
    expect(linkedLevels(levels, 0, -50, true)).toEqual([-50, -40, 930, -50, -50, -50]);
  });

  it("keeps the moved slider inside the range", () => {
    expect(linkedLevels(levels, 1, 5000, false)[1]).toBe(1000);
  });
});

describe("preview", () => {
  // 3 x 2 pixels at (1, 1) of a 5 x 4 image.
  function answer(): ArrayBuffer {
    const buffer = new ArrayBuffer(16 + 3 * 2 * 4);
    const view = new DataView(buffer);
    [1, 1, 3, 2].forEach((v, i) => view.setUint32(i * 4, v, true));
    new Uint8Array(buffer, 16).forEach((_, i, all) => (all[i] = i + 1));
    return buffer;
  }

  it("reads area and pixels from the binary answer", () => {
    const p = parsePreview(answer());
    expect([p.x, p.y, p.width, p.height]).toEqual([1, 1, 3, 2]);
    expect(p.rgba).toHaveLength(24);
    expect(p.rgba[0]).toBe(1);
  });

  it("writes only the rows of the area into the image", () => {
    const image = new Uint8ClampedArray(5 * 4 * 4);
    writePreview(image, 5, parsePreview(answer()));
    const at = (x: number, y: number) => image[(y * 5 + x) * 4];
    expect(at(0, 0)).toBe(0);
    expect(at(1, 1)).toBe(1);
    expect(at(3, 1)).toBe(9);
    expect(at(1, 2)).toBe(13);
    expect(at(4, 2)).toBe(0);
    expect(at(1, 3)).toBe(0);
    expect(image.reduce((n, v) => n + (v > 0 ? 1 : 0), 0)).toBe(24);
  });
});

describe("colour mode", () => {
  it("maps the Colour and LRGB checkboxes", () => {
    expect(colourMode(true, false)).toBe("colour");
    expect(colourMode(true, true)).toBe("lrgb");
    expect(colourMode(false, false)).toBe("gray");
  });

  it("ignores LRGB without Colour", () => {
    expect(colourMode(false, true)).toBe("gray");
  });
});

describe("previewWanted", () => {
  const ready = { view: "wavelets", hasStack: true, busy: false, doAllCurrent: false };

  it("is true in the Wavelets step with a stack, while nothing runs and Do All is not up to date", () => {
    expect(previewWanted(ready)).toBe(true);
  });

  it("is false in the Save step, so entering it keeps the result and the message of Do All", () => {
    expect(previewWanted({ ...ready, view: "save" })).toBe(false);
  });

  it("is false when Do All already made the image for these values, also when coming back to Wavelets", () => {
    expect(previewWanted({ ...ready, doAllCurrent: true })).toBe(false);
  });

  it("is false without a stack and while a job runs", () => {
    expect(previewWanted({ ...ready, hasStack: false })).toBe(false);
    expect(previewWanted({ ...ready, busy: true })).toBe(false);
  });
});
