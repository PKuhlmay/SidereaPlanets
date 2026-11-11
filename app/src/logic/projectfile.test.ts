import { describe, expect, it } from "vitest";
import { applyProject, buildProject, mergeKnown } from "./projectfile";
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
