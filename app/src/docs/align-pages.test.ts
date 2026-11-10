import { describe, expect, it } from "vitest";
import { parsePage, renderBody, searchPages } from "./docs";

const files = import.meta.glob("./pages/*.md", { query: "?raw", import: "default", eager: true }) as Record<string, string>;
const pages = Object.entries(files).map(([path, raw]) => parsePage(path.replace(/^.*\/(.+)\.md$/, "$1"), raw));
const bySlug = new Map(pages.map((p) => [p.slug, p]));

const alignPages = ["align-your-frames", "frame-quality-and-limit"].map((slug) => {
  const page = bySlug.get(slug);
  if (!page) throw new Error(`Doc page "${slug}" is missing`);
  return page;
});

describe("documentation of the Align tab", () => {
  it("sits in the group Using the window, the Limit page after the Align page", () => {
    expect(alignPages.map((p) => p.group)).toEqual(["Using the window", "Using the window"]);
    expect(alignPages[0].order).toBeLessThan(alignPages[1].order);
    expect(alignPages.every((p) => Number.isFinite(p.order))).toBe(true);
  });

  it("links only to pages that exist, and not to anchors inside a page", () => {
    for (const page of alignPages) {
      expect(page.body).not.toContain("](#");
      for (const [, slug] of page.body.matchAll(/\]\(([a-z0-9-]+)\)/g)) {
        expect(bySlug.has(slug), `${page.slug} links to ${slug}`).toBe(true);
      }
    }
  });

  it("names the controls the way the window does", () => {
    const text = alignPages.map((p) => p.body).join("\n");
    for (const term of [
      "Set Alignpoints",
      "Align",
      "Limit",
      "R.o.Interest",
      "Show ROI",
      "Threshold",
      "Minimum distance between",
      "Min distance from edge",
      "Intensity selection",
      "Alignmentbox size",
      "Max Alignpoint movement",
      "Lowest Quality (%)",
      "Best Frames (%)",
      "Best Frames",
      "Frames/Apoint",
      "Framelist",
      "Registrationgraph",
    ]) {
      expect(text, term).toContain(term);
    }
  });

  it("describes all four limit modes in one table", () => {
    const html = renderBody(alignPages[1].body);
    for (const mode of ["Lowest Quality (%)", "Best Frames (%)", "Best Frames", "Frames/Apoint"]) {
      expect(html).toMatch(new RegExp(`<td>${mode.replace(/[()%/]/g, "\$&")}</td>`));
    }
  });

  it("says what does not work yet only on the overview page: No Align, Scan Frames and Empty List", () => {
    expect(alignPages.every((p) => !p.body.includes("Not available yet"))).toBe(true);
    const overview = bySlug.get("align-tab")?.body ?? "";
    const open = overview.split("\n").filter((line) => line.includes("Not available yet")).join("\n");
    for (const name of ["No Align", "Scan Frames", "Empty List"]) expect(open, name).toContain(name);
  });

  it("is found by searching for an alignpoint", () => {
    expect(searchPages(pages, "alignpoint").map((p) => p.slug)).toEqual(expect.arrayContaining(["align-your-frames"]));
    expect(searchPages(pages, "frames/apoint").map((p) => p.slug)).toContain("frame-quality-and-limit");
  });
});
