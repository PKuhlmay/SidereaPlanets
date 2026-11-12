import { describe, expect, it } from "vitest";
// @ts-expect-error type error without @types/node package
import { readdirSync, readFileSync } from "node:fs";

// Buttons of the step panels whose function does not exist yet are locked and say so, like the locked options of the
// Stack step (stack-pages.test.ts). They must not be clickable just to report "not available yet" in the status bar.

const read = (name: string): string => (readFileSync as (file: URL, encoding: "utf8") => string)(new URL(`../components/${name}`, import.meta.url), "utf8");
const waveletsStep = read("WaveletsStep.vue");
const alignStep = read("AlignStep.vue");

/** The lines of a template that hold a button with this label. */
const lines = (source: string, label: string): string[] => source.split("\n").filter((l) => l.includes(`>${label}</button>`));
const count = (source: string, text: string): number => source.split(text).length - 1;

describe("the step panel Wavelets", () => {
  it("locks Realign with Processed and does not run an action for it", () => {
    const [line, ...others] = lines(waveletsStep, "Realign with Processed");
    expect(others).toEqual([]);
    expect(line).toContain("disabled");
    expect(line).not.toContain("@click");
    expect(line).not.toContain(":disabled");
  });

  it("marks Realign with Processed as not available yet, in a tooltip and in a note next to it", () => {
    expect(waveletsStep).toMatch(/title="Not available yet">\s*<button[^>]*>Realign with Processed<\/button>\s*<span class="factor">Not available yet<\/span>/);
  });

  it("locks the Preview button of every layer and says why in its tooltip", () => {
    const line = waveletsStep.split("\n").find((l) => l.includes("formatPreview(layer.level)"));
    expect(line).toBeDefined();
    expect(line).toContain(" disabled>");
    expect(line).not.toContain("@click");
    expect(line).toContain("Not available yet");
  });

  it("leaves the buttons that work clickable", () => {
    for (const label of ["Load Scheme", "Save Scheme"]) {
      const [line] = lines(waveletsStep, label);
      expect(line, label).toContain("@click");
      expect(line, label).not.toContain("disabled");
    }
  });
});

describe("the step panel Align", () => {
  it("locks No Align and Scan Frames for good, not only until Align ran", () => {
    for (const label of ["No Align", "Scan Frames"]) {
      const [line] = lines(alignStep, label);
      expect(line, label).toContain(" disabled>");
      expect(line, label).not.toContain("@click");
      expect(line, label).not.toContain(":disabled");
    }
  });

  it("marks them as not available yet in a tooltip and in a note", () => {
    expect(count(alignStep, `title="Not available yet"`)).toBe(1);
    expect(count(alignStep, `Not available yet</span>`)).toBe(1);
  });
});

describe("the actions without a function", () => {
  const components = (readdirSync as (dir: URL) => string[])(new URL("../components/", import.meta.url)).filter((f) => f.endsWith(".vue"));

  it("are not run by any button of the step panels any more", () => {
    for (const id of ["realign", "noAlign", "scanFrames", "previewLayer"]) {
      const users = components.filter((f) => read(f).includes(`runAction('${id}')`));
      expect(users, id).toEqual([]);
    }
  });
});
