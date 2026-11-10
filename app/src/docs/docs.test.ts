import { describe, expect, it } from "vitest";
import { groupPages, parsePage, renderBody, searchPages, type DocPage } from "./docs";

const page = (slug: string, title: string, group: string, order: number, body = ""): DocPage => ({
  slug,
  title,
  group,
  order,
  body,
});

describe("parsePage", () => {
  it("reads the front matter and strips it from the body", () => {
    const raw = "---\ntitle: Select a recording\ngroup: Getting started\norder: 2\n---\n# Select a recording\n\nText.\n";
    expect(parsePage("select-a-recording", raw)).toEqual({
      slug: "select-a-recording",
      title: "Select a recording",
      group: "Getting started",
      order: 2,
      body: "# Select a recording\n\nText.\n",
    });
  });

  it("copes with Windows line endings", () => {
    const raw = "---\r\ntitle: A\r\ngroup: G\r\norder: 1\r\n---\r\nBody\r\n";
    expect(parsePage("a", raw).title).toBe("A");
    expect(parsePage("a", raw).body).toBe("Body\n");
  });

  it("rejects a page without front matter, naming the page", () => {
    expect(() => parsePage("broken", "# No header")).toThrow(/broken/);
  });

  it("rejects a page without a title", () => {
    expect(() => parsePage("untitled", "---\ngroup: G\norder: 1\n---\nBody")).toThrow(/title/);
  });
});

describe("groupPages", () => {
  it("groups by name in the order of each group's first page, pages sorted by order", () => {
    const pages = [
      page("formats", "Supported formats", "Reference", 10),
      page("select", "Select a recording", "Getting started", 2),
      page("welcome", "Welcome", "Getting started", 1),
    ];
    expect(groupPages(pages).map((g) => [g.name, g.pages.map((p) => p.slug)])).toEqual([
      ["Getting started", ["welcome", "select"]],
      ["Reference", ["formats"]],
    ]);
  });
});

describe("searchPages", () => {
  const pages = [
    page("welcome", "Welcome", "Getting started", 1, "SidereaPlanets stacks planets."),
    page("select", "Select a recording", "Getting started", 2, "Pick a SER or AVI file."),
    page("formats", "Supported formats", "Reference", 10, "SER files hold raw frames."),
  ];

  it("returns every page for an empty query", () => {
    expect(searchPages(pages, "  ")).toHaveLength(3);
  });

  it("matches title and body regardless of case, title hits first", () => {
    expect(searchPages(pages, "ser").map((p) => p.slug)).toEqual(["select", "formats"]);
    expect(searchPages(pages, "FORMATS").map((p) => p.slug)).toEqual(["formats"]);
  });

  it("requires every word to match", () => {
    expect(searchPages(pages, "ser raw").map((p) => p.slug)).toEqual(["formats"]);
    expect(searchPages(pages, "ser planets")).toEqual([]);
  });
});

describe("renderBody", () => {
  it("renders markdown to HTML", () => {
    expect(renderBody("## Steps\n\n1. One\n")).toContain("<h2");
    expect(renderBody("**Select**")).toContain("<strong>Select</strong>");
  });

  it("turns links to other pages into in-app links", () => {
    expect(renderBody("[Welcome](welcome)")).toContain('data-page="welcome"');
  });

  it("marks table rows that are not available yet", () => {
    const table = "| A | B |\n|---|---|\n| Works | Yes |\n| Later | Not available yet |\n";
    const html = renderBody(table);
    expect(html.match(/<tr class="off">/g)).toHaveLength(1);
    expect(html).toContain("<tr>");
  });

  it("leaves web links alone", () => {
    expect(renderBody("[SER](https://example.org/ser)")).not.toContain("data-page");
  });
});
