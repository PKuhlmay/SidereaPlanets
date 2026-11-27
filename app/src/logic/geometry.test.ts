import { describe, expect, it } from "vitest";
import {
  applyGeometrySection,
  defaultGeometry,
  flipHorizontally,
  flipVertically,
  geometryPayload,
  geometrySection,
  isOrientationNeutral,
  orientationLayout,
  orientationSummary,
  orientedSize,
  resizeSize,
  resizeTarget,
  RESIZE_METHODS,
  RESIZE_LIMITS,
  rotate,
  setAngle,
  setResizeHeight,
  setResizeLock,
  setResizeWidth,
  switchToPixels,
  wrapAngle,
} from "./geometry";

// Flip and Rotate and Resize Image (SP-022): the cases of the earlier SwiftUI app and, for the sizes, the
// values of the tests in crates/core/src/geometry.rs, so that the window shows the size the core writes.
const source = { width: 1200, height: 900 };

describe("Flip and Rotate", () => {
  it("wraps angles into -180 … 180 in tenths", () => {
    expect(wrapAngle(190)).toBe(-170);
    expect(wrapAngle(-190)).toBe(170);
    expect(wrapAngle(180)).toBe(180);
    expect(wrapAngle(-180)).toBe(180);
    expect(wrapAngle(360)).toBe(0);
    expect(wrapAngle(12.34)).toBe(12.3);
  });

  it("turns the picture on screen with the rotate buttons", () => {
    const o = defaultGeometry().orientation;
    rotate(o, 90);
    expect(o.angle).toBe(90);
    rotate(o, 90);
    expect(o.angle).toBe(180);
    rotate(o, 90);
    expect(o.angle).toBe(-90);
    rotate(o, -90);
    rotate(o, -90);
    rotate(o, -90);
    expect(o.angle).toBe(0);
    rotate(o, 180);
    rotate(o, 180);
    expect(isOrientationNeutral(o)).toBe(true);
  });

  it("mirrors what is on screen, so a flip changes the sign of the angle", () => {
    const o = defaultGeometry().orientation;
    setAngle(o, 30);
    flipHorizontally(o);
    expect([o.flipHorizontal, o.flipVertical, o.angle]).toEqual([true, false, -30]);
    flipHorizontally(o);
    expect([o.flipHorizontal, o.angle]).toEqual([false, 30]);
    flipVertically(o);
    expect([o.flipVertical, o.angle]).toEqual([true, -30]);
    const plain = defaultGeometry().orientation;
    flipHorizontally(plain);
    expect([plain.flipHorizontal, plain.angle, isOrientationNeutral(plain)]).toEqual([true, 0, false]);
  });

  it("holds the angle to the limits and to tenths", () => {
    const o = defaultGeometry().orientation;
    setAngle(o, 250);
    expect(o.angle).toBe(180);
    setAngle(o, -999);
    expect(o.angle).toBe(180);
    setAngle(o, 15.26);
    expect(o.angle).toBe(15.3);
    setAngle(o, 0.04);
    expect(o.angle).toBe(0);
    expect(isOrientationNeutral(o)).toBe(true);
  });

  it("names what changes and leaves the method out of the layout", () => {
    const o = defaultGeometry().orientation;
    expect(orientationSummary(o)).toBeNull();
    setAngle(o, 15);
    expect(orientationSummary(o)).toBe("turned 15.0° clockwise");
    flipHorizontally(o);
    expect(orientationSummary(o)).toBe("turned 15.0° counterclockwise, flipped horizontally");
    const layout = orientationLayout(o);
    o.method = "bilinear";
    expect(orientationLayout(o)).toEqual(layout);
    o.cropToFit = true;
    expect(orientationLayout(o)).not.toEqual(layout);
  });

  it("gives the turned canvas the size the core makes", () => {
    const turn = (angle: number) => ({ ...defaultGeometry().orientation, angle });
    expect(orientedSize(100, 50, turn(30))).toEqual({ width: 112, height: 94 });
    expect(orientedSize(100, 50, turn(-30))).toEqual({ width: 112, height: 94 });
    expect(orientedSize(100, 50, turn(90))).toEqual({ width: 50, height: 100 });
    expect(orientedSize(100, 50, turn(0))).toEqual({ width: 100, height: 50 });
    expect(orientedSize(100, 50, turn(180))).toEqual({ width: 100, height: 50 });
    // Crop to fit keeps the shape and leaves no black corners, so it is smaller than the source.
    const fitted = orientedSize(100, 60, { ...turn(20), cropToFit: true });
    expect(fitted.width).toBeLessThan(100);
    expect(fitted.height).toBeLessThan(60);
    expect(fitted.width % 2).toBe(0);
  });
});

describe("Resize Image", () => {
  it("scales both sides by a percentage, rounded half away from zero and held to 1 … 16384", () => {
    const r = defaultGeometry().resize;
    expect(resizeSize(source, r)).toEqual(source);
    r.percent = 50;
    expect(resizeSize(source, r)).toEqual({ width: 600, height: 450 });
    r.percent = 400;
    expect(resizeSize(source, r)).toEqual({ width: 4800, height: 3600 });
    r.percent = 50;
    expect(resizeSize({ width: 101, height: 11 }, r)).toEqual({ width: 51, height: 6 });
    r.percent = 10;
    expect(resizeSize({ width: 4, height: 4 }, r)).toEqual({ width: 1, height: 1 });
  });

  it("lets a locked side decide the other and holds typed sides to what the core takes", () => {
    const r = defaultGeometry().resize;
    switchToPixels(r, source);
    expect([r.mode, r.width, r.height]).toEqual(["pixels", 1200, 900]);
    setResizeWidth(r, 800, source);
    expect(r.driver).toBe("width");
    expect(resizeSize(source, r)).toEqual({ width: 800, height: 600 });
    setResizeHeight(r, 450, source);
    expect(resizeSize(source, r)).toEqual({ width: 600, height: 450 });
    expect(resizeTarget(r)).toEqual({ pixels: { width: null, height: 450 } });
    setResizeWidth(r, 99_999, source);
    expect(r.width).toBe(RESIZE_LIMITS.side.max);
    setResizeWidth(r, 0, source);
    expect(r.width).toBe(1);
  });

  it("fixes both sides without the lock", () => {
    const r = defaultGeometry().resize;
    switchToPixels(r, source);
    setResizeLock(r, false, source);
    setResizeWidth(r, 800, source);
    expect(resizeSize(source, r)).toEqual({ width: 800, height: 900 });
    setResizeHeight(r, 100, source);
    expect(resizeSize(source, r)).toEqual({ width: 800, height: 100 });
    expect(resizeTarget(r)).toEqual({ pixels: { width: 800, height: 100 } });
    expect(resizeSize({ width: 10, height: 10 }, r)).toEqual({ width: 800, height: 100 });
    setResizeLock(r, true, source);
    expect(resizeSize(source, r)).toEqual({ width: Math.trunc((1200 * 100) / 900), height: 100 });
  });

  it("offers the methods with Lanczos3 first", () => {
    expect(RESIZE_METHODS.map((m) => m.title)).toEqual(["Lanczos3", "Bicubic", "Bilinear", "Nearest Neighbor"]);
    expect(defaultGeometry().resize.method).toBe("lanczos3");
  });
});

describe("what Save Image gets and the project file", () => {
  it("sends nothing while neither card changes the picture", () => {
    const g = defaultGeometry();
    expect(geometryPayload(g, source)).toBeNull();
    g.resize.percent = 50;
    expect(geometryPayload(g, source)).toEqual({
      orientation: { flipHorizontal: false, flipVertical: false, angle: 0, cropToFit: false, method: "bicubic" },
      resize: { target: { percent: 50 }, method: "lanczos3" },
    });
    g.resize.percent = 100;
    setAngle(g.orientation, -12.5);
    expect(geometryPayload(g, source)?.resize).toBeNull();
  });

  it("keeps every value in the section geometry", () => {
    const g = defaultGeometry();
    setAngle(g.orientation, -37.5);
    Object.assign(g.orientation, { flipHorizontal: true, cropToFit: true, method: "bilinear" });
    Object.assign(g.resize, { mode: "pixels", percent: 250, width: 800, height: 600, lockAspect: false, driver: "height", method: "nearest" });
    const loaded = defaultGeometry();
    applyGeometrySection(JSON.parse(JSON.stringify(geometrySection(g))), loaded);
    expect(loaded).toEqual(g);
  });

  it("keeps the values without a section and cuts wrong ones", () => {
    const g = defaultGeometry();
    setAngle(g.orientation, 10);
    applyGeometrySection(undefined, g);
    expect(g.orientation.angle).toBe(10);
    applyGeometrySection(
      {
        angle: 900,
        flipHorizontal: "yes",
        rotationMethod: "lanczos",
        resize: { percent: 5000, width: -4, height: 500_000_000, method: "nearest" },
      },
      g,
    );
    expect([g.orientation.angle, g.orientation.flipHorizontal, g.orientation.method]).toEqual([180, false, "bicubic"]);
    expect([g.resize.percent, g.resize.width, g.resize.height, g.resize.method]).toEqual([400, 0, RESIZE_LIMITS.side.max, "nearest"]);
  });
});
