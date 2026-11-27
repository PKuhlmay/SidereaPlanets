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

  it("describes De-rotate images as working, not as a locked option", () => {
    const html = renderBody(page.body);
    expect(html).toMatch(/<tr>\s*<td>De-rotate images<\/td>\s*<td>off<\/td>\s*<td>Works\./);
    expect(page.body).not.toContain("locked for now");
    expect(page.body).not.toContain("Mac version");
    expect(page.body).not.toContain("the Windows version ignores it");
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
    const locked = ["Smallest common area", "Largest area", "Region", "Stretch histogram after stacking", "Resampling", "Drizzling"];
    for (const label of [...locked, "Use prefilters", "Compressed"]) {
      expect(line(label), label).toContain("disabled");
      expect(line(label), label).not.toContain("v-model");
    }
  });

  it("marks each locked option as not available yet, in its tooltip and, where the row has room, in a note", () => {
    const count = (text: string): number => stackStep.split(text).length - 1;
    // Six locked options of Stacking, three locked frame sizes, Compressed, Use prefilters, Codec and Compression quality.
    expect(count(`title="Not available yet`)).toBe(13);
    // A note in the row of every locked option except the side-by-side Use prefilters and the two locked fields.
    expect(count(`class="factor">Not available yet</span>`)).toBe(10);
    expect(stackStep).not.toContain(`title="Not available yet. Turns the frames back against the rotation of the planet."`);
  });

  it("has De-rotate images as a working check box with its settings", () => {
    expect(line("De-rotate images")).toContain(`v-model="stack.derotate"`);
    expect(line("De-rotate images")).not.toContain("disabled");
    expect(line("Mirrored image")).toContain(`v-model="stack.mirrored"`);
    for (const text of [
      "Turns every frame back to the middle of the recording by the rotation of the planet, so that details at the edge of the disc stay sharp. Needs the date and time of the recording.",
      ">Planet<",
      "The planet in the recording: it decides how fast it turns and how flattened it is.",
      "Start time",
      "When the first frame was taken, in the time zone of this computer. The file has no time for its frames.",
      " · taken from the file date, check it",
      "Enter the time of the first frame",
      "Camera angle (°)",
      "How far the camera was turned, in degrees counter-clockwise on the screen. 0 means celestial north is up in the picture.",
      "The picture is a mirror image, as a single mirror in the light path makes it: east is on the right when north is up.",
      "The file has a time for every frame.",
    ]) {
      expect(stackStep, text).toContain(text);
    }
  });

  it("leaves the options that work switchable", () => {
    for (const label of ["Default", "Correct geometry", "Even out frame brightness", "Sort by quality", "Moving average"]) {
      expect(line(label), label).not.toContain("disabled");
      expect(line(label), label).toContain("v-model");
    }
  });
});
