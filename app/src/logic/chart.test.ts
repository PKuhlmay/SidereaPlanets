import { describe, expect, it } from "vitest";
import { seriesPoints } from "./chart";

const box = { left: 8, top: 10, width: 100, height: 80 };

describe("seriesPoints", () => {
  it("spreads the values from the left to the right edge, the maximum at the top", () => {
    expect(seriesPoints([50, 100, 25], box)).toBe("8.0,50.0 58.0,10.0 108.0,70.0");
  });

  it("scales each series to its own maximum", () => {
    expect(seriesPoints([2, 4], box)).toBe(seriesPoints([20, 40], box));
  });

  it("is empty without data or with only zeros", () => {
    expect(seriesPoints([], box)).toBe("");
    expect(seriesPoints([0, 0], box)).toBe("");
  });

  it("puts a single value at the left edge", () => {
    expect(seriesPoints([5], box)).toBe("8.0,10.0");
  });
});
