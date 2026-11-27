import { describe, expect, it } from "vitest";
import {
  alignArgs,
  bestFrame,
  bestFrameHint,
  isRegion,
  limitArgs,
  limitMessages,
  qualityPercent,
  rankedFrames,
  rectBetween,
  roomForPoints,
  setAlignpointsArgs,
} from "./align";
import { createSettings } from "./settings";
import { imagePosition } from "./viewport";

const align = () => createSettings().align;
const image = { w: 1024, h: 680 };

describe("setAlignpointsArgs", () => {
  it("sends the point parameters with the default values, the frame and no region", () => {
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
    expect(alignArgs(align(), roi)).toEqual({ settings: { boxSize: 30, maxMovement: 5, roi, centreOfGravity: null } });
  });

  it("sends the Brightness threshold while Align by center of gravity is on, held to 0–255", () => {
    const a = align();
    expect([a.centreOfGravity, a.brightnessThreshold]).toEqual([false, 20]);
    a.centreOfGravity = true;
    expect(alignArgs(a, null).settings.centreOfGravity).toBe(20);
    a.brightnessThreshold = 300;
    expect(alignArgs(a, null).settings.centreOfGravity).toBe(255);
    a.brightnessThreshold = -4;
    expect(alignArgs(a, null).settings.centreOfGravity).toBe(0);
    a.centreOfGravity = false;
    expect(alignArgs(a, null).settings.centreOfGravity).toBeNull();
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

describe("typed values the backend cannot take", () => {
  // The fields accept any typed number (docs/ui.md); the commands read whole numbers from zero (u32, usize).
  it("rounds and bounds the whole-number fields of Set Alignpoints and Align", () => {
    const a = align();
    a.minDistance = 30.5;
    a.edgeDistance = -4;
    a.maxMovement = Number.NaN;
    a.boxSize = 0;
    expect(setAlignpointsArgs(0, a, null).settings).toMatchObject({ minDistance: 31, edgeDistance: 0, maxMovement: 0, boxSize: 1 });
    expect(alignArgs(a, null).settings).toMatchObject({ maxMovement: 0, boxSize: 1 });
  });

  it("rounds the count of the count modes and keeps at least one frame", () => {
    const a = align();
    a.limitMode = "bestCount";
    a.limitCount = 5.6;
    expect(limitArgs(a).settings.value).toBe(6);
    a.limitCount = 0;
    expect(limitArgs(a).settings.value).toBe(1);
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
  it("reports the frames that stay and the quality of the weakest of them", () => {
    expect(limitMessages(8, 10, [100, 93.5, 87.2, 90])).toEqual({
      action: "Keeping 8 of 10 frames",
      result: "8 frames will be stacked · weakest kept: 87%",
    });
    expect(limitMessages(1, 10, [100]).result).toBe("1 frame will be stacked · weakest kept: 100%");
  });

  it("leaves the quality out when no kept frame has one", () => {
    expect(limitMessages(10, 10, [])).toEqual({ action: "Keeping 10 of 10 frames", result: "10 frames will be stacked" });
  });
});

describe("imagePosition", () => {
  it("undoes zoom and pan, also outside the image", () => {
    const view = { scale: 2, x: 100, y: 50 };
    expect(imagePosition(view, 300, 150)).toEqual({ x: 100, y: 50 });
    expect(imagePosition(view, 0, 0)).toEqual({ x: -50, y: -25 });
  });
});

describe("Go to Best Frame", () => {
  const frame = (index: number, quality: number | null) => ({ index, quality });

  it("ranks the frames best first, equal quality by the smaller number, and keeps the order before Align", () => {
    expect(rankedFrames([frame(0, 80), frame(1, 100), frame(2, 90), frame(3, 100)]).map((f) => f.index)).toEqual([1, 3, 2, 0]);
    expect(rankedFrames([frame(0, null), frame(1, null)]).map((f) => f.index)).toEqual([0, 1]);
  });

  it("takes the first of the ranking as the best frame, none before Align", () => {
    expect(bestFrame([frame(0, 80), frame(1, 100), frame(2, 100)])).toBe(1);
    expect(bestFrame([frame(0, null), frame(1, null)])).toBeNull();
  });

  it("names the frame counted from 1 like the frame bar", () => {
    expect(bestFrameHint(6)).toBe("Best frame: 7. Run Set Alignpoints and Align again to line all frames up on this one.");
  });
});
