import { describe, expect, it } from "vitest";
import { ChangeLog, drawRegion, unionRect } from "./dirty";

const rect = (x: number, y: number, width: number, height: number) => ({ x, y, width, height });

describe("unionRect", () => {
  it("is the smallest rectangle holding both", () => {
    expect(unionRect(rect(10, 10, 20, 20), rect(25, 5, 30, 10))).toEqual(rect(10, 5, 45, 25));
  });

  it("is the second rectangle when there is no first one", () => {
    expect(unionRect(undefined, rect(1, 2, 3, 4))).toEqual(rect(1, 2, 3, 4));
  });
});

describe("ChangeLog", () => {
  it("gathers what changed in a buffer until it is taken", () => {
    const log = new ChangeLog();
    const buffer = new ArrayBuffer(8);
    log.mark(buffer, rect(0, 0, 10, 10));
    log.mark(buffer, rect(40, 40, 10, 10));
    expect(log.take(buffer)).toEqual(rect(0, 0, 50, 50));
    expect(log.take(buffer)).toBeUndefined();
  });

  it("keeps the buffers apart", () => {
    const log = new ChangeLog();
    const a = new ArrayBuffer(8);
    const b = new ArrayBuffer(8);
    log.mark(a, rect(0, 0, 5, 5));
    expect(log.take(b)).toBeUndefined();
    expect(log.take(a)).toEqual(rect(0, 0, 5, 5));
  });
});

describe("drawRegion", () => {
  const surface = {};
  const buffer = new ArrayBuffer(8);
  const drawn = { surface, buffer, width: 100, height: 80 };
  const image = { surface, buffer, width: 100, height: 80 };

  it("draws only the changed rectangle over the same buffer on the same canvas", () => {
    expect(drawRegion(drawn, image, rect(20, 10, 30, 30))).toEqual(rect(20, 10, 30, 30));
  });

  it("cuts the rectangle to the image", () => {
    expect(drawRegion(drawn, image, rect(90, 70, 30, 30))).toEqual(rect(90, 70, 10, 10));
  });

  it("draws everything the first time", () => {
    expect(drawRegion(null, image, rect(20, 10, 30, 30))).toBeUndefined();
  });

  it("draws everything when another image is on the canvas", () => {
    expect(drawRegion({ ...drawn, buffer: new ArrayBuffer(8) }, image, rect(20, 10, 30, 30))).toBeUndefined();
  });

  it("draws everything when the size or the canvas changed", () => {
    expect(drawRegion({ ...drawn, width: 50 }, image, rect(20, 10, 30, 30))).toBeUndefined();
    expect(drawRegion({ ...drawn, surface: {} }, image, rect(20, 10, 30, 30))).toBeUndefined();
  });

  it("draws everything when nothing is known about the change", () => {
    expect(drawRegion(drawn, image, undefined)).toBeUndefined();
  });
});

describe("drawing the changes only", () => {
  const W = 20;
  const H = 12;

  /** `putImageData(image, 0, 0, x, y, w, h)` on a canvas that is a plain pixel array here. */
  function put(canvas: Uint8ClampedArray, image: Uint8ClampedArray, part: ReturnType<typeof rect> | undefined) {
    const { x, y, width, height } = part ?? rect(0, 0, W, H);
    for (let row = y; row < y + height; row++) {
      const from = (row * W + x) * 4;
      canvas.set(image.subarray(from, from + width * 4), from);
    }
  }

  /** A preview: writes `value` into the rectangle of the image and logs the change, as renderPreview does. */
  function preview(image: Uint8ClampedArray, log: ChangeLog, area: ReturnType<typeof rect>, value: number) {
    for (let row = area.y; row < area.y + area.height; row++) image.fill(value, (row * W + area.x) * 4, (row * W + area.x + area.width) * 4);
    log.mark(image.buffer, area);
  }

  it("leaves the canvas as a whole new draw would, also when a draw is skipped or the area moves", () => {
    const image = new Uint8ClampedArray(W * H * 4).fill(7);
    const canvas = new Uint8ClampedArray(W * H * 4);
    const surface = {};
    const log = new ChangeLog();
    let last: { surface: object; buffer: object; width: number; height: number } | null = null;
    const draw = () => {
      const now = { surface, buffer: image.buffer, width: W, height: H };
      put(canvas, image, drawRegion(last, now, log.take(image.buffer)));
      last = now;
    };

    draw();
    preview(image, log, rect(4, 2, 8, 6), 100);
    draw();
    // Two previews of different areas before the viewer draws once.
    preview(image, log, rect(2, 1, 5, 4), 150);
    preview(image, log, rect(10, 6, 8, 5), 200);
    draw();
    expect(canvas).toEqual(image);
  });
});
