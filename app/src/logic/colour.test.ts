import { describe, expect, it } from "vitest";
import {
  colourPayload,
  differencePixels,
  histogramKey,
  histogramMax,
  histogramPath,
  isNeutral,
  isToneCard,
  levelToX,
  logHeight,
  moveBlack,
  moveWhite,
  pickToneImage,
  xToLevel,
} from "./colour";
import { createSettings } from "./settings";

const defaults = () => colourPayload(createSettings().wavelet);

describe("colourPayload", () => {
  it("is neutral with the defaults of the window", () => {
    expect(isNeutral(defaults())).toBe(true);
  });

  it("sends the Contrast field (100), not the slider position (250)", () => {
    const s = createSettings();
    expect(colourPayload(s.wavelet).contrast).toBe(100);
    s.wavelet.contrast = 500;
    expect(colourPayload(s.wavelet).contrast).toBe(200);
  });

  it("copies the shifts, so a later edit does not change a request in flight", () => {
    const s = createSettings();
    const payload = colourPayload(s.wavelet);
    s.wavelet.tone.shifts.red.x = 2;
    expect(payload.shifts.red.x).toBe(0);
  });

  it("stops being neutral when any value moves", () => {
    const changes: ((s: ReturnType<typeof createSettings>) => void)[] = [
      (s) => (s.wavelet.tone.shifts.blue.y = 0.5),
      (s) => (s.wavelet.tone.red = 90),
      (s) => (s.wavelet.tone.black = 3),
      (s) => (s.wavelet.tone.white = 250),
      (s) => (s.wavelet.tone.gamma = 1.2),
      (s) => (s.wavelet.contrast = 300),
      (s) => (s.wavelet.brightness = -4),
    ];
    for (const change of changes) {
      const s = createSettings();
      change(s);
      expect(isNeutral(colourPayload(s.wavelet))).toBe(false);
    }
  });
});

describe("histogramKey", () => {
  it("ignores the stretch, gamma and contrast for the histogram before the stretch", () => {
    const a = defaults();
    const b = { ...a, black: 20, white: 200, gamma: 2, contrast: 150, brightness: 30 };
    expect(histogramKey(a, false)).toBe(histogramKey(b, false));
    expect(histogramKey(a, true)).not.toBe(histogramKey(b, true));
  });

  it("follows the balance and the shifts", () => {
    const a = defaults();
    expect(histogramKey({ ...a, red: 80 }, false)).not.toBe(histogramKey(a, false));
    const moved = { ...a, shifts: { ...a.shifts, red: { x: 1, y: 0 } } };
    expect(histogramKey(moved, false)).not.toBe(histogramKey(a, false));
  });
});

describe("histogram graph", () => {
  it("scales logarithmically between 0 and 1", () => {
    expect(logHeight(0, 1000)).toBe(0);
    expect(logHeight(1000, 1000)).toBe(1);
    expect(logHeight(31, 1000)).toBeCloseTo(0.5, 1);
    expect(logHeight(5, 0)).toBe(0);
  });

  it("finds the largest count of all series", () => {
    expect(histogramMax({ channels: [[1, 5], [2, 3]], luma: [9, 1], bins: 2 })).toBe(9);
  });

  it("draws a line from the left to the right edge, the area closed down to the axis", () => {
    expect(histogramPath([0, 10], 10, 100, 50)).toBe("M0.0 50.0L100.0 0.0");
    expect(histogramPath([0, 10], 10, 100, 50, true)).toBe("M0.0 50.0L100.0 0.0L100 50L0 50Z");
    expect(histogramPath([], 10, 100, 50)).toBe("");
  });
});

describe("histogram markers", () => {
  it("map the graph position to a level and back", () => {
    expect(xToLevel(0, 212)).toBe(0);
    expect(xToLevel(106, 212)).toBe(128);
    expect(xToLevel(500, 212)).toBe(255);
    expect(xToLevel(-5, 212)).toBe(0);
    expect(levelToX(255, 212)).toBe(212);
  });

  it("keep black below white", () => {
    expect(moveBlack(300, 200)).toBe(199);
    expect(moveBlack(-5, 200)).toBe(0);
    expect(moveWhite(10, 50)).toBe(51);
    expect(moveWhite(999, 50)).toBe(255);
  });
});

describe("pickToneImage", () => {
  const images = { current: "now", clipboard: "copy", difference: "diff" };
  it("shows what the Toggle and Difference chose", () => {
    expect(pickToneImage("current", images)).toBe("now");
    expect(pickToneImage("clipboard", images)).toBe("copy");
    expect(pickToneImage("difference", images)).toBe("diff");
  });

  it("falls back to the current image when the other one does not exist", () => {
    expect(pickToneImage("clipboard", { ...images, clipboard: null })).toBe("now");
    expect(pickToneImage("difference", { ...images, difference: null })).toBe("now");
  });
});

describe("differencePixels", () => {
  const px = (rgba: number[], width = 1, height = 1) => ({ width, height, data: new Uint8ClampedArray(rgba) });

  it("amplifies the absolute difference per channel and clamps it", () => {
    const out = differencePixels(px([10, 50, 100, 255]), px([8, 53, 20, 255]));
    expect(Array.from(out?.data ?? [])).toEqual([8, 12, 255, 255]);
  });

  it("is black where the images are equal", () => {
    expect(Array.from(differencePixels(px([9, 9, 9, 255]), px([9, 9, 9, 255]))?.data ?? [])).toEqual([0, 0, 0, 255]);
  });

  it("refuses images of different sizes", () => {
    expect(differencePixels(px([0, 0, 0, 255]), px([0, 0, 0, 255, 0, 0, 0, 255], 2, 1))).toBeNull();
  });
});

describe("toneCards", () => {
  it("name the Functions buttons that have a card", () => {
    expect(isToneCard("Histogram")).toBe(true);
    expect(isToneCard("RGB Align")).toBe(true);
    expect(isToneCard("Masking")).toBe(false);
  });
});
