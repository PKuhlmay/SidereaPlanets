import { describe, expect, it } from "vitest";
import {
  ASPECTS,
  centreCrossArm,
  centred,
  clamped,
  cropAspect,
  cropToCore,
  dragged,
  drawn,
  effectiveTolerance,
  fitted,
  full,
  handleAt,
  isFull,
  MINIMUM_SIZE,
  ratio,
  rect,
  sizeText,
  thirdsLines,
} from "./crop";

// The geometry of the crop rectangle, the cases that drag, reshape and clamp
// the rectangle.
const W = 1000;
const H = 800;
const { square, threeByTwo, fourByThree } = ASPECTS;

describe("thirds and the centre cross", () => {
  it("divides the rectangle into nine equal fields, unrounded where a side does not divide by three", () => {
    expect(thirdsLines(rect(100, 50, 400, 350))).toEqual({ xs: [200, 300], ys: [150, 250] });
    const lines = thirdsLines(rect(10, 20, 21, 30));
    expect(lines.xs[0]).toBeCloseTo(10 + 11 / 3, 9);
    expect(lines.xs[1]).toBeCloseTo(10 + 22 / 3, 9);
    expect(lines.ys[0]).toBeCloseTo(20 + 10 / 3, 9);
    expect(lines.ys[1]).toBeCloseTo(20 + 20 / 3, 9);
  });

  it("keeps the full arm on a large rectangle and caps it at a quarter of the shorter side", () => {
    expect(centreCrossArm(rect(100, 50, 800, 770), 10)).toBe(10);
    expect(centreCrossArm(rect(0, 0, 20, 12), 10)).toBe(3);
    expect(centreCrossArm(rect(5, 7, 18, 16), 10)).toBe(2.25);
    expect(centreCrossArm(rect(0, 0, 8, 8), 10)).toBe(2);
    expect(centreCrossArm(rect(0, 0, 40, 40), 10)).toBe(10);
    expect(centreCrossArm(rect(0, 0, 40, 40), 11)).toBe(10);
  });
});

describe("free dragging", () => {
  const r = rect(100, 100, 500, 400);

  it("knows the whole image", () => {
    expect(isFull(full(W, H), W, H)).toBe(true);
    expect(isFull(rect(1, 0, W, H), W, H)).toBe(false);
  });

  it("moves an edge alone and a corner with two edges", () => {
    expect(dragged(r, "left", 30, 999, W, H)).toEqual(rect(130, 100, 500, 400));
    expect(dragged(r, "bottomRight", 50, -20, W, H)).toEqual(rect(100, 100, 550, 380));
  });

  it("stops an edge at the image and short of the opposite edge", () => {
    expect(dragged(r, "left", -500, 0, W, H).minX).toBe(0);
    expect(dragged(r, "right", 9999, 0, W, H).maxX).toBe(W);
    expect(dragged(r, "left", 9999, 0, W, H).minX).toBe(500 - MINIMUM_SIZE);
    expect(dragged(r, "bottom", 0, -9999, W, H).maxY).toBe(100 + MINIMUM_SIZE);
  });

  it("moves the body and keeps it inside the image", () => {
    expect(dragged(r, "body", 40, 10, W, H)).toEqual(rect(140, 110, 540, 410));
    expect(dragged(r, "body", 9999, -9999, W, H)).toEqual(rect(600, 0, 1000, 300));
  });

  it("rounds fractions of a pixel to whole pixels", () => {
    expect(dragged(r, "left", 10.4, 0, W, H).minX).toBe(110);
    expect(dragged(r, "left", 10.6, 0, W, H).minX).toBe(111);
  });

  it("orders a drawn rectangle, keeps it inside the image and drops one that is too small", () => {
    expect(drawn(400, 300, 100, 200, W, H)).toEqual(rect(100, 200, 400, 300));
    expect(drawn(900, 700, 1500, 2000, W, H)).toEqual(rect(900, 700, 1000, 800));
    expect(drawn(10, 10, 12, 300, W, H)).toBeNull();
  });

  it("finds corners first, then edges along their length, then the body", () => {
    expect(handleAt(r, 102, 98, 6)).toBe("topLeft");
    expect(handleAt(r, 498, 403, 6)).toBe("bottomRight");
    expect(handleAt(r, 300, 103, 6)).toBe("top");
    expect(handleAt(r, 496, 250, 6)).toBe("right");
    expect(handleAt(r, 300, 250, 6)).toBe("body");
    expect(handleAt(r, 700, 250, 6)).toBeNull();
    expect(handleAt(r, 700, 100, 6)).toBeNull();
  });

  it("caps the grab reach at a third of the shorter side, so a small rectangle still has a body", () => {
    expect(effectiveTolerance(rect(100, 100, 108, 108), 6)).toBe(8 / 3);
    expect(effectiveTolerance(rect(0, 0, 300, 200), 7)).toBe(7);
    expect(effectiveTolerance(rect(0, 0, 90, 30), 20)).toBe(10);
    const small = rect(100, 100, 108, 108);
    expect(handleAt(small, 104, 104, 6)).toBe("body");
    expect(handleAt(small, 101, 101, 6)).toBe("topLeft");
    expect(handleAt(small, 107, 107, 6)).toBe("bottomRight");
    expect(handleAt(small, 104, 101, 6)).toBe("top");
  });
});

describe("tiny images and the whole image", () => {
  it("keeps an image below the minimum size as the whole rectangle", () => {
    const r = full(5, 5);
    expect(r).toEqual(rect(0, 0, 5, 5));
    expect(drawn(0, 0, 5, 5, 5, 5)).toBeNull();
    expect(dragged(r, "body", 3, -3, 5, 5)).toEqual(r);
    expect(dragged(r, "right", 9999, 0, 5, 5).maxX).toBe(5);
    expect(dragged(r, "bottom", 0, 9999, 5, 5).maxY).toBe(5);
    expect(dragged(r, "topLeft", -9999, -9999, 5, 5)).toEqual(r);
  });

  it("keeps a one-pixel image as the whole rectangle", () => {
    const r = full(1, 1);
    expect(drawn(0, 0, 1, 1, 1, 1)).toBeNull();
    expect(dragged(r, "body", -5, 5, 1, 1)).toEqual(r);
    expect(dragged(r, "bottomRight", 50, 50, 1, 1)).toEqual(r);
  });

  it("does not move a rectangle that covers the whole image", () => {
    const whole = rect(0, 0, W, H);
    expect(dragged(whole, "body", 50, -30, W, H)).toEqual(whole);
    expect(dragged(whole, "body", -9999, 9999, W, H)).toEqual(whole);
  });
});

describe("fixed aspect ratio", () => {
  it("knows its ratio lying and standing and refuses a long side shorter than the short one", () => {
    expect(ratio(threeByTwo, false)).toBe(1.5);
    expect(ratio(threeByTwo, true)).toBeCloseTo(2 / 3, 12);
    expect(cropAspect(0, 2)).toBeNull();
    expect(cropAspect(2, 3)).toBeNull();
    expect(ratio(square, true)).toBe(1);
  });

  it("reshapes around the same centre and area as far as the image allows", () => {
    expect(fitted(rect(200, 50, 600, 750), threeByTwo, W, H)).toEqual(rect(184, 76, 616, 724));
    expect(fitted(rect(0, 0, W, H), fourByThree, W, H)).toEqual(rect(0, 25, 1000, 775));
  });

  it("slides a reshaped rectangle inside instead of cutting it", () => {
    const reshaped = fitted(rect(0, 100, 300, 400), threeByTwo, W, H);
    expect([reshaped.minX, reshaped.maxX - reshaped.minX, reshaped.maxY - reshaped.minY]).toEqual([0, 367, 245]);
    expect([reshaped.minY, reshaped.maxY]).toEqual([128, 373]);
  });

  it("does not shrink the rectangle when the format is switched back and forth", () => {
    const start = rect(200, 100, 800, 700);
    const twice = fitted(fitted(start, square, W, H), threeByTwo, W, H);
    const area = (start.maxX - start.minX) * (start.maxY - start.minY);
    expect(Math.abs((twice.maxX - twice.minX) * (twice.maxY - twice.minY) - area) / area).toBeLessThan(0.01);
  });

  it("drags a corner with the ratio and the opposite corner kept", () => {
    const r = rect(100, 100, 400, 300);
    const moved = dragged(r, "bottomRight", 90, 10, W, H, threeByTwo);
    expect([moved.minX, moved.minY, moved.maxX - moved.minX, moved.maxY - moved.minY]).toEqual([100, 100, 390, 260]);
    expect(dragged(r, "topLeft", 60, 0, W, H, square)).toEqual(rect(160, 60, 400, 300));
  });

  it("drags an edge with the ratio around the middle of the other side", () => {
    const r = rect(100, 100, 400, 300);
    expect(dragged(r, "right", 150, 0, W, H, threeByTwo)).toEqual(rect(100, 50, 550, 350));
    expect(dragged(r, "bottom", 0, 100, W, H, square)).toEqual(rect(100, 100, 400, 400));
  });

  it("stops a corner at the image and never goes below the minimum size", () => {
    const r = rect(100, 100, 400, 300);
    expect(dragged(r, "bottomRight", 9999, 9999, W, H, threeByTwo)).toEqual(rect(100, 100, 1000, 700));
    const moved = dragged(r, "bottomRight", -9999, -9999, W, H, threeByTwo);
    expect(moved.maxX - moved.minX).toBeGreaterThanOrEqual(MINIMUM_SIZE);
    expect(moved.maxY - moved.minY).toBeGreaterThanOrEqual(MINIMUM_SIZE);
  });

  it("draws in the direction of the drag, lying or standing, inside the image", () => {
    expect(drawn(100, 100, 400, 150, W, H, square)).toEqual(rect(100, 100, 400, 400));
    expect(drawn(100, 100, 400, 150, W, H, threeByTwo)).toEqual(rect(100, 100, 400, 300));
    expect(drawn(500, 500, 450, 200, W, H, threeByTwo)).toEqual(rect(300, 200, 500, 500));
    expect(drawn(900, 700, 1500, 720, W, H, square)).toEqual(rect(900, 700, 1000, 800));
    expect(drawn(10, 10, 12, 12, W, H, square)).toBeNull();
  });

  it("keeps both sides of the smallest rectangle at the minimum when it is reshaped", () => {
    const smallest = rect(100, 100, 108, 108);
    expect(fitted(smallest, threeByTwo, W, H)).toEqual(rect(98, 100, 110, 108));
    expect(fitted(smallest, fourByThree, W, H)).toEqual(rect(99, 100, 110, 108));
  });

  it("stops an edge where the other side reaches the image", () => {
    expect(dragged(rect(100, 50, 400, 250), "right", 500, 0, W, H, threeByTwo)).toEqual(rect(100, 0, 550, 300));
    expect(dragged(rect(100, 500, 400, 700), "right", 500, 0, W, H, threeByTwo)).toEqual(rect(100, 400, 700, 800));
    expect(dragged(rect(30, 100, 330, 300), "bottom", 0, 400, W, H, threeByTwo)).toEqual(rect(0, 100, 360, 340));
  });

  it("keeps the ratio on the left and top edges too, and never below the minimum", () => {
    const r = rect(300, 200, 600, 400);
    expect(dragged(r, "left", -150, 0, W, H, threeByTwo)).toEqual(rect(150, 150, 600, 450));
    expect(dragged(r, "top", 0, -100, W, H, threeByTwo)).toEqual(rect(225, 100, 675, 400));
    const s = rect(100, 100, 400, 300);
    expect(dragged(s, "right", -9999, 0, W, H, threeByTwo)).toEqual(rect(100, 196, 112, 204));
    expect(dragged(s, "bottom", 0, -9999, W, H, threeByTwo)).toEqual(rect(244, 100, 256, 108));
  });

  it("keeps a standing rectangle standing and handles 3:4 lying and standing", () => {
    const standing = rect(100, 100, 300, 400);
    expect(dragged(standing, "bottomRight", 50, 30, W, H, threeByTwo)).toEqual(rect(100, 100, 350, 475));
    expect(dragged(standing, "topLeft", 0, -90, W, H, threeByTwo)).toEqual(rect(40, 10, 300, 400));
    expect(drawn(100, 100, 500, 200, W, H, fourByThree)).toEqual(rect(100, 100, 500, 400));
    expect(drawn(500, 500, 450, 200, W, H, fourByThree)).toEqual(rect(275, 200, 500, 500));
    const r = rect(100, 100, 400, 300);
    expect(dragged(r, "bottomRight", 100, 0, W, H, fourByThree)).toEqual(rect(100, 100, 500, 400));
    expect(dragged(r, "right", 100, 0, W, H, fourByThree)).toEqual(rect(100, 50, 500, 350));
    expect(dragged(r, "bottom", 0, 100, W, H, fourByThree)).toEqual(rect(50, 100, 450, 400));
  });
});

describe("holding, centring and the size", () => {
  it("holds a rectangle inside the image", () => {
    expect(clamped(rect(-50, -10, 2000, 900), W, H)).toEqual(full(W, H));
    expect(clamped(rect(900, 700, 1100, 900), W, H)).toEqual(rect(800, 600, 1000, 800));
    expect(clamped(rect(10, 10, 12, 12), W, H)).toEqual(rect(10, 10, 18, 18));
    expect(clamped(rect(100, 100, 500, 400), W, H)).toEqual(rect(100, 100, 500, 400));
  });

  it("centres a share of each side in the shape of the format", () => {
    expect(centred(0.5, W, H)).toEqual(rect(250, 200, 750, 600));
    const sq = centred(0.5, W, H, square);
    expect(sq.maxX - sq.minX).toBe(sq.maxY - sq.minY);
    const ft = centred(0.8, W, H, fourByThree);
    expect(Math.abs((ft.maxX - ft.minX) / (ft.maxY - ft.minY) - 4 / 3)).toBeLessThan(0.01);
  });

  it("reads width times height in pixels and gives the core its rectangle", () => {
    expect(sizeText(rect(100, 50, 900, 650))).toBe("800 × 600 px");
    expect(cropToCore(rect(100, 50, 900, 650))).toEqual({ x: 100, y: 50, width: 800, height: 600 });
  });
});
