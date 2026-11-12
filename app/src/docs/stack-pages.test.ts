import { describe, expect, it } from "vitest";
import { parsePage, renderBody } from "./docs";

const files = import.meta.glob("./pages/step-stack.md", { query: "?raw", import: "default", eager: true }) as Record<string, string>;
const page = parsePage("step-stack", Object.values(files)[0]);

describe("documentation of the step Stack", () => {
  it("lists Resampling and Drizzling as table rows that are not available yet, so the page greys them out", () => {
    const html = renderBody(page.body);
    for (const name of ["Resampling", "Drizzling"]) {
      expect(html, name).toMatch(new RegExp(String.raw`<tr class="off">\s*<td>${name}</td>`));
    }
  });

  it("no longer tells the reader that Resampling and Drizzling can be set", () => {
    expect(page.body).not.toContain("Resampling and Drizzling can be set");
    expect(page.body).not.toContain("appear next to it");
  });
});
