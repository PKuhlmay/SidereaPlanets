import { describe, expect, it } from "vitest";
import { byteText, cpuPercent, cpuReading, isHigh, isMeasuring } from "./processUsage";

// The rules of the Mac app (mac/Tests/SidereaPlanetsTests/ProcessUsageTests.swift), so that both status bars read the same.
const sample = (cpuSeconds: number, uptime: number) => ({ cpuSeconds, memoryBytes: 0, uptime });

describe("cpuPercent: CPU load between two readings", () => {
  it("counts one busy core as 100 percent, adds up several cores and gives 0 for an idle process", () => {
    expect(cpuPercent(sample(10, 100), sample(12, 102))).toBe(100);
    expect(cpuPercent(sample(0, 0), sample(6, 2))).toBe(300);
    expect(cpuPercent(sample(5, 0), sample(5, 1))).toBe(0);
  });

  it("gives no value for readings that are not in order", () => {
    expect(cpuPercent(sample(1, 3), sample(2, 3))).toBeNull();
    expect(cpuPercent(sample(1, 5), sample(2, 4))).toBeNull();
    expect(cpuPercent(sample(9, 0), sample(8, 1))).toBeNull();
  });
});

describe("cpuReading: the text of the CPU field", () => {
  it("is whole percent of the whole machine, padded to three digits", () => {
    expect(cpuReading(600, 12)).toBe(" 50 %");
    expect(cpuReading(387.2, 4)).toBe(" 97 %");
    expect(cpuReading(25, 8)).toBe("  3 %");
    expect(cpuReading(0, 1)).toBe("  0 %");
    expect(cpuReading(12.5, 1)).toBe(" 13 %");
  });

  it("never exceeds 100 percent", () => {
    expect(cpuReading(1250, 12)).toBe("100 %");
    expect(cpuReading(1e30, 1)).toBe("100 %");
  });

  it("shows nothing for a value that is no reading", () => {
    for (const missing of [null, -1, NaN, Infinity, -Infinity]) expect(cpuReading(missing, 4), String(missing)).toBeNull();
    expect(cpuReading(50, 0)).toBeNull();
  });
});

describe("byteText: the text of the RAM field", () => {
  it("is plain below one kilobyte and counts in 1024s above", () => {
    expect(byteText(0)).toBe("0 B");
    expect(byteText(1000)).toBe("1000 B");
    expect(byteText(1024)).toBe("1.0 KB");
    expect(byteText(1_572_864)).toBe("1.5 MB");
    expect(byteText(250 * 1_048_576)).toBe("250 MB");
    expect(byteText(3 * 1_073_741_824)).toBe("3.0 GB");
    expect(byteText(-5)).toBe("0 B");
  });

  it("moves up a whole unit when rounding reaches it", () => {
    expect(byteText(1_048_575)).toBe("1.0 MB");
    expect(byteText(102_370)).toBe("100 KB");
    expect(byteText(102_300)).toBe("99.9 KB");
  });

  it("is never longer than 8 characters", () => {
    for (const bytes of [0, 999, 1_048_575, 1_023_999_999, 1_073_741_823, 12_345_678_901, 999_999_999_999_999]) {
      expect(byteText(bytes).length, String(bytes)).toBeLessThanOrEqual(8);
    }
  });
});

describe("isHigh: the CPU value in the accent colour", () => {
  it("is high from 90 percent of all cores on", () => {
    expect(isHigh(360, 4)).toBe(true);
    expect(isHigh(400, 4)).toBe(true);
    expect(isHigh(90, 1)).toBe(true);
    expect(isHigh(359.9, 4)).toBe(false);
    expect(isHigh(89.9, 1)).toBe(false);
  });

  it("is never high without a reading or without cores", () => {
    expect(isHigh(null, 8)).toBe(false);
    expect(isHigh(500, 0)).toBe(false);
    for (const value of [Infinity, NaN, -400]) expect(isHigh(value, 4), String(value)).toBe(false);
  });
});

describe("isMeasuring", () => {
  it("measures only while the readout is shown and the window is visible", () => {
    expect(isMeasuring(true, true)).toBe(true);
    expect(isMeasuring(false, true)).toBe(false);
    expect(isMeasuring(true, false)).toBe(false);
  });
});
