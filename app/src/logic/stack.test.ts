import { describe, expect, it } from "vitest";
import { createSettings } from "./settings";
import { buildStackRequest } from "./stack";

const session = () => ({
  roi: null,
  referenceFrame: 2,
  alignpoints: [
    { x: 10, y: 20, checked: true },
    { x: 30, y: 40, checked: false },
    { x: 50, y: 60, shiftX: 1, shiftY: -1, checked: true },
  ],
  frames: [
    { index: 0, checked: true, quality: 90, difference: 1 },
    { index: 1, checked: false, quality: 50, difference: 2 },
    { index: 2, checked: true, quality: 100, difference: 0 },
  ],
});

describe("buildStackRequest", () => {
  it("takes the checked points and frames and the default options", () => {
    expect(buildStackRequest(session(), createSettings())).toEqual({
      referenceFrame: 2,
      points: [
        { x: 10, y: 20 },
        { x: 50, y: 60 },
      ],
      frames: [0, 2],
      boxSize: 30,
      maxMovement: 5,
      nearestPoints: 3,
      correctGeometry: true,
      normalize: false,
      edgeSkip: 0,
      roi: null,
    });
  });

  it("turns typed values into numbers the backend accepts", () => {
    const settings = createSettings();
    settings.stack.nearestPoints = 0;
    settings.stack.edgePixels = -4;
    settings.align.maxMovement = 2.6;
    const request = buildStackRequest(session(), settings);
    expect([request.nearestPoints, request.edgeSkip, request.maxMovement]).toEqual([1, 0, 3]);
  });
});
