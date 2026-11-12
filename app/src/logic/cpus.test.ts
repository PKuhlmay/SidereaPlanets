import { describe, expect, it } from "vitest";
import { clampCpus, coreCount } from "./cpus";

describe("coreCount", () => {
  it("takes the reported number of cores", () => {
    expect(coreCount(8)).toBe(8);
    expect(coreCount(16)).toBe(16);
  });

  it("counts anything unusable as one core", () => {
    expect(coreCount(0)).toBe(1);
    expect(coreCount(-4)).toBe(1);
    expect(coreCount(Number.NaN)).toBe(1);
    expect(coreCount(Number.POSITIVE_INFINITY)).toBe(1);
  });
});

describe("clampCpus", () => {
  it("keeps a value from 1 up to the number of cores", () => {
    expect(clampCpus(1, 8)).toBe(1);
    expect(clampCpus(5, 8)).toBe(5);
    expect(clampCpus(8, 8)).toBe(8);
  });

  it("raises a value below 1 and cuts one above the cores", () => {
    expect(clampCpus(0, 8)).toBe(1);
    expect(clampCpus(-3, 8)).toBe(1);
    expect(clampCpus(9, 8)).toBe(8);
    expect(clampCpus(1000, 8)).toBe(8);
  });

  it("rounds a fraction and falls back to 1 for no number", () => {
    expect(clampCpus(3.4, 8)).toBe(3);
    expect(clampCpus(Number.NaN, 8)).toBe(1);
  });

  it("never goes beyond one on a machine that reports no cores", () => {
    expect(clampCpus(4, 0)).toBe(1);
  });
});
