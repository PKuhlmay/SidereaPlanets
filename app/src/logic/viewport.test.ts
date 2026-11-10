import { describe, expect, it } from "vitest";
import { actualSize, fitView, formatZoom, imagePixel, MAX_SCALE, MIN_SCALE, processingArea, zoomAt } from "./viewport";

const image = { w: 1024, h: 680 };

describe("fitView", () => {
  it("fits the whole image into the area with a margin and centres it", () => {
    const view = fitView({ w: 1048, h: 2000 }, image);
    expect(view.scale).toBeCloseTo(1);
    expect(view.x).toBeCloseTo(12);
    expect(view.y).toBeCloseTo((2000 - 680) / 2);
  });

  it("is limited by the narrower side", () => {
    expect(fitView({ w: 536, h: 2000 }, image).scale).toBeCloseTo(0.5);
    expect(fitView({ w: 5000, h: 364 }, image).scale).toBeCloseTo(0.5);
  });
});

describe("zoomAt", () => {
  it("keeps the point under the pointer in place", () => {
    const view = { scale: 1, x: 10, y: 20 };
    const zoomed = zoomAt(view, 2, 110, 120);
    // The image pixel under (110, 120) is (100, 100) before and after.
    expect((110 - zoomed.x) / zoomed.scale).toBeCloseTo(100);
    expect((120 - zoomed.y) / zoomed.scale).toBeCloseTo(100);
    expect(zoomed.scale).toBe(2);
  });

  it("stays between the smallest and the largest scale", () => {
    expect(zoomAt({ scale: 1, x: 0, y: 0 }, 1000, 0, 0).scale).toBe(MAX_SCALE);
    expect(zoomAt({ scale: 1, x: 0, y: 0 }, 0.0001, 0, 0).scale).toBe(MIN_SCALE);
  });
});

describe("actualSize", () => {
  it("shows 100 % of the image pixels around the centre of the area", () => {
    const box = { w: 800, h: 600 };
    const view = actualSize(fitView(box, image), box);
    expect(view.scale).toBeCloseTo(1);
    expect(formatZoom(view.scale)).toBe("100%");
    // The image pixel at the centre of the area did not move.
    const before = fitView(box, image);
    expect((400 - view.x) / view.scale).toBeCloseTo((400 - before.x) / before.scale);
  });
});

describe("formatZoom", () => {
  it("rounds to whole percent of the image pixels", () => {
    expect(formatZoom(0.9812)).toBe("98%");
    expect(formatZoom(1.03)).toBe("103%");
  });
});

describe("imagePixel", () => {
  const view = { scale: 2, x: 100, y: 50 };

  it("maps a point of the area to the image pixel under it", () => {
    expect(imagePixel(view, image, 100, 50)).toEqual({ x: 0, y: 0 });
    expect(imagePixel(view, image, 501, 251)).toEqual({ x: 200, y: 100 });
  });

  it("returns null outside the image", () => {
    expect(imagePixel(view, image, 99, 60)).toBeNull();
    expect(imagePixel(view, image, 100 + 2 * 1024, 60)).toBeNull();
  });
});

describe("processingArea", () => {
  it("is a square in the middle of the image", () => {
    expect(processingArea(image, 512)).toEqual({ x: 256, y: 84, w: 512, h: 512 });
  });

  it("is cut to a smaller image", () => {
    expect(processingArea({ w: 300, h: 200 }, 512)).toEqual({ x: 0, y: 0, w: 300, h: 200 });
  });
});
