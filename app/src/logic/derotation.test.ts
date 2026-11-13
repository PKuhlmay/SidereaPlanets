import { describe, expect, it } from "vitest";
import {
  applyDerotationSection,
  derotationRequest,
  derotationSection,
  derotationSummary,
  discOverlay,
  localInputValue,
  parseLocalInput,
  planetFromName,
  spanText,
  utcText,
  type DerotationInfo,
} from "./derotation";
import { createSettings } from "./settings";

// Ported from the Mac tests of De-rotate images (StackLogicTests, DerotationModelTests).

describe("derotationRequest", () => {
  it("is nothing while De-rotate images is off", () => {
    expect(derotationRequest(createSettings().stack)).toBeNull();
  });

  it("sends planet, camera angle, mirror and start time, and always uses the times of the file", () => {
    const stack = createSettings().stack;
    Object.assign(stack, { derotate: true, planet: "saturn", cameraAngle: -12.5, mirrored: true, startTime: 1_700_000_000.5 });
    expect(derotationRequest(stack)).toEqual({
      planet: "saturn",
      cameraAngleDeg: -12.5,
      mirrored: true,
      startTimeUtc: 1_700_000_000.5,
      useFileTimestamps: true,
    });
    stack.startTime = null;
    expect(derotationRequest(stack)?.startTimeUtc).toBeNull();
  });
});

describe("planetFromName", () => {
  it("finds the planet the file name names, in any spelling", () => {
    expect(planetFromName("JUPITER.AVI")).toBe("jupiter");
    expect(planetFromName("2023-sat_ir.ser")).toBe("saturn");
    expect(planetFromName("Mars")).toBe("mars");
    expect(planetFromName("SATURN_2023.ser")).toBe("saturn");
  });

  it("is nothing for a file that names no planet", () => {
    expect(planetFromName("moon.ser")).toBeNull();
    expect(planetFromName("capture.avi")).toBeNull();
  });
});

describe("spanText", () => {
  it("writes a span in the largest units that fit", () => {
    expect(spanText(0)).toBe("0.0 s");
    expect(spanText(0.36)).toBe("0.4 s");
    expect(spanText(9.94)).toBe("9.9 s");
    expect(spanText(44.6)).toBe("45 s");
    expect(spanText(192)).toBe("3 min 12 s");
    expect(spanText(3600)).toBe("1 h 00 min");
    expect(spanText(3900)).toBe("1 h 05 min");
    expect(spanText(-4)).toBe("0.0 s");
  });
});

describe("derotationSummary", () => {
  const info = (spanSeconds: number, rotationDegrees: number, problem: string | null = null): DerotationInfo => ({
    spanSeconds,
    rotationDegrees,
    disc: null,
    problem,
  });

  it("names span and rotation and warns when the rotation is too little to matter", () => {
    expect(derotationSummary(info(192, 5.84))).toEqual({ text: "Span 3 min 12 s · rotation 5.8°", hint: null, isProblem: false });
    expect(derotationSummary(info(90, 0.91))).toEqual({ text: "Span 1 min 30 s · rotation 0.9°", hint: "Too short to matter", isProblem: false });
    // One degree is where it starts to matter.
    expect(derotationSummary(info(90, 1.0)).hint).toBeNull();
  });

  it("shows a problem of the core instead of the numbers", () => {
    expect(derotationSummary(info(0, 0, "De-rotation needs the start time of the recording"))).toEqual({
      text: "De-rotation needs the start time of the recording",
      hint: null,
      isProblem: true,
    });
  });
});

describe("start time", () => {
  it("shows the time in UTC below the field", () => {
    expect(utcText(1_678_482_000)).toBe("2023-03-10 21:00 UTC");
  });

  it("goes into the field as local date and time to the minute and comes back as the same moment", () => {
    const value = localInputValue(1_678_482_000);
    expect(value).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/);
    expect(parseLocalInput(value)).toBe(1_678_482_000);
    // Seconds are dropped: the field is precise to the minute.
    expect(localInputValue(1_678_482_059)).toBe(value);
  });

  it("leaves the field empty without a time and reads an empty field as no time", () => {
    expect(localInputValue(null)).toBe("");
    expect(parseLocalInput("")).toBeNull();
    expect(parseLocalInput("not a date")).toBeNull();
  });
});

describe("the section derotation of the project file (same keys as the Mac app)", () => {
  it("round-trips planet, camera angle, mirror and start time", () => {
    const saved = createSettings().stack;
    Object.assign(saved, { derotate: true, planet: "mars", cameraAngle: 33.5, mirrored: true, startTime: 1_700_000_000.25 });
    const section = JSON.parse(JSON.stringify(derotationSection(saved)));
    expect(section).toEqual({ enabled: true, planet: "mars", cameraAngle: 33.5, mirrored: true, startTimeUtc: 1_700_000_000.25 });
    const back = createSettings().stack;
    applyDerotationSection(section, back);
    expect(back).toMatchObject({ derotate: true, planet: "mars", cameraAngle: 33.5, mirrored: true, startTime: 1_700_000_000.25 });
  });

  it("writes no start time when there is none", () => {
    expect(derotationSection(createSettings().stack)).not.toHaveProperty("startTimeUtc");
  });

  it("leaves the values alone when the project has no such section", () => {
    const stack = createSettings().stack;
    Object.assign(stack, { derotate: true, startTime: 1_700_000_000 });
    applyDerotationSection(undefined, stack);
    applyDerotationSection("nope", stack);
    expect(stack).toMatchObject({ derotate: true, startTime: 1_700_000_000 });
  });

  it("takes only keys of the right type and cuts the camera angle to its limits", () => {
    const stack = createSettings().stack;
    stack.startTime = 5;
    applyDerotationSection({ enabled: "yes", planet: "venus", cameraAngle: 900, mirrored: 1, startTimeUtc: "tomorrow" }, stack);
    expect(stack).toMatchObject({ derotate: false, planet: "jupiter", mirrored: false, cameraAngle: 360, startTime: 5 });
  });
});

describe("discOverlay", () => {
  it("draws the globe turned by its angle, the ring of Saturn and a stroke with N over the north pole", () => {
    const overlay = discOverlay({ x: 100, y: 80, semiMajor: 40, semiMinor: 36, angle: -12, ring: [90, 20], north: [0, -1] });
    expect(overlay.globe).toEqual({ cx: 100, cy: 80, rx: 40, ry: 36, transform: "rotate(-12 100 80)" });
    expect(overlay.ring).toEqual({ cx: 100, cy: 80, rx: 90, ry: 20, transform: "rotate(-12 100 80)" });
    // From the pole out by 14 px, the letter 10 px further.
    expect(overlay.axis).toEqual({ x1: 100, y1: 44, x2: 100, y2: 30 });
    expect(overlay.label).toEqual({ x: 100, y: 20 });
  });

  it("has no ring for Jupiter and Mars, and a ring that is only a line stays visible", () => {
    expect(discOverlay({ x: 0, y: 0, semiMajor: 10, semiMinor: 9, angle: 0, ring: null, north: [1, 0] }).ring).toBeNull();
    expect(discOverlay({ x: 0, y: 0, semiMajor: 10, semiMinor: 9, angle: 0, ring: [20, 0], north: [1, 0] }).ring?.ry).toBe(0.5);
  });
});
