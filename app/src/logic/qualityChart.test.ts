import { describe, expect, it } from "vitest";
import { keptCount, qualityChart, type LimitValues, type RatedFrame } from "./qualityChart";

const QUALITY = [92.1, 88.4, 97.3, 85.0, 90.6, 99.2, 83.7, 95.8, 87.2, 100.0];
const frames = (qualities: number[], checked = true): RatedFrame[] => qualities.map((quality) => ({ quality, checked }));
const limit = (limitMode: LimitValues["limitMode"], limitPercent = 80, limitCount = 200): LimitValues => ({
  limitMode,
  limitPercent,
  limitCount,
});

describe("keptCount (as limit_frames in crates/core)", () => {
  const sorted = [...QUALITY].sort((a, b) => b - a);

  it("Lowest Quality (%) keeps the frames at or above the share of the best", () => {
    expect(keptCount(sorted, limit("lowestQuality", 90))).toBe(6);
    expect(keptCount(sorted, limit("lowestQuality", 0))).toBe(10);
  });

  it("Lowest Quality (%) keeps a frame that sits exactly on the limit", () => {
    expect(keptCount([100, 90, 80], limit("lowestQuality", 90))).toBe(2);
  });

  it("is zero for no frames, whatever the limit", () => {
    expect(keptCount([], limit("bestCount", 80, 5))).toBe(0);
  });

  it("Best Frames (%) rounds up", () => {
    expect(keptCount(sorted, limit("bestPercent", 25))).toBe(3);
  });

  it("Best Frames keeps the count, at most all frames and at least one", () => {
    expect(keptCount(sorted, limit("bestCount", 80, 4))).toBe(4);
    expect(keptCount(sorted, limit("bestCount", 80, 200))).toBe(10);
    expect(keptCount(sorted, limit("bestCount", 80, 0))).toBe(1);
  });

  it("has no count for Frames/Apoint, which needs the qualities of the points", () => {
    expect(keptCount(sorted, limit("framesPerPoint", 80, 2))).toBeNull();
  });
});

describe("qualityChart", () => {
  it("sorts the bars best first and marks the kept frames", () => {
    const chart = qualityChart(frames(QUALITY), limit("lowestQuality", 90), 240);
    expect(chart.bars).toHaveLength(10);
    expect(chart.bars[0]).toEqual({ height: 1, kept: true });
    expect(chart.bars.map((b) => b.kept)).toEqual([true, true, true, true, true, true, false, false, false, false]);
    const heights = chart.bars.map((b) => b.height);
    expect([...heights].sort((a, b) => b - a)).toEqual(heights);
  });

  it("draws Lowest Quality (%) as a horizontal line at the value", () => {
    const chart = qualityChart(frames(QUALITY), limit("lowestQuality", 90), 240);
    expect(chart.line).toEqual({ kind: "level", at: 0.9 });
    expect(chart.value).toBe("90%");
  });

  it("draws the other modes as a vertical line after the last kept frame", () => {
    const chart = qualityChart(frames(QUALITY), limit("bestCount", 80, 4), 240);
    expect(chart.line).toEqual({ kind: "after", at: 0.4 });
    expect(chart.value).toBe("4");
  });

  it("follows a changed limit value at once", () => {
    expect(qualityChart(frames(QUALITY), limit("bestPercent", 50), 240).kept).toBe(5);
    expect(qualityChart(frames(QUALITY), limit("bestPercent", 80), 240).kept).toBe(8);
  });

  it("shows the frames the last Limit kept for Frames/Apoint", () => {
    const list = frames(QUALITY).map((f, i) => ({ ...f, checked: i % 2 === 0 }));
    const chart = qualityChart(list, limit("framesPerPoint", 80, 1), 240);
    expect(chart.kept).toBe(5);
  });

  it("joins neighbouring frames into one bar when there are more frames than room", () => {
    const many = frames(Array.from({ length: 1000 }, (_, i) => 100 - i * 0.05));
    const chart = qualityChart(many, limit("bestCount", 80, 500), 240);
    expect(chart.bars.length).toBeLessThanOrEqual(240);
    expect(chart.bars.length).toBe(Math.ceil(1000 / 5));
    expect(chart.bars[0]?.kept).toBe(true);
    expect(chart.bars[chart.bars.length - 1]?.kept).toBe(false);
    expect(chart.kept).toBe(500);
  });

  it("draws each bar as the share of the best quality, whatever the best quality is", () => {
    const chart = qualityChart(frames([50, 25]), limit("bestCount", 80, 2), 240);
    expect(chart.bars.map((b) => b.height)).toEqual([1, 0.5]);
  });

  it("gives a joined bar the mean height of its frames and counts it as kept when half of them are kept", () => {
    const chart = qualityChart(frames([100, 50, 40, 20]), limit("bestCount", 80, 3), 2);
    expect(chart.bars.map((b) => b.height)).toEqual([0.75, 0.3]);
    expect(chart.bars.map((b) => b.kept)).toEqual([true, true]);
    expect(qualityChart(frames([100, 50, 40, 20]), limit("bestCount", 80, 1), 2).bars.map((b) => b.kept)).toEqual([true, false]);
  });

  it("describes itself for screen readers", () => {
    expect(qualityChart(frames(QUALITY), limit("bestCount", 80, 8), 240).label).toBe("Quality of 10 frames, best first. 8 are kept.");
  });

  it("is empty before Align rated the frames", () => {
    const chart = qualityChart([{ quality: null, checked: true }], limit("lowestQuality"), 240);
    expect(chart.bars).toEqual([]);
    expect(chart.line).toBeNull();
  });
});
