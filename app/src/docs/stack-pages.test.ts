import { describe, expect, it } from "vitest";
import { parsePage, renderBody } from "./docs";
// @ts-expect-error type error without @types/node package
import { readFileSync } from "node:fs";

const files = import.meta.glob("./pages/step-stack.md", { query: "?raw", import: "default", eager: true }) as Record<string, string>;
const page = parsePage("step-stack", Object.values(files)[0]);
const stackStep = (readFileSync as (file: URL, encoding: "utf8") => string)(new URL("../components/StackStep.vue", import.meta.url), "utf8");

/** Options of the Stack step that the backend ignores (docs/stacking.md, "Nicht umgesetzt"). */
const notAvailable = ["Output size: Smallest common area", "Output size: Largest area", "Output size: Region", "Stretch histogram after stacking", "Resampling", "Drizzling"];
/** Options that work: the stack reads them (`StackSettings` in stack_commands.rs). */
const works = ["Output size: Default", "Use nearest points", "Correct geometry", "Even out frame brightness", "Ignore edge pixels \\(px\\)"];

describe("documentation of the step Stack", () => {
  it("lists every option that does nothing yet as a table row that is not available yet, so the page greys it out", () => {
    const html = renderBody(page.body);
    for (const name of notAvailable) {
      expect(html, name).toMatch(new RegExp(String.raw`<tr class="off">\s*<td>${name}</td>`));
    }
  });

  it("does not grey out an option that works", () => {
    const html = renderBody(page.body);
    for (const name of works) {
      expect(html, name).toMatch(new RegExp(String.raw`<tr>\s*<td>${name}</td>`));
    }
  });

  it("describes De-rotate images as working in the Mac version, not as a locked option", () => {
    const html = renderBody(page.body);
    expect(html).toMatch(/<tr>\s*<td>De-rotate images<\/td>/);
    expect(html).toContain("<h2");
    expect(page.body).toContain("## De-rotate images");
    for (const word of ["Planet", "Start time", "Camera angle (°)", "Mirrored image", "Too short to matter", "Span 3 min 12 s"]) {
      expect(page.body, word).toContain(word);
    }
  });

  it("no longer tells the reader that Resampling and Drizzling can be set", () => {
    expect(page.body).not.toContain("Resampling and Drizzling can be set");
    expect(page.body).not.toContain("appear next to it");
  });
});

describe("the step panel Stack", () => {
  // The line of the template that holds the label of an option.
  const line = (label: string): string => {
    const found = stackStep.split("\n").find((l) => l.includes(`>${label}</label>`));
    if (!found) throw new Error(`No line for ${label}`);
    return found;
  };

  it("locks every option that does nothing yet and keeps it out of the settings", () => {
    for (const label of ["Minimized", "Maximized", "R.of Interest", "Stretch histogram after stacking", "De-rotate images", "Resampling", "Drizzling"]) {
      expect(line(label), label).toContain("disabled");
      expect(line(label), label).not.toContain("v-model");
    }
  });

  it("marks each locked option as not available yet in its row", () => {
    const count = (text: string): number => stackStep.split(text).length - 1;
    // one tooltip and one note in the row of each of the seven locked options
    expect(count(`title="Not available yet"`)).toBe(7);
    expect(count(`class="factor">Not available yet</span>`)).toBe(7);
  });

  it("leaves the options that work switchable", () => {
    for (const label of ["Default", "Correct geometry", "Normalization of frame intensity"]) {
      expect(line(label), label).not.toContain("disabled");
      expect(line(label), label).toContain("v-model");
    }
  });
});
