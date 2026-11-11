import { describe, expect, it } from "vitest";
import { atLeast, formatDuration, pickImage, stepButtons, tabLocked, tabs, timeLeft, windowTitle, type Step } from "./workflow";

describe("tabs", () => {
  it("come in the order and with the labels of the original", () => {
    expect(tabs.map((t) => t.label)).toEqual(["Align", "Stack", "Wavelet"]);
  });

  it("unlock one after the other: Stack after Limit, Wavelet after Stack", () => {
    const locked = (step: Step) => tabs.map((t) => tabLocked(step, t.id));
    expect(locked("empty")).toEqual([false, true, true]);
    expect(locked("loaded")).toEqual([false, true, true]);
    expect(locked("aligned")).toEqual([false, true, true]);
    expect(locked("limited")).toEqual([false, false, true]);
    expect(locked("stacked")).toEqual([false, false, false]);
  });

  it("name the missing step in the tooltip", () => {
    expect(tabs.find((t) => t.id === "stack")?.hint).toBe("Run Limit first");
    expect(tabs.find((t) => t.id === "wavelet")?.hint).toBe("Run Stack first");
  });
});

describe("stepButtons (docs/design.md, Zustände)", () => {
  const state = (step: Step) => {
    const b = stepButtons(step, false);
    return {
      setAlignpoints: [b.setAlignpoints.next, b.setAlignpoints.disabled, b.setAlignpoints.done],
      align: [b.align.next, b.align.disabled, b.align.done],
      limit: [b.limit.next, b.limit.disabled, b.limit.done],
      stack: [b.stack.next, b.stack.disabled, b.stack.done],
      saveImage: b.saveImage.disabled,
    };
  };

  it("fills Set Alignpoints once a recording is loaded, the rest is locked", () => {
    expect(state("loaded")).toEqual({
      setAlignpoints: [true, false, false],
      align: [false, true, false],
      limit: [false, true, false],
      stack: [false, true, false],
      saveImage: true,
    });
  });

  it("fills Align after the points are set; Set Alignpoints stays usable with a check mark", () => {
    const s = state("points");
    expect(s.setAlignpoints).toEqual([false, false, true]);
    expect(s.align).toEqual([true, false, false]);
    expect(s.limit[1]).toBe(true);
  });

  it("fills Limit after Align; Set Alignpoints and Align stay usable, with check marks", () => {
    const s = state("aligned");
    expect(s.setAlignpoints).toEqual([false, false, true]);
    expect(s.align).toEqual([false, false, true]);
    expect(s.limit).toEqual([true, false, false]);
  });

  it("fills Stack after Limit, Save image stays locked, Set Alignpoints and Align can run again", () => {
    const s = state("limited");
    expect(s.setAlignpoints).toEqual([false, false, true]);
    expect(s.align).toEqual([false, false, true]);
    expect(s.limit).toEqual([false, false, true]);
    expect(s.stack).toEqual([true, false, false]);
    expect(s.saveImage).toBe(true);
  });

  it("opens Save image after Stack, Set Alignpoints and Align can still run again", () => {
    const s = state("stacked");
    expect(s.setAlignpoints).toEqual([false, false, true]);
    expect(s.align).toEqual([false, false, true]);
    expect(s.stack).toEqual([false, false, true]);
    expect(s.saveImage).toBe(false);
  });

  it("locks everything without a recording", () => {
    const b = stepButtons("empty", false);
    expect(Object.values(b).every((x) => x.disabled && !x.next)).toBe(true);
  });

  it("locks every step during a calculation and fills none", () => {
    const b = stepButtons("points", true);
    expect(Object.values(b).every((x) => x.disabled && !x.next)).toBe(true);
  });

  it("never fills more than one step button", () => {
    for (const step of ["empty", "loaded", "points", "aligned", "limited", "stacked"] as Step[]) {
      expect(Object.values(stepButtons(step, false)).filter((x) => x.next).length).toBeLessThanOrEqual(1);
    }
  });
});

describe("atLeast", () => {
  it("compares steps by their order", () => {
    expect(atLeast("aligned", "points")).toBe(true);
    expect(atLeast("points", "aligned")).toBe(false);
    expect(atLeast("empty", "loaded")).toBe(false);
  });
});

describe("windowTitle", () => {
  it("shows the file name after loading", () => {
    expect(windowTitle("JUPITER.AVI", null)).toBe("SidereaPlanets — JUPITER.AVI");
    expect(windowTitle("", null)).toBe("SidereaPlanets");
  });

  it("shows step and percent during a calculation", () => {
    expect(windowTitle("JUPITER.AVI", { name: "Align", progress: 0.4 })).toBe("SidereaPlanets · Align 40%");
  });
});

describe("time display", () => {
  it("formats as hh:mm:ss", () => {
    expect(formatDuration(0)).toBe("00:00:00");
    expect(formatDuration(65)).toBe("00:01:05");
    expect(formatDuration(3725.4)).toBe("01:02:05");
  });

  it("estimates the time left from the progress", () => {
    expect(timeLeft(4, 0.4)).toBeCloseTo(6);
    expect(timeLeft(4, 0)).toBe(0);
    expect(timeLeft(10, 1)).toBe(0);
  });
});

describe("pickImage", () => {
  const images = { frame: "frame", stack: "stack", processed: "processed" };

  it("shows the current frame on the Align tab", () => {
    expect(pickImage("align", "aligned", false, images)).toBe("frame");
  });

  it("shows the frame on the Stack tab until the stack exists, then the stack", () => {
    expect(pickImage("stack", "limited", false, images)).toBe("frame");
    expect(pickImage("stack", "stacked", false, images)).toBe("stack");
    expect(pickImage("stack", "stacked", false, { ...images, stack: null })).toBe("frame");
  });

  it("shows the processed image on the Wavelet tab and the stack with Before", () => {
    expect(pickImage("wavelet", "stacked", false, images)).toBe("processed");
    expect(pickImage("wavelet", "stacked", true, images)).toBe("stack");
  });

  it("shows the stack on the Wavelet tab while nothing is processed", () => {
    expect(pickImage("wavelet", "stacked", false, { ...images, processed: null })).toBe("stack");
  });
});
