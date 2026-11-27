import { describe, expect, it } from "vitest";
import { createSettings } from "./settings";
import { buildSequenceRequest, buildStackRequest } from "./stack";

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
      centreOfGravity: null,
    });
  });

  it("lines the frames up by the centre of gravity like Align when the option is on (MAC-021)", () => {
    const settings = createSettings();
    settings.align.centreOfGravity = true;
    settings.align.brightnessThreshold = 40;
    expect(buildStackRequest(session(), settings).centreOfGravity).toBe(40);
  });

  it("sends De-rotate images with its settings only when the check box is on (SP-023)", () => {
    const settings = createSettings();
    Object.assign(settings.stack, { planet: "saturn", cameraAngle: -12.5, mirrored: true, startTime: 1_700_000_000.5 });
    expect(buildStackRequest(session(), settings)).not.toHaveProperty("derotation");
    settings.stack.derotate = true;
    expect(buildStackRequest(session(), settings).derotation).toEqual({
      planet: "saturn",
      cameraAngleDeg: -12.5,
      mirrored: true,
      startTimeUtc: 1_700_000_000.5,
      useFileTimestamps: true,
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

describe("buildSequenceRequest", () => {
  it("adds the Create AVI options to the request of Stack", () => {
    const settings = createSettings();
    settings.stack.framerate = 12.6;
    settings.stack.sortOnQuality = true;
    settings.stack.averageFrames = 0;
    const request = buildSequenceRequest(session(), settings);
    expect(request.stack).toEqual(buildStackRequest(session(), settings));
    expect(request.framerate).toBe(13);
    expect(request.sortOnQuality).toBe(true);
    // At least one frame per average.
    expect(request.averageFrames).toBe(1);
  });

  it("uses the defaults of the reference program: 25 frames per second, an average of 5", () => {
    const request = buildSequenceRequest(session(), createSettings());
    expect(request).toMatchObject({ framerate: 25, sortOnQuality: false, averageFrames: 5 });
  });
});
