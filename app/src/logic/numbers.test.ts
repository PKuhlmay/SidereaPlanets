import { describe, expect, it } from "vitest";
import { clampNumber, formatNumber, parseNumber, stepNumber } from "./numbers";

describe("formatNumber", () => {
  it("uses the decimals of the field and a point as separator", () => {
    expect(formatNumber(0.1, 3)).toBe("0.100");
    expect(formatNumber(0, 2)).toBe("0.00");
    expect(formatNumber(30, 0)).toBe("30");
  });
});

describe("parseNumber", () => {
  it("reads integers, decimals and signs", () => {
    expect(parseNumber("30")).toBe(30);
    expect(parseNumber(" 0.25 ")).toBe(0.25);
    expect(parseNumber("-5")).toBe(-5);
    expect(parseNumber(".5")).toBe(0.5);
  });

  it("accepts values outside the usual range, as the original does", () => {
    expect(parseNumber("99")).toBe(99);
  });

  it("returns null for anything else", () => {
    expect(parseNumber("")).toBeNull();
    expect(parseNumber("abc")).toBeNull();
    expect(parseNumber("1,5")).toBeNull();
    expect(parseNumber("1e3")).toBeNull();
  });
});

describe("clampNumber", () => {
  it("only limits where a bound is given", () => {
    expect(clampNumber(5, 1, 10)).toBe(5);
    expect(clampNumber(0, 1, 10)).toBe(1);
    expect(clampNumber(11, 1, 10)).toBe(10);
    expect(clampNumber(-100)).toBe(-100);
  });
});

describe("stepNumber", () => {
  it("moves by the step and rounds to the decimals of the field", () => {
    expect(stepNumber(0.1, 1, 0.2, 1)).toBe(0.3);
    expect(stepNumber(0.1, -1, 0.005, 3)).toBe(0.095);
    expect(stepNumber(7500, 1, 100, 0)).toBe(7600);
  });

  it("stays within the limits", () => {
    expect(stepNumber(1, -1, 1, 0, 1, 10)).toBe(1);
    expect(stepNumber(10, 1, 1, 0, 1, 10)).toBe(10);
  });
});
