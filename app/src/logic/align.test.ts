import { describe, expect, it } from "vitest";
import {
  alignArgs,
  isRegion,
  limitArgs,
  limitMessages,
  qualityPercent,
  rectBetween,
  roomForPoints,
  setAlignpointsArgs,
} from "./align";
import { createSettings } from "./settings";
import { imagePosition } from "./viewport";

const align = () => createSettings().align;
const image = { w: 1024, h: 680 };

describe("setAlignpointsArgs", () => {
  it("sends the point parameters with the defaults of the reference program, the frame and no region", () => {
    expect(setAlignpointsArgs(3, align(), null)).toEqual({
      frame: 3,
      settings: {
        minDistance: 30,
        edgeDistance: 20,
        maxMovement: 5,
        roi: null,
        intensity: "default",
        lo: 30,
        hi: 230,
        boxSize: 30,
      },
    });
  });

  it("passes the region and changed values on", () => {
    const a = align();
    a.minDistance = 45;
    a.intensity = "area3x3";
    const roi = { x: 100, y: 50, width: 400, height: 300 };
    const { settings } = setAlignpointsArgs(0, a, roi);
    expect(settings.roi).toEqual(roi);
    expect(settings.minDistance).toBe(45);
    expect(settings.intensity).toBe("area3x3");
  });
});

describe("alignArgs", () => {
  it("sends box size, movement and the region", () => {
    const roi = { x: 1, y: 2, width: 30, height: 40 };
    expect(alignArgs(align(), roi)).toEqual({ settings: { boxSize: 30, maxMovement: 5, roi } });
  });
});

describe("limitArgs", () => {
  it("takes the percent field for the percent modes", () => {
    const a = align();
    expect(limitArgs(a)).toEqual({ settings: { mode: "lowestQuality", value: 80 } });
    a.limitMode = "bestPercent";
    a.limitPercent = 30;
    expect(limitArgs(a)).toEqual({ settings: { mode: "bestPercent", value: 30 } });
  });

  it("takes the number field for Frames/Apoint and Best Frames", () => {
    const a = align();
    a.limitMode = "framesPerPoint";
    a.limitCount = 12;
    expect(limitArgs(a)).toEqual({ settings: { mode: "framesPerPoint", value: 12 } });
    a.limitMode = "bestCount";
    expect(limitArgs(a)).toEqual({ settings: { mode: "bestCount", value: 12 } });
  });
});

describe("qualityPercent", () => {
  it("rates the best frame with 100 and the others relative to it", () => {
    const percent = qualityPercent([0.05, 0.1, 0.075]);
    [50, 100, 75].forEach((expected, i) => expect(percent[i]).toBeCloseTo(expected, 6));
  });

  it("gives 0 for everything if no frame has any quality", () => {
    expect(qualityPercent([0, 0])).toEqual([0, 0]);
    expect(qualityPercent([])).toEqual([]);
  });
});

describe("rectBetween", () => {
  it("is the same whichever corner the drag starts at", () => {
    const expected = { x: 100, y: 200, width: 300, height: 150 };
    expect(rectBetween({ x: 100, y: 200 }, { x: 400, y: 350 }, image)).toEqual(expected);
    expect(rectBetween({ x: 400, y: 350 }, { x: 100, y: 200 }, image)).toEqual(expected);
    expect(rectBetween({ x: 400, y: 200 }, { x: 100, y: 350 }, image)).toEqual(expected);
  });

  it("stays inside the image and uses whole pixels", () => {
    expect(rectBetween({ x: -30, y: 10.4 }, { x: 5000, y: 700 }, image)).toEqual({ x: 0, y: 10, width: 1024, height: 670 });
  });
});

describe("isRegion", () => {
  it("drops a click or a tiny drag", () => {
    expect(isRegion({ x: 5, y: 5, width: 0, height: 0 })).toBe(false);
    expect(isRegion({ x: 5, y: 5, width: 7, height: 200 })).toBe(false);
    expect(isRegion({ x: 5, y: 5, width: 8, height: 8 })).toBe(true);
  });
});

describe("roomForPoints", () => {
  it("needs more than twice Min distance from edge in both directions", () => {
    expect(roomForPoints({ x: 0, y: 0, width: 41, height: 41 }, 20)).toBe(true);
    expect(roomForPoints({ x: 0, y: 0, width: 40, height: 100 }, 20)).toBe(false);
    expect(roomForPoints({ x: 0, y: 0, width: 100, height: 40 }, 20)).toBe(false);
  });
});

describe("limitMessages", () => {
  it("reports the frames that stay like the original", () => {
    expect(limitMessages(10)).toEqual({ action: "Processing limited to 10 frames", result: "Stacksize: 10" });
  });
});

describe("imagePosition", () => {
  it("undoes zoom and pan, also outside the image", () => {
    const view = { scale: 2, x: 100, y: 50 };
    expect(imagePosition(view, 300, 150)).toEqual({ x: 100, y: 50 });
    expect(imagePosition(view, 0, 0)).toEqual({ x: -50, y: -25 });
  });
});
