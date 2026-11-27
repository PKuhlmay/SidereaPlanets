import { describe, expect, it } from "vitest";
import { applyProject, buildProject, isProjectFile, mergeKnown } from "./projectfile";
import { createSettings } from "./settings";

describe("buildProject", () => {
  it("holds the recording and the settings of the steps, but not the threshold", () => {
    const settings = createSettings();
    settings.align.threshold = 9;
    settings.align.minDistance = 45;
    settings.lrgb = true;
    const project = buildProject(["C:/a/jupiter.avi"], settings);
    expect(project.formatVersion).toBe(1);
    expect(project.recording).toEqual(["C:/a/jupiter.avi"]);
    expect(project.sections.align).toMatchObject({ minDistance: 45 });
    expect(project.sections.align).not.toHaveProperty("threshold");
    expect(project.sections.colour).toEqual({ colour: true, lrgb: true });
  });

  it("is a copy of the settings", () => {
    const settings = createSettings();
    const project = buildProject([], settings);
    settings.wavelet.layers[0].level = 77;
    expect(JSON.stringify(project.sections.wavelet)).not.toContain("77");
  });
});

describe("applyProject", () => {
  it("restores what buildProject stored", () => {
    const saved = createSettings();
    saved.align.minDistance = 45;
    saved.stack.nearestPoints = 5;
    saved.wavelet.layers[2].sharpen = 0.4;
    saved.wavelet.scheme = "dyadic";
    saved.colour = false;
    const project = JSON.parse(JSON.stringify(buildProject(["x.ser"], saved)));

    const target = createSettings();
    applyProject(target, project);
    expect(target.align.minDistance).toBe(45);
    expect(target.stack.nearestPoints).toBe(5);
    expect(target.wavelet.layers[2].sharpen).toBe(0.4);
    expect(target.wavelet.scheme).toBe("dyadic");
    expect(target.colour).toBe(false);
  });

  it("leaves settings alone that the project lacks or has with the wrong type", () => {
    const target = createSettings();
    applyProject(target, {
      formatVersion: 1,
      recording: [],
      sections: { align: { minDistance: "far", edgeDistance: 7 }, unknown: { a: 1 }, colour: { colour: "yes" } },
    });
    expect(target.align.minDistance).toBe(30);
    expect(target.align.edgeDistance).toBe(7);
    expect(target.stack.nearestPoints).toBe(3);
    expect(target.colour).toBe(true);
  });
});

describe("Align by center of gravity in the project file", () => {
  it("writes centreOfGravity and brightnessThreshold in the section align", () => {
    const settings = createSettings();
    expect(buildProject([], settings).sections.align).toMatchObject({ centreOfGravity: false, brightnessThreshold: 20 });
    settings.align.centreOfGravity = true;
    settings.align.brightnessThreshold = 35;
    expect(buildProject([], settings).sections.align).toMatchObject({ centreOfGravity: true, brightnessThreshold: 35 });
  });

  it("reads them back, holds the threshold to 0–255 and keeps the values a file lacks", () => {
    const target = createSettings();
    applyProject(target, { formatVersion: 1, recording: [], sections: { align: { centreOfGravity: true, brightnessThreshold: 999 } } });
    expect([target.align.centreOfGravity, target.align.brightnessThreshold]).toEqual([true, 255]);
    applyProject(target, { formatVersion: 1, recording: [], sections: { align: { brightnessThreshold: -3 } } });
    expect([target.align.centreOfGravity, target.align.brightnessThreshold]).toEqual([true, 0]);
    applyProject(target, { formatVersion: 1, recording: [], sections: { align: { minDistance: 40 } } });
    expect([target.align.centreOfGravity, target.align.brightnessThreshold]).toEqual([true, 0]);
  });
});

describe("De-rotate images in the project file", () => {
  it("stores the settings in the section derotation, not in the section stack, and reads them back", () => {
    const saved = createSettings();
    Object.assign(saved.stack, { derotate: true, planet: "mars", cameraAngle: 33.5, mirrored: true, startTime: 1_700_000_000.25 });
    const project = JSON.parse(JSON.stringify(buildProject(["x.ser"], saved)));
    expect(project.sections.derotation).toEqual({ enabled: true, planet: "mars", cameraAngle: 33.5, mirrored: true, startTimeUtc: 1_700_000_000.25 });
    for (const key of ["derotate", "planet", "cameraAngle", "mirrored", "startTime"]) expect(project.sections.stack).not.toHaveProperty(key);

    const target = createSettings();
    applyProject(target, project);
    expect(target.stack).toMatchObject({ derotate: true, planet: "mars", cameraAngle: 33.5, mirrored: true, startTime: 1_700_000_000.25 });
  });

  it("keeps the values a project without the section, or with missing keys, does not name", () => {
    const target = createSettings();
    Object.assign(target.stack, { derotate: true, planet: "saturn", startTime: 1_700_000_000 });
    applyProject(target, { formatVersion: 1, recording: [], sections: { stack: { nearestPoints: 4 } } });
    expect(target.stack).toMatchObject({ derotate: true, planet: "saturn", startTime: 1_700_000_000, nearestPoints: 4 });
    applyProject(target, { formatVersion: 1, recording: [], sections: { derotation: { mirrored: true } } });
    expect(target.stack).toMatchObject({ derotate: true, planet: "saturn", mirrored: true, startTime: 1_700_000_000 });
  });
});

describe("Color Mixing and the refine cards in the project file", () => {
  it("stores the mixing matrix under wavelet.tone.mix and the refine values in a section of their own", () => {
    const saved = createSettings();
    saved.wavelet.tone.mix.matrix = [0, 0, 100, 0, 100, 0, 100, 0, 0];
    Object.assign(saved.refine, { denoise: 30, maskOn: true, feather: 9 });
    const project = JSON.parse(JSON.stringify(buildProject(["x.ser"], saved)));
    expect(project.sections.refine).toEqual({ denoise: 30, deringing: 0, radius: 4, maskOn: true, threshold: 20, feather: 9, invert: false });
    expect(project.sections.wavelet.tone.mix.matrix).toEqual([0, 0, 100, 0, 100, 0, 100, 0, 0]);

    const target = createSettings();
    applyProject(target, project);
    expect(target.wavelet.tone.mix.matrix).toEqual([0, 0, 100, 0, 100, 0, 100, 0, 0]);
    expect(target.refine).toMatchObject({ denoise: 30, maskOn: true, feather: 9 });
  });

  it("cuts refine values to their limits and takes a matrix only when it is nine numbers", () => {
    const target = createSettings();
    applyProject(target, {
      formatVersion: 1,
      recording: [],
      sections: {
        refine: { denoise: 300, radius: 0, threshold: -5, feather: 80.4 },
        wavelet: { tone: { mix: { matrix: [50, 0, 0, 0, 50, 0, 0, 0], preserveBrightness: true } } },
      },
    });
    expect(target.refine).toMatchObject({ denoise: 100, radius: 1, threshold: 0, feather: 50 });
    expect(target.wavelet.tone.mix.matrix).toEqual([100, 0, 0, 0, 100, 0, 0, 0, 100]);
    expect(target.wavelet.tone.mix.preserveBrightness).toBe(true);

    applyProject(target, {
      formatVersion: 1,
      recording: [],
      sections: { wavelet: { tone: { mix: { matrix: [300, 0, 0, 0, 100, 0, 0, 0, -12.4] } } } },
    });
    expect(target.wavelet.tone.mix.matrix).toEqual([200, 0, 0, 0, 100, 0, 0, 0, -12]);
  });
});

describe("Flip and Rotate and Resize Image in the project file", () => {
  it("writes the section geometry and reads it back, cut to the limits", () => {
    const saved = createSettings();
    Object.assign(saved.geometry.orientation, { flipVertical: true, angle: -12.5 });
    saved.geometry.resize.percent = 250;
    const project = JSON.parse(JSON.stringify(buildProject(["x.ser"], saved)));
    expect(project.sections.geometry).toMatchObject({ flipVertical: true, angle: -12.5, resize: { percent: 250 } });
    const target = createSettings();
    applyProject(target, project);
    expect(target.geometry).toEqual(saved.geometry);
    applyProject(target, { formatVersion: 1, recording: [], sections: { geometry: { resize: { percent: 9000 } } } });
    expect(target.geometry.resize.percent).toBe(400);
    expect(target.geometry.orientation.angle).toBe(-12.5);
  });
});

describe("mergeKnown", () => {
  it("ignores keys the target does not have, non-finite numbers and non-objects", () => {
    const target: Record<string, unknown> = { a: 1, nested: { b: true }, list: [{ c: 1 }, { c: 2 }] };
    mergeKnown(target, { a: Number.NaN, extra: 5, nested: { b: false }, list: [{ c: 9 }] });
    expect(target).toEqual({ a: 1, nested: { b: false }, list: [{ c: 9 }, { c: 2 }] });
    mergeKnown(target, "text");
    mergeKnown(target, null);
    expect(target.a).toBe(1);
  });
});

describe("isProjectFile", () => {
  it("knows a project file by its extension, whatever the case", () => {
    expect(isProjectFile("/Users/a/JUPITER.sidereaplanets")).toBe(true);
    expect(isProjectFile("C:\\data\\Saturn.SidereaPlanets")).toBe(true);
    expect(isProjectFile("/Users/a/JUPITER.ser")).toBe(false);
    expect(isProjectFile("/Users/a/sidereaplanets")).toBe(false);
  });
});
